//! Find command filters (find, fd/fd-find).

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

pub struct FindFilter;

impl Filter for FindFilter {
    fn name(&self) -> &'static str {
        "find"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        // Match Unix find and fd-find.
        // Windows CMD `find` (string search in files) is NOT matched here —
        // that's handled by WindowsSystemFilter's `findstr` or passthrough.
        // In git-bash context `find` is always the Unix utility.
        matches!(cmd.as_str(), "find" | "fd" | "fd.exe" | "fdfind" | "fdfind.exe")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        // On Windows, `find` resolves to C:\Windows\System32\find.exe (string search).
        // Prefer the git-bash Unix find if available.
        let actual_cmd = if command.to_lowercase() == "find" || command.to_lowercase() == "find.exe" {
            resolve_unix_find().unwrap_or_else(|| command.to_string())
        } else {
            command.to_string()
        };

        let output = Command::new(&actual_cmd).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_find_output(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        75
    }
}

/// Resolve the Unix `find` utility path, preferring git-bash over Windows CMD find.exe.
fn resolve_unix_find() -> Option<String> {
    // Common git-bash / MSYS2 paths for Unix find
    let candidates = [
        "/usr/bin/find",
        "C:/Program Files/Git/usr/bin/find.exe",
        "C:/Program Files/Git/usr/bin/find",
    ];
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

fn filter_find_output(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        let err = stderr.trim();
        if !err.is_empty() {
            return format!("error: {}", err.lines().next().unwrap_or(err));
        }
        return "No files found".to_string();
    }

    // Group by parent directory
    let mut by_dir: HashMap<String, Vec<String>> = HashMap::new();

    for line in &lines {
        let path = Path::new(line);
        let parent = path
            .parent()
            .map(|p| {
                let s = p.to_string_lossy();
                if s.is_empty() { ".".to_string() } else { s.into_owned() }
            })
            .unwrap_or_else(|| ".".to_string());
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| line.to_string());
        by_dir.entry(parent).or_default().push(filename);
    }

    let total = lines.len();
    let mut result = vec![format!(
        "{} file{} in {} dir{}",
        total,
        if total == 1 { "" } else { "s" },
        by_dir.len(),
        if by_dir.len() == 1 { "" } else { "s" }
    )];

    let mut dirs: Vec<(&String, &Vec<String>)> = by_dir.iter().collect();
    dirs.sort_by(|a, b| a.0.cmp(b.0));

    for (dir, files) in dirs.iter().take(20) {
        result.push(format!(
            "{}/  ({})",
            shorten_path(dir),
            files.len()
        ));
        for f in files.iter().take(5) {
            result.push(format!("  {}", f));
        }
        if files.len() > 5 {
            result.push(format!("  ... +{} more", files.len() - 5));
        }
    }

    if dirs.len() > 20 {
        result.push(format!("... +{} more dirs", dirs.len() - 20));
    }

    result.join("\n")
}

fn shorten_path(path: &str) -> &str {
    if path.len() > 50 {
        &path[path.len() - 50..]
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_results() {
        let result = filter_find_output("", "");
        assert_eq!(result, "No files found");
    }

    #[test]
    fn test_groups_by_dir() {
        let stdout = "src/filters/git/mod.rs\nsrc/filters/git/status.rs\nsrc/main.rs\n";
        let result = filter_find_output(stdout, "");
        assert!(result.contains("3 files in 2 dirs"));
        assert!(result.contains("src/filters/git"));
    }

    #[test]
    fn test_single_file() {
        let stdout = "src/main.rs\n";
        let result = filter_find_output(stdout, "");
        assert!(result.contains("1 file in 1 dir"));
    }

    #[test]
    fn test_error() {
        let result = filter_find_output("", "find: 'missing': No such file or directory");
        assert!(result.starts_with("error:"));
    }
}
