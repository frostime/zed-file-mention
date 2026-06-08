mod completion;
mod config;
mod index;
mod server;

use server::FileMentionsServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(FileMentionsServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
