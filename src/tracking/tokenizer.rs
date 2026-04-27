//! Tokenizer module - Real LLM token counting via tiktoken (cl100k_base BPE).
//!
//! By default, WTK measures byte counts (the cheap heuristic: `bytes / 4 ≈ tokens`).
//! When `cl100k` mode is active, raw and filtered output are encoded with the
//! cl100k_base BPE used by GPT-3.5/4 — a high-fidelity proxy for Claude's
//! tokenizer (Anthropic does not publish theirs publicly, but cl100k correlates
//! within ~5% on typical English/code content).
//!
//! Selection precedence (highest first):
//!   1. CLI flag (`--tokens` on `wtk gain`, or per-call env via shell)
//!   2. `WTK_TOKENIZER` env var
//!   3. `~/.config/wtk/config.toml [tracking] tokenizer = "cl100k"`
//!   4. Default: `bytes` (current behavior, fast)

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// How to count tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenizerKind {
    /// No tokenizer — only byte/char counts are tracked. Default. Fast.
    Bytes,
    /// cl100k_base BPE (OpenAI GPT-3.5/4). High-fidelity proxy for Claude.
    Cl100k,
}

impl Default for TokenizerKind {
    fn default() -> Self {
        TokenizerKind::Bytes
    }
}

impl TokenizerKind {
    /// String label for display and DB storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenizerKind::Bytes => "bytes",
            TokenizerKind::Cl100k => "cl100k",
        }
    }
}

impl FromStr for TokenizerKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "bytes" | "byte" | "off" | "false" | "0" | "" => Ok(TokenizerKind::Bytes),
            "cl100k" | "tiktoken" | "real" | "true" | "1" => Ok(TokenizerKind::Cl100k),
            other => Err(format!("unknown tokenizer kind: {}", other)),
        }
    }
}

/// Lazy-loaded cl100k_base BPE. Loaded once on first use, ~10-30ms cold start.
static CL100K: Lazy<Option<tiktoken_rs::CoreBPE>> = Lazy::new(|| {
    match tiktoken_rs::cl100k_base() {
        Ok(bpe) => Some(bpe),
        Err(e) => {
            tracing::warn!("Failed to load cl100k_base tokenizer: {}", e);
            None
        }
    }
});

/// Count real tokens for the given text using the active tokenizer.
///
/// Returns `None` when `kind == Bytes` (no token count to record).
/// Returns `Some(n)` when a real tokenizer was used.
pub fn count(text: &str, kind: TokenizerKind) -> Option<usize> {
    match kind {
        TokenizerKind::Bytes => None,
        TokenizerKind::Cl100k => CL100K
            .as_ref()
            .map(|bpe| bpe.encode_with_special_tokens(text).len()),
    }
}

/// Resolve the active tokenizer kind from CLI override, env, or config default.
///
/// `cli_override` takes precedence. If `None`, falls back to env var
/// `WTK_TOKENIZER`, then to the value loaded from config.
pub fn resolve_kind(cli_override: Option<TokenizerKind>) -> TokenizerKind {
    if let Some(k) = cli_override {
        return k;
    }
    if let Ok(env_val) = std::env::var("WTK_TOKENIZER") {
        if let Ok(k) = env_val.parse() {
            return k;
        }
    }
    // Config file default — silent failure to bytes if config unreadable.
    crate::config::load()
        .ok()
        .and_then(|cfg| cfg.tracking.tokenizer.parse().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kind_aliases() {
        assert_eq!("bytes".parse::<TokenizerKind>().unwrap(), TokenizerKind::Bytes);
        assert_eq!("off".parse::<TokenizerKind>().unwrap(), TokenizerKind::Bytes);
        assert_eq!("cl100k".parse::<TokenizerKind>().unwrap(), TokenizerKind::Cl100k);
        assert_eq!("tiktoken".parse::<TokenizerKind>().unwrap(), TokenizerKind::Cl100k);
        assert!("xyz".parse::<TokenizerKind>().is_err());
    }

    #[test]
    fn count_bytes_returns_none() {
        assert_eq!(count("hello world", TokenizerKind::Bytes), None);
    }

    #[test]
    fn count_cl100k_returns_some() {
        let n = count("hello world", TokenizerKind::Cl100k);
        assert!(n.is_some());
        assert!(n.unwrap() > 0);
    }
}
