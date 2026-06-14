use super::{EntryKind, FileEntry, FileIndex};
use crate::config::IndexConfig;
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn scan_roots(roots: &[PathBuf], config: &IndexConfig) -> Result<FileIndex> {
    let include = build_globset(&config.include).context("invalid include glob")?;
    let exclude = build_exclude_globset(&config.exclude).context("invalid exclude glob")?;

    let mut entries = Vec::new();
    let mut file_count = 0usize;
    let mut truncated = false;

    for root in roots {
        let root = root.canonicalize().unwrap_or_else(|_| root.clone());
        if !root.is_dir() {
            continue;
        }

        let root_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace")
            .to_string();

        let mut builder = WalkBuilder::new(&root);
        builder
            .hidden(!config.include_hidden)
            .git_ignore(config.respect_gitignore)
            .git_global(config.respect_gitignore)
            .git_exclude(config.respect_gitignore)
            .parents(config.respect_ignore_files)
            .ignore(config.respect_ignore_files)
            .follow_links(config.follow_symlinks);

        let filter_root = root.clone();
        let filter_exclude = exclude.clone();
        builder.filter_entry(move |dent| keep_entry(dent.path(), &filter_root, &filter_exclude));

        for result in builder.build() {
            let dent = match result {
                Ok(dent) => dent,
                Err(_) => continue,
            };
            let path = dent.path();
            let Some(file_type) = dent.file_type() else {
                continue;
            };
            let kind = if file_type.is_file() {
                EntryKind::File
            } else if file_type.is_dir() {
                EntryKind::Directory
            } else {
                continue;
            };

            let rel = match path.strip_prefix(&root) {
                Ok(rel) if !rel.as_os_str().is_empty() => rel,
                _ => continue,
            };

            if should_skip(rel, &kind, &include, &exclude) {
                continue;
            }

            if matches!(kind, EntryKind::File) && file_count >= config.max_files {
                truncated = true;
                break;
            }

            if let Some(entry) = entry_from_path(&root, &root_name, rel, kind) {
                entries.push(entry);
                if matches!(kind, EntryKind::File) {
                    file_count += 1;
                }
            }
        }

        if truncated {
            break;
        }
    }

    Ok(FileIndex {
        entries,
        truncated,
        updated_at: std::time::SystemTime::now(),
    })
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn build_exclude_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
        if let Some(dir_pattern) = pattern.strip_suffix("/**") {
            builder.add(Glob::new(dir_pattern)?);
            if let Some(root_dir_pattern) = dir_pattern.strip_prefix("**/") {
                builder.add(Glob::new(root_dir_pattern)?);
            }
        }
    }
    Ok(builder.build()?)
}

fn keep_entry(path: &Path, root: &Path, exclude: &GlobSet) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return true;
    };
    rel.as_os_str().is_empty() || !exclude.is_match(rel)
}

fn should_skip(rel: &Path, kind: &EntryKind, include: &GlobSet, exclude: &GlobSet) -> bool {
    if exclude.is_match(rel) {
        return true;
    }
    matches!(kind, EntryKind::File) && !include.is_match(rel)
}

fn entry_from_path(root: &Path, root_name: &str, rel: &Path, kind: EntryKind) -> Option<FileEntry> {
    let file_name = rel.file_name()?.to_str()?.to_string();
    let stem = match kind {
        EntryKind::File => rel.file_stem()?.to_str()?.to_string(),
        EntryKind::Directory => file_name.clone(),
    };
    let extension = match kind {
        EntryKind::File => rel
            .extension()
            .and_then(|ext| ext.to_str())
            .map(String::from),
        EntryKind::Directory => None,
    };
    let rel_path = normalize_path(rel);
    let depth = rel
        .components()
        .count()
        .saturating_sub(1)
        .min(u16::MAX as usize) as u16;

    Some(FileEntry {
        root: root.to_path_buf(),
        root_name: root_name.to_string(),
        rel_path,
        file_name,
        stem,
        extension,
        depth,
        kind,
    })
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scan_includes_directories_without_applying_file_include_globs() {
        let root = temp_root();
        fs::create_dir_all(root.join("src/empty")).unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();
        fs::write(root.join("README.md"), "").unwrap();
        fs::write(root.join("node_modules/pkg/index.js"), "").unwrap();

        let mut config = IndexConfig::default();
        config.include = vec!["**/*.rs".into()];

        let index = scan_roots(&[root.clone()], &config).unwrap();
        let entries = index
            .entries
            .into_iter()
            .map(|entry| (entry.rel_path, entry.kind))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(entries.get("src"), Some(&EntryKind::Directory));
        assert_eq!(entries.get("src/empty"), Some(&EntryKind::Directory));
        assert_eq!(entries.get("src/main.rs"), Some(&EntryKind::File));
        assert!(!entries.contains_key("README.md"));
        assert!(!entries.contains_key("node_modules"));
        assert!(!entries.contains_key("node_modules/pkg"));
        assert!(!entries.contains_key("node_modules/pkg/index.js"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn max_files_counts_files_not_directories() {
        let root = temp_root();
        fs::create_dir_all(root.join("aaa")).unwrap();
        fs::write(root.join("aaa/lib.rs"), "").unwrap();
        fs::write(root.join("aaa/main.rs"), "").unwrap();

        let mut config = IndexConfig::default();
        config.max_files = 1;

        let index = scan_roots(&[root.clone()], &config).unwrap();
        let file_count = index
            .entries
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File))
            .count();
        let entries = index
            .entries
            .into_iter()
            .map(|entry| (entry.rel_path, entry.kind))
            .collect::<BTreeMap<_, _>>();

        assert!(index.truncated);
        assert_eq!(file_count, 1);
        assert_eq!(entries.get("aaa"), Some(&EntryKind::Directory));
        assert!(entries.contains_key("aaa/lib.rs") || entries.contains_key("aaa/main.rs"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn max_files_zero_indexes_no_files() {
        let root = temp_root();
        fs::create_dir_all(root.join("aaa")).unwrap();
        fs::write(root.join("aaa/main.rs"), "").unwrap();

        let mut config = IndexConfig::default();
        config.max_files = 0;

        let index = scan_roots(&[root.clone()], &config).unwrap();
        let file_count = index
            .entries
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File))
            .count();
        let entries = index
            .entries
            .into_iter()
            .map(|entry| (entry.rel_path, entry.kind))
            .collect::<BTreeMap<_, _>>();

        assert!(index.truncated);
        assert_eq!(file_count, 0);
        assert_eq!(entries.get("aaa"), Some(&EntryKind::Directory));

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_directory_is_not_indexed_when_follow_symlinks_is_false() {
        use std::os::unix::fs::symlink;

        let root = temp_root();
        fs::create_dir_all(root.join("real_dir")).unwrap();
        symlink(root.join("real_dir"), root.join("linked_dir")).unwrap();

        let mut config = IndexConfig::default();
        config.follow_symlinks = false;

        let index = scan_roots(&[root.clone()], &config).unwrap();
        let entries = index
            .entries
            .into_iter()
            .map(|entry| (entry.rel_path, entry.kind))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(entries.get("real_dir"), Some(&EntryKind::Directory));
        assert!(!entries.contains_key("linked_dir"));

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("file_mentions_scanner_{nanos}"))
    }
}
