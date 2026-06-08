use super::FileEntry;
use crate::config::IndexConfig;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ScoredEntry {
    pub entry: FileEntry,
    pub score: i64,
}

pub fn search(
    entries: &[FileEntry],
    query: &str,
    current_file: Option<&Path>,
    config: &IndexConfig,
) -> Vec<ScoredEntry> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default().ignore_case();
    let query_lower = query.to_lowercase();
    let is_path_query = query.contains('/') || query.contains('\\');
    let normalized_query = query.replace('\\', "/");

    let mut scored = entries
        .iter()
        .filter_map(|entry| {
            let score = score_entry(
                entry,
                &query_lower,
                &normalized_query,
                is_path_query,
                current_file,
                &matcher,
            )?;
            Some(ScoredEntry {
                entry: entry.clone(),
                score,
            })
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.entry.rel_path.cmp(&b.entry.rel_path))
    });
    scored.truncate(config.max_results);
    scored
}

fn score_entry(
    entry: &FileEntry,
    query_lower: &str,
    normalized_query: &str,
    is_path_query: bool,
    current_file: Option<&Path>,
    matcher: &SkimMatcherV2,
) -> Option<i64> {
    let file_lower = entry.file_name.to_lowercase();
    let stem_lower = entry.stem.to_lowercase();
    let path_lower = entry.rel_path.to_lowercase();
    let normalized_lower = normalized_query.to_lowercase();

    let mut score: i64 = 0;

    if file_lower == query_lower {
        score += 10_000;
    } else if file_lower.starts_with(query_lower) {
        score += 8_000;
    } else if stem_lower == query_lower {
        score += 7_000;
    } else if stem_lower.starts_with(query_lower) {
        score += 6_000;
    }

    if path_lower == normalized_lower {
        score += 9_000;
    } else if path_lower.starts_with(&normalized_lower) {
        score += 5_500;
    }

    if is_path_query && !path_lower.contains(&normalized_lower) {
        score += matcher.fuzzy_match(&path_lower, &normalized_lower)?;
    } else if !is_path_query {
        let file_fuzzy = matcher.fuzzy_match(&file_lower, query_lower);
        let path_fuzzy = matcher.fuzzy_match(&path_lower, query_lower);
        match (file_fuzzy, path_fuzzy) {
            (Some(file), Some(path)) => score += file + path / 3,
            (Some(file), None) => score += file,
            (None, Some(path)) => score += path / 2,
            (None, None) => {
                if score == 0 {
                    return None;
                }
            }
        }
    } else if path_lower.contains(&normalized_lower) {
        score += 3_500;
    }

    if let Some(current_file) = current_file {
        score += context_boost(entry, current_file);
    }

    score -= entry.depth as i64 * 10;
    score -= generated_or_test_penalty(&path_lower);

    if score <= 0 {
        None
    } else {
        Some(score)
    }
}

fn context_boost(entry: &FileEntry, current_file: &Path) -> i64 {
    let Ok(current_root_rel) = current_file.strip_prefix(&entry.root) else {
        return 0;
    };
    let current_parent = current_root_rel.parent();
    let entry_parent = Path::new(&entry.rel_path).parent();

    if current_parent == entry_parent {
        return 1_000;
    }

    let current_first = current_root_rel.components().next();
    let entry_first = Path::new(&entry.rel_path).components().next();
    if current_first.is_some() && current_first == entry_first {
        return 400;
    }

    150
}

fn generated_or_test_penalty(path_lower: &str) -> i64 {
    let mut penalty = 0;
    for marker in [
        "__snapshots__/",
        ".generated.",
        ".gen.",
        ".test.",
        ".spec.",
        "/tests/",
        "/test/",
    ] {
        if path_lower.contains(marker) {
            penalty += 200;
        }
    }
    penalty
}
