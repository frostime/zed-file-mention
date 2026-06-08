pub mod scanner;
pub mod scoring;
pub mod watcher;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub root: PathBuf,
    pub root_name: String,
    pub rel_path: String,
    pub file_name: String,
    pub stem: String,
    pub extension: Option<String>,
    pub depth: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    pub entries: Vec<FileEntry>,
    pub truncated: bool,
    #[serde(skip, default = "SystemTime::now")]
    pub updated_at: SystemTime,
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::empty()
    }
}

impl FileIndex {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            truncated: false,
            updated_at: SystemTime::now(),
        }
    }

    pub fn is_stale(&self, ttl_seconds: u64) -> bool {
        if ttl_seconds == 0 {
            return false;
        }
        self.updated_at
            .elapsed()
            .map(|elapsed| elapsed.as_secs() > ttl_seconds)
            .unwrap_or(false)
    }
}
