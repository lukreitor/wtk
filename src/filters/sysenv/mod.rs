//! System environment variable filters (env, printenv).

use anyhow::Result;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

pub struct EnvFilter;

impl Filter for EnvFilter {
    fn name(&self) -> &'static str {
        "env"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        matches!(cmd.as_str(), "env" | "env.exe" | "printenv" | "printenv.exe")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let raw = format!("{}{}", stdout, stderr);

        // If a specific var is requested (e.g. `printenv HOME`), pass through as-is
        let has_var_arg = args.iter().any(|a| !a.starts_with('-') && !a.is_empty());
        if has_var_arg {
            return Ok(FilterResult::with_raw(stdout.trim().to_string(), raw, exec_time_ms));
        }

        let filtered = filter_env_output(&stdout);
        Ok(FilterResult::with_raw(filtered, raw, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        70
    }
}

// System-noise variable prefixes to drop (uppercase comparison)
const SKIP_PREFIXES: &[&str] = &[
    "ALLUSERSPROFILE=",
    "APPDATA=",
    "COMMONPROGRAMFILES=",
    "COMMONPROGRAMW6432=",
    "COMPUTERNAME=",
    "COMSPEC=",
    "DRIVERDATA=",
    "LOCALAPPDATA=",
    "LOGONSERVER=",
    "NUMBER_OF_PROCESSORS=",
    "OS=",
    "PATHEXT=",
    "PROCESSOR_ARCHITECTURE=",
    "PROCESSOR_IDENTIFIER=",
    "PROCESSOR_LEVEL=",
    "PROCESSOR_REVISION=",
    "PROGRAMDATA=",
    "PROGRAMFILES=",
    "PROGRAMW6432=",
    "PSMODULEPATH=",
    "PUBLIC=",
    "SESSIONNAME=",
    "SYSTEMDRIVE=",
    "SYSTEMROOT=",
    "TEMP=",
    "TMP=",
    "USERDOMAIN=",
    "USERDOMAIN_ROAMINGPROFILE=",
    "USERNAME=",
    "USERPROFILE=",
    "WINDIR=",
    "__COMPAT_LAYER=",
    // Git-bash internal
    "EXEPATH=",
    "INFOPATH=",
    "MANPATH=",
    "MSYSTEM=",
    "MSYSTEM_CARCH=",
    "MSYSTEM_CHOST=",
    "MSYSTEM_PREFIX=",
    "MINGW_CHOST=",
    "MINGW_MOUNT_POINT=",
    "MINGW_PREFIX=",
    "ACLOCAL_PATH=",
    "PKG_CONFIG_PATH=",
    "PKG_CONFIG_SYSTEM_INCLUDE_PATH=",
    "PKG_CONFIG_SYSTEM_LIBRARY_PATH=",
    "ORIGINAL_PATH=",
    "ORIGINAL_TEMP=",
    "ORIGINAL_TMP=",
    "PLINK_PROTOCOL=",
    "SHELL=",
    "SHLVL=",
    "TERM=",
];

fn filter_env_output(stdout: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    let mut path_entry: Option<String> = None;
    let mut skipped = 0usize;

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let upper = line.to_ascii_uppercase();

        // Separate PATH for special handling
        if upper.starts_with("PATH=") {
            path_entry = Some(format_path(line));
            continue;
        }

        // Skip known noise vars
        if SKIP_PREFIXES
            .iter()
            .any(|p| upper.starts_with(&p.to_ascii_uppercase()))
        {
            skipped += 1;
            continue;
        }

        // Skip vars with empty values
        if let Some(val) = line.splitn(2, '=').nth(1) {
            if val.trim().is_empty() {
                skipped += 1;
                continue;
            }
        }

        // Mask sensitive values
        let entry = if is_sensitive_var(line) {
            let key = line.splitn(2, '=').next().unwrap_or(line);
            format!("{}=***", key)
        } else if line.len() > 120 {
            format!("{}...", &line[..117])
        } else {
            line.to_string()
        };
        kept.push(entry);
    }

    kept.sort();

    let mut result = Vec::new();
    result.push(format!(
        "{} env var{} ({} system vars hidden)",
        kept.len() + path_entry.is_some() as usize,
        if kept.len() + path_entry.is_some() as usize == 1 { "" } else { "s" },
        skipped
    ));

    if let Some(path) = path_entry {
        result.push(path);
    }

    result.extend(kept);
    result.join("\n")
}

const SENSITIVE_SUFFIXES: &[&str] = &[
    "_KEY", "_SECRET", "_TOKEN", "_PASSWORD", "_PASSWD", "_PASS",
    "_CREDENTIAL", "_CREDENTIALS", "_PRIVATE", "_AUTH", "_API_KEY",
    "_ACCESS_KEY", "_SECRET_KEY",
];

fn is_sensitive_var(line: &str) -> bool {
    let key = line.splitn(2, '=').next().unwrap_or("").to_ascii_uppercase();
    SENSITIVE_SUFFIXES.iter().any(|s| key.ends_with(s))
}

fn format_path(line: &str) -> String {
    let val = line.splitn(2, '=').nth(1).unwrap_or("");
    // PATH can be : separated (Unix) or ; separated (Windows)
    let entries: Vec<&str> = if val.contains(';') {
        val.split(';').filter(|s| !s.trim().is_empty()).collect()
    } else {
        val.split(':').filter(|s| !s.trim().is_empty()).collect()
    };

    let count = entries.len();
    let mut lines = vec![format!("PATH ({} entries):", count)];

    for entry in entries.iter().take(6) {
        lines.push(format!("  {}", truncate(entry, 70)));
    }
    if count > 6 {
        lines.push(format!("  ... +{} more", count - 6));
    }

    lines.join("\n")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() > max { &s[..max] } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hides_system_vars() {
        let stdout = "COMPUTERNAME=PC\nMY_VAR=hello\nNODE_ENV=production\n";
        let result = filter_env_output(stdout);
        assert!(!result.contains("COMPUTERNAME"));
        assert!(result.contains("MY_VAR"));
        assert!(result.contains("NODE_ENV"));
        assert!(result.contains("1 system vars hidden"));
    }

    #[test]
    fn test_formats_path() {
        let stdout = "PATH=/usr/bin:/usr/local/bin\nMY_VAR=val\n";
        let result = filter_env_output(stdout);
        assert!(result.contains("PATH (2 entries)"));
        assert!(result.contains("/usr/bin"));
    }

    #[test]
    fn test_passthrough_empty() {
        let result = filter_env_output("");
        assert!(result.contains("0 env vars"));
    }
}
