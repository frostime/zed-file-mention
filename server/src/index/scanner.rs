use super::{FileEntry, FileIndex};
use crate::config::IndexConfig;
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn scan_roots(roots: &[PathBuf], config: &IndexConfig) -> Result<FileIndex> {
    let include = build_globset(&config.include).context("invalid include glob")?;
    let exclude = build_globset(&config.exclude).context("invalid exclude glob")?;

    let mut entries = Vec::new();
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

        for result in builder.build() {
            let dent = match result {
                Ok(dent) => dent,
                Err(_) => continue,
            };
            let path = dent.path();
            if !path.is_file() {
                continue;
            }

            let rel = match path.strip_prefix(&root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };

            if should_skip(rel, &include, &exclude) {
                continue;
            }

            if let Some(entry) = entry_from_path(&root, &root_name, rel) {
                entries.push(entry);
                if entries.len() >= config.max_files {
                    truncated = true;
                    break;
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

fn should_skip(rel: &Path, include: &GlobSet, exclude: &GlobSet) -> bool {
    if exclude.is_match(rel) {
        return true;
    }
    !include.is_match(rel)
}

fn entry_from_path(root: &Path, root_name: &str, rel: &Path) -> Option<FileEntry> {
    let file_name = rel.file_name()?.to_str()?.to_string();
    let stem = rel.file_stem()?.to_str()?.to_string();
    let extension = rel.extension().and_then(|ext| ext.to_str()).map(String::from);
    let rel_path = normalize_path(rel);
    let depth = rel.components().count().saturating_sub(1).min(u16::MAX as usize) as u16;

    Some(FileEntry {
        root: root.to_path_buf(),
        root_name: root_name.to_string(),
        rel_path,
        file_name,
        stem,
        extension,
        depth,
    })
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
