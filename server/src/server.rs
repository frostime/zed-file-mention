use crate::completion::{completion_item_for_score, extract_mention_token};
use crate::config::Config;
use crate::index::{scanner, scoring, watcher, FileIndex};
use notify::RecommendedWatcher;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{async_trait, Client, LanguageServer};
use url::Url;

#[derive(Debug, Clone)]
struct RuntimeState {
    config: Config,
    roots: Vec<PathBuf>,
}

pub struct FileMentionsServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
    index: Arc<RwLock<FileIndex>>,
    runtime: Arc<RwLock<Option<RuntimeState>>>,
    watchers: Arc<Mutex<Vec<RecommendedWatcher>>>,
}

impl FileMentionsServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(FileIndex::empty())),
            runtime: Arc::new(RwLock::new(None)),
            watchers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn current_file_path(&self, uri: &Url) -> Option<PathBuf> {
        uri.to_file_path().ok()
    }
}

#[async_trait]
impl LanguageServer for FileMentionsServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        let config = Config::from_initialization_options(params.initialization_options);
        let roots = workspace_roots(&params);

        if let Ok(mut guard) = self.runtime.write() {
            *guard = Some(RuntimeState {
                config: config.clone(),
                roots,
            });
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![config.completion.trigger.clone()]),
                    ..CompletionOptions::default()
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "file-mentions-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let Some(runtime) = self.runtime.read().ok().and_then(|guard| guard.clone()) else {
            return;
        };

        if runtime.roots.is_empty() {
            self.client
                .log_message(MessageType::WARNING, "File Mentions: no workspace root found")
                .await;
            return;
        }

        let roots = runtime.roots.clone();
        let config = runtime.config.clone();
        let index = self.index.clone();
        let client = self.client.clone();
        let handle = tokio::runtime::Handle::current();

        tokio::task::spawn_blocking(move || match scanner::scan_roots(&roots, &config.index) {
            Ok(new_index) => {
                let len = new_index.entries.len();
                let truncated = new_index.truncated;
                if let Ok(mut guard) = index.write() {
                    *guard = new_index;
                }
                handle.spawn(async move {
                    client
                        .log_message(
                            MessageType::LOG,
                            format!(
                                "File Mentions index ready: {len} files{}",
                                if truncated { " (truncated)" } else { "" }
                            ),
                        )
                        .await;
                });
            }
            Err(err) => {
                handle.spawn(async move {
                    client
                        .log_message(
                            MessageType::WARNING,
                            format!("File Mentions initial index failed: {err:#}"),
                        )
                        .await;
                });
            }
        });

        match watcher::spawn_watchers(
            runtime.roots.clone(),
            runtime.config.clone(),
            self.index.clone(),
            self.client.clone(),
        ) {
            Ok(new_watchers) => {
                if let Ok(mut watchers) = self.watchers.lock() {
                    *watchers = new_watchers;
                }
            }
            Err(err) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("File Mentions watcher setup failed: {err:#}"),
                    )
                    .await;
            }
        }

        watcher::spawn_ttl_refresh(
            runtime.roots,
            runtime.config,
            self.index.clone(),
            self.client.clone(),
        );
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Ok(mut docs) = self.documents.write() {
            docs.insert(params.text_document.uri, params.text_document.text);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            if let Ok(mut docs) = self.documents.write() {
                docs.insert(params.text_document.uri, change.text);
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(mut docs) = self.documents.write() {
            docs.remove(&params.text_document.uri);
        }
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let Some(runtime) = self.runtime.read().ok().and_then(|guard| guard.clone()) else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let text = self
            .documents
            .read()
            .ok()
            .and_then(|docs| docs.get(&uri).cloned());
        let Some(text) = text else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };

        let Some(token) = extract_mention_token(&text, position, &runtime.config) else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };

        let current_file = self.current_file_path(&uri);
        let entries = self
            .index
            .read()
            .map(|index| index.entries.clone())
            .unwrap_or_default();

        let scored = scoring::search(
            &entries,
            &token.query,
            current_file.as_deref(),
            &runtime.config.index,
        );

        let items = scored
            .into_iter()
            .enumerate()
            .map(|(rank, scored)| completion_item_for_score(scored, &token, rank, &runtime.config))
            .collect::<Vec<_>>();

        Ok(Some(CompletionResponse::Array(items)))
    }
}

fn workspace_roots(params: &InitializeParams) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(folders) = &params.workspace_folders {
        for folder in folders {
            if let Some(path) = url_to_existing_dir(&folder.uri) {
                roots.push(path);
            }
        }
    }

    #[allow(deprecated)]
    if roots.is_empty() {
        if let Some(uri) = &params.root_uri {
            if let Some(path) = url_to_existing_dir(uri) {
                roots.push(path);
            }
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

fn url_to_existing_dir(uri: &Url) -> Option<PathBuf> {
    let path = uri.to_file_path().ok()?;
    let path = path.canonicalize().unwrap_or(path);
    if path.is_dir() {
        Some(path)
    } else {
        path.parent().map(Path::to_path_buf)
    }
}
