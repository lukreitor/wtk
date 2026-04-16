//! CMD hook installer.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Install CMD hooks (batch wrappers).
pub fn install(global: bool) -> Result<()> {
    let install_dir = get_install_dir(global)?;

    // Create directory
    fs::create_dir_all(&install_dir)?;

    // Create wrapper scripts for common commands
    let commands = ["git", "npm", "pnpm", "yarn", "docker", "kubectl"];

    for cmd in commands {
        let wrapper_path = install_dir.join(format!("{}.cmd", cmd));
        let wrapper_content = format!(
            r#"@echo off
REM WTK wrapper for {}
wtk.exe {} %*
"#,
            cmd, cmd
        );

        fs::write(&wrapper_path, wrapper_content)
            .with_context(|| format!("Failed to write wrapper for {}", cmd))?;

        tracing::debug!("Created wrapper: {:?}", wrapper_path);
    }

    // Print instructions
    println!();
    println!("CMD wrappers installed to: {:?}", install_dir);
    println!();
    println!("To enable, add this directory to your PATH:");
    println!("  set PATH={};%PATH%", install_dir.display());
    println!();

    Ok(())
}

fn get_install_dir(global: bool) -> Result<PathBuf> {
    if global {
        // Global: %LOCALAPPDATA%\wtk\bin
        let local_app_data =
            dirs::data_local_dir().context("Could not find local app data directory")?;
        Ok(local_app_data.join("wtk").join("bin"))
    } else {
        // Local: .wtk/bin
        Ok(PathBuf::from(".wtk").join("bin"))
    }
}
