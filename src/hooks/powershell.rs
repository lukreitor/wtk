//! PowerShell hook installer.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const POWERSHELL_HOOK: &str = r#"
# WTK - Windows Token Killer Integration
# Added by: wtk init --powershell

function Invoke-WTK {
    param(
        [Parameter(Mandatory=$true, Position=0, ValueFromRemainingArguments=$true)]
        [string[]]$Arguments
    )

    $command = $Arguments -join ' '
    & wtk.exe $Arguments
}

# Aliases for common commands (optional)
# Set-Alias -Name git -Value { Invoke-WTK git $args }
# Set-Alias -Name npm -Value { Invoke-WTK npm $args }
# Set-Alias -Name docker -Value { Invoke-WTK docker $args }

Write-Host "[WTK] Windows Token Killer loaded" -ForegroundColor Green
"#;

/// Install PowerShell hooks.
pub fn install(global: bool) -> Result<()> {
    let profile_path = get_profile_path(global)?;

    // Create directory if it doesn't exist
    if let Some(parent) = profile_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing profile or create new
    let existing = if profile_path.exists() {
        fs::read_to_string(&profile_path)?
    } else {
        String::new()
    };

    // Check if WTK already installed
    if existing.contains("WTK - Windows Token Killer") {
        tracing::info!("WTK PowerShell hooks already installed");
        return Ok(());
    }

    // Append WTK hooks
    let new_content = format!("{}\n{}", existing.trim(), POWERSHELL_HOOK);
    fs::write(&profile_path, new_content)
        .with_context(|| format!("Failed to write profile to {:?}", profile_path))?;

    tracing::info!("Installed PowerShell hooks to {:?}", profile_path);
    Ok(())
}

fn get_profile_path(global: bool) -> Result<PathBuf> {
    if global {
        // Global PowerShell profile
        let docs = dirs::document_dir().context("Could not find documents directory")?;
        Ok(docs
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"))
    } else {
        // Local profile (current directory)
        Ok(PathBuf::from(".wtk").join("profile.ps1"))
    }
}
