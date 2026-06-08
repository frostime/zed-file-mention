use super::{scanner, FileIndex};
use crate::config::Config;
use anyhow::Result;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tower_lsp::Client;

pub fn spawn_watchers(
    roots: Vec<PathBuf>,
    config: Config,
    index: Arc<RwLock<FileIndex>>,
    client: Client,
) -> Result<Vec<RecommendedWatcher>> {
    if !config.index.watch_files {
        return Ok(Vec::new());
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watchers = Vec::new();

    for root in &roots {
        if !root.is_dir() {
            continue;
        }
        let tx = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = tx.send(event);
            },
            NotifyConfig::default(),
        )?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        watchers.push(watcher);
    }

    let handle = Handle::current();
    thread::spawn(move || {
        let debounce = Duration::from_millis(config.index.debounce_ms.max(100));
        let mut last_event = Instant::now();
        let mut pending = false;

        loop {
            match rx.recv_timeout(debounce) {
                Ok(Ok(event)) => {
                    if should_ignore_event(&event, &config) {
                        continue;
                    }
                    pending = true;
                    last_event = Instant::now();
                }
                Ok(Err(err)) => {
                    log_with_handle(
                        &handle,
                        &client,
                        tower_lsp::lsp_types::MessageType::WARNING,
                        format!("file watcher error: {err}"),
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if pending && last_event.elapsed() >= debounce {
                        pending = false;
                        rebuild(&roots, &config, &index, &client, &handle);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Ok(watchers)
}

pub fn spawn_ttl_refresh(
    roots: Vec<PathBuf>,
    config: Config,
    index: Arc<RwLock<FileIndex>>,
    client: Client,
) {
    if config.index.refresh_ttl_seconds == 0 {
        return;
    }

    let handle = Handle::current();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(config.index.refresh_ttl_seconds));
        let stale = index
            .read()
            .map(|index| index.is_stale(config.index.refresh_ttl_seconds))
            .unwrap_or(false);
        if stale {
            rebuild(&roots, &config, &index, &client, &handle);
        }
    });
}

pub fn rebuild(
    roots: &[PathBuf],
    config: &Config,
    index: &Arc<RwLock<FileIndex>>,
    client: &Client,
    handle: &Handle,
) {
    match scanner::scan_roots(roots, &config.index) {
        Ok(new_index) => {
            let len = new_index.entries.len();
            let truncated = new_index.truncated;
            if let Ok(mut guard) = index.write() {
                *guard = new_index;
            }
            log_with_handle(
                handle,
                client,
                tower_lsp::lsp_types::MessageType::LOG,
                format!(
                    "File Mentions index refreshed: {len} files{}",
                    if truncated { " (truncated)" } else { "" }
                ),
            );
        }
        Err(err) => {
            log_with_handle(
                handle,
                client,
                tower_lsp::lsp_types::MessageType::WARNING,
                format!("File Mentions index refresh failed: {err:#}"),
            );
        }
    }
}

fn log_with_handle(
    handle: &Handle,
    client: &Client,
    ty: tower_lsp::lsp_types::MessageType,
    message: String,
) {
    let client = client.clone();
    handle.spawn(async move {
        client.log_message(ty, message).await;
    });
}

fn should_ignore_event(event: &Event, config: &Config) -> bool {
    let excludes = &config.index.exclude;
    event.paths.iter().all(|path| {
        let text = path.to_string_lossy().replace('\\', "/");
        excludes.iter().any(|pattern| {
            let needle = pattern.trim_start_matches("**/").trim_end_matches("/**");
            !needle.is_empty() && text.contains(needle)
        })
    })
}
