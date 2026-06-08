use zed::settings::LspSettings;
use zed_extension_api as zed;

const SERVER_ID: &str = "file-mentions-lsp";
const SERVER_BINARY: &str = "file-mentions-lsp";

struct FileMentionsExtension;

impl zed::Extension for FileMentionsExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != SERVER_ID {
            return Err(format!("unknown language server: {language_server_id}"));
        }

        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?;
        if let Some(binary) = settings.binary {
            if let Some(path) = binary.path {
                let args = binary.arguments.unwrap_or_default();
                let env = binary.env.unwrap_or_default().into_iter().collect::<Vec<_>>();
                return Ok(zed::Command {
                    command: path,
                    args,
                    env,
                });
            }
        }

        if let Some(path) = worktree.which(SERVER_BINARY) {
            return Ok(zed::Command {
                command: path,
                args: vec![],
                env: vec![],
            });
        }

        Err(
            "file-mentions-lsp was not found. Build it with `cargo build --manifest-path server/Cargo.toml --release`, then either add the binary to PATH or configure `lsp.file-mentions-lsp.binary.path` in Zed settings."
                .to_string(),
        )
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?;
        Ok(settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?;
        Ok(settings.settings)
    }
}

zed::register_extension!(FileMentionsExtension);
