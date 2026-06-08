use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub index: IndexConfig,
    pub insert: InsertConfig,
    pub completion: CompletionConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            index: IndexConfig::default(),
            insert: InsertConfig::default(),
            completion: CompletionConfig::default(),
        }
    }
}

impl Config {
    pub fn from_initialization_options(value: Option<Value>) -> Self {
        let mut config = Config::default();
        if let Some(value) = value {
            config.merge_value(&value);
        }
        config
    }

    pub fn merge_value(&mut self, value: &Value) {
        if let Ok(partial) = serde_json::from_value::<PartialConfig>(value.clone()) {
            if let Some(index) = partial.index {
                self.index.merge(index);
            }
            if let Some(insert) = partial.insert {
                self.insert.merge(insert);
            }
            if let Some(completion) = partial.completion {
                self.completion.merge(completion);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct IndexConfig {
    pub respect_gitignore: bool,
    pub respect_ignore_files: bool,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub watch_files: bool,
    pub max_files: usize,
    pub max_results: usize,
    pub refresh_ttl_seconds: u64,
    pub debounce_ms: u64,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            respect_gitignore: true,
            respect_ignore_files: true,
            include_hidden: false,
            follow_symlinks: false,
            watch_files: true,
            max_files: 100_000,
            max_results: 50,
            refresh_ttl_seconds: 60,
            debounce_ms: 700,
            include: vec!["**/*".into()],
            exclude: default_excludes(),
        }
    }
}

impl IndexConfig {
    fn merge(&mut self, partial: PartialIndexConfig) {
        if let Some(value) = partial.respect_gitignore {
            self.respect_gitignore = value;
        }
        if let Some(value) = partial.respect_ignore_files {
            self.respect_ignore_files = value;
        }
        if let Some(value) = partial.include_hidden {
            self.include_hidden = value;
        }
        if let Some(value) = partial.follow_symlinks {
            self.follow_symlinks = value;
        }
        if let Some(value) = partial.watch_files {
            self.watch_files = value;
        }
        if let Some(value) = partial.max_files {
            self.max_files = value;
        }
        if let Some(value) = partial.max_results {
            self.max_results = value;
        }
        if let Some(value) = partial.refresh_ttl_seconds {
            self.refresh_ttl_seconds = value;
        }
        if let Some(value) = partial.debounce_ms {
            self.debounce_ms = value;
        }
        if let Some(include) = partial.include {
            self.include = include;
        }
        if let Some(mut exclude) = partial.exclude {
            // User excludes are additive. Built-in hygiene excludes remain active unless the
            // source code is changed intentionally.
            self.exclude.append(&mut exclude);
            self.exclude.sort();
            self.exclude.dedup();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct InsertConfig {
    pub keep_trigger: bool,
    pub path_style: PathStyle,
    pub quote_paths_with_spaces: bool,
}

impl Default for InsertConfig {
    fn default() -> Self {
        Self {
            keep_trigger: true,
            path_style: PathStyle::Relative,
            quote_paths_with_spaces: false,
        }
    }
}

impl InsertConfig {
    fn merge(&mut self, partial: PartialInsertConfig) {
        if let Some(value) = partial.keep_trigger {
            self.keep_trigger = value;
        }
        if let Some(value) = partial.path_style {
            self.path_style = value;
        }
        if let Some(value) = partial.quote_paths_with_spaces {
            self.quote_paths_with_spaces = value;
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PathStyle {
    Relative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CompletionConfig {
    pub trigger: String,
    pub min_query_len: usize,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            trigger: "@".into(),
            min_query_len: 1,
        }
    }
}

impl CompletionConfig {
    fn merge(&mut self, partial: PartialCompletionConfig) {
        if let Some(value) = partial.trigger {
            self.trigger = value;
        }
        if let Some(value) = partial.min_query_len {
            self.min_query_len = value;
        }
    }
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    index: Option<PartialIndexConfig>,
    insert: Option<PartialInsertConfig>,
    completion: Option<PartialCompletionConfig>,
}

#[derive(Debug, Deserialize)]
struct PartialIndexConfig {
    respect_gitignore: Option<bool>,
    respect_ignore_files: Option<bool>,
    include_hidden: Option<bool>,
    follow_symlinks: Option<bool>,
    watch_files: Option<bool>,
    max_files: Option<usize>,
    max_results: Option<usize>,
    refresh_ttl_seconds: Option<u64>,
    debounce_ms: Option<u64>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PartialInsertConfig {
    keep_trigger: Option<bool>,
    path_style: Option<PathStyle>,
    quote_paths_with_spaces: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PartialCompletionConfig {
    trigger: Option<String>,
    min_query_len: Option<usize>,
}

pub fn default_excludes() -> Vec<String> {
    [
        "**/.git/**",
        "**/node_modules/**",
        "**/.venv/**",
        "**/venv/**",
        "**/dist/**",
        "**/build/**",
        "**/target/**",
        "**/.next/**",
        "**/coverage/**",
        "**/__pycache__/**",
        "**/.pytest_cache/**",
        "**/.mypy_cache/**",
        "**/.ruff_cache/**",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
