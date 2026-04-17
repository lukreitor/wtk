//! Claude Code hook installer.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Install Claude Code hooks.
pub fn install(global: bool) -> Result<()> {
    let settings_path = get_settings_path(global)?;

    // Create directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing settings or create new
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Get full path to wtk executable
    let wtk_path = std::env::current_exe()
        .context("Failed to get current executable path")?;

    // Convert to absolute path and normalize separators for Windows
    let wtk_path_str = wtk_path
        .canonicalize()
        .unwrap_or(wtk_path)
        .to_str()
        .context("Failed to convert path to string")?
        .replace("\\\\?\\", "") // Remove Windows UNC prefix
        .replace('\\', "/");    // Use forward slashes for compatibility

    // Add WTK hooks with full path
    let wtk_hook = serde_json::json!({
        "matcher": { "tool_name": "Bash" },
        "hooks": [{
            "type": "command",
            "command": format!("{} rewrite", wtk_path_str)
        }]
    });

    // Ensure hooks structure exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    // Add PreToolUse hook
    if settings["hooks"].get("PreToolUse").is_none() {
        settings["hooks"]["PreToolUse"] = serde_json::json!([]);
    }

    // Check if WTK hook already exists
    if let Some(hooks) = settings["hooks"]["PreToolUse"].as_array() {
        let already_installed = hooks.iter().any(|h| {
            h.get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter().any(|hook| {
                        hook.get("command")
                            .and_then(|c| c.as_str())
                            .map(|s| s.contains("wtk"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if already_installed {
            tracing::info!("WTK hook already installed");
            return Ok(());
        }
    }

    // Add the hook
    settings["hooks"]["PreToolUse"]
        .as_array_mut()
        .unwrap()
        .push(wtk_hook);

    // Write back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, content)
        .with_context(|| format!("Failed to write settings to {:?}", settings_path))?;

    tracing::info!("Installed Claude Code hooks to {:?}", settings_path);
    Ok(())
}

fn get_settings_path(global: bool) -> Result<PathBuf> {
    if global {
        // Global: ~/.claude/settings.json
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".claude").join("settings.json"))
    } else {
        // Local: .claude/settings.json
        Ok(PathBuf::from(".claude").join("settings.json"))
    }
}
