use crate::config::Config;
use crate::index::scoring::ScoredEntry;
use std::path::Path;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Position, Range, TextEdit,
};

#[derive(Debug, Clone)]
pub struct MentionToken {
    pub query: String,
    pub range: Range,
}

pub fn extract_mention_token(text: &str, position: Position, config: &Config) -> Option<MentionToken> {
    let line = text.lines().nth(position.line as usize)?;
    let cursor_byte = byte_index_from_utf16(line, position.character as usize)?;
    let before = &line[..cursor_byte];

    let start_byte = scan_token_start(before, &config.completion.trigger)?;
    let token = &line[start_byte..cursor_byte];
    let trigger = config.completion.trigger.as_str();

    if !token.starts_with(trigger) {
        return None;
    }
    if token[trigger.len()..].contains(trigger) {
        return None;
    }
    if start_byte > 0 {
        let boundary = line[..start_byte].chars().last()?;
        if !is_boundary_char(boundary) {
            return None;
        }
    }

    let query = token[trigger.len()..].to_string();
    if query.len() < config.completion.min_query_len {
        return None;
    }

    let start_character = utf16_len(&line[..start_byte]) as u32;
    Some(MentionToken {
        query,
        range: Range {
            start: Position {
                line: position.line,
                character: start_character,
            },
            end: position,
        },
    })
}

pub fn completion_item_for_score(
    scored: ScoredEntry,
    token: &MentionToken,
    rank: usize,
    config: &Config,
) -> CompletionItem {
    let entry = scored.entry;
    let display_parent = Path::new(&entry.rel_path)
        .parent()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| ".".into());

    let label = format!("{} — {display_parent}/", entry.file_name);
    let mut new_text = entry.rel_path.clone();
    if config.insert.keep_trigger {
        new_text = format!("{}{}", config.completion.trigger, new_text);
    }
    if config.insert.quote_paths_with_spaces && new_text.contains(' ') {
        new_text = format!("\"{new_text}\"");
    }

    CompletionItem {
        label,
        kind: Some(CompletionItemKind::FILE),
        detail: Some(format!("{} · {}", entry.rel_path, entry.root_name)),
        filter_text: Some(format!("{} {}", entry.file_name, entry.rel_path)),
        sort_text: Some(format!("{:04}", rank)),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: token.range,
            new_text,
        })),
        data: Some(serde_json::json!({
            "rel_path": entry.rel_path,
            "score": scored.score
        })),
        ..CompletionItem::default()
    }
}

fn scan_token_start(before: &str, trigger: &str) -> Option<usize> {
    let bytes = before.as_bytes();
    let mut idx = bytes.len();
    while idx > 0 {
        let ch = before[..idx].chars().last()?;
        if is_token_char(ch) || trigger.contains(ch) {
            idx -= ch.len_utf8();
            continue;
        }
        break;
    }
    Some(idx)
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '\\')
}

fn is_boundary_char(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '"' | '\'' | '`' | ':' | '=')
}

fn byte_index_from_utf16(line: &str, utf16_idx: usize) -> Option<usize> {
    if utf16_idx == 0 {
        return Some(0);
    }
    let mut units = 0usize;
    for (byte_idx, ch) in line.char_indices() {
        if units == utf16_idx {
            return Some(byte_idx);
        }
        units += ch.len_utf16();
        if units > utf16_idx {
            return None;
        }
    }
    if units == utf16_idx {
        Some(line.len())
    } else {
        None
    }
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_mentions() {
        let config = Config::default();
        let token = extract_mention_token(
            "open @index.ts",
            Position {
                line: 0,
                character: 14,
            },
            &config,
        )
        .unwrap();
        assert_eq!(token.query, "index.ts");
    }

    #[test]
    fn rejects_email_context() {
        let config = Config::default();
        assert!(extract_mention_token(
            "foo@example.com",
            Position {
                line: 0,
                character: 15,
            },
            &config,
        )
        .is_none());
    }
}
