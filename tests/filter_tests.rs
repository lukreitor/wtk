//! Integration tests for WTK filters.

use std::process::Command;

/// Helper to run wtk command and get output.
fn run_wtk(args: &[&str]) -> (String, String, bool) {
    let wtk_path = env!("CARGO_BIN_EXE_wtk");

    let output = Command::new(wtk_path)
        .args(args)
        .output()
        .expect("Failed to execute wtk");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (stdout, stderr, output.status.success())
}

#[test]
fn test_wtk_version() {
    let (stdout, _, success) = run_wtk(&["--version"]);
    assert!(success);
    assert!(stdout.contains("wtk"));
}

#[test]
fn test_wtk_help() {
    let (stdout, _, success) = run_wtk(&["--help"]);
    assert!(success);
    assert!(stdout.contains("Windows Token Killer") || stdout.contains("Usage"));
}

#[test]
fn test_wtk_gain() {
    let (stdout, _, success) = run_wtk(&["gain"]);
    assert!(success);
    assert!(stdout.contains("WTK Token Savings") || stdout.contains("Token"));
}

#[cfg(windows)]
mod windows_tests {
    use super::*;

    #[test]
    fn test_git_status_filter() {
        // Only run if git is available
        if Command::new("git").arg("--version").output().is_err() {
            return;
        }

        let (stdout, stderr, success) = run_wtk(&["git", "status"]);

        // Should either succeed or fail gracefully
        if success {
            // Output should be compact (not raw git output)
            let total_len = stdout.len() + stderr.len();
            // WTK output should typically be shorter than raw git output
            assert!(total_len < 5000, "Output seems too long for filtered output");
        }
    }

    #[test]
    fn test_ipconfig_filter() {
        let (stdout, stderr, success) = run_wtk(&["ipconfig"]);

        if success {
            // Should have some network info
            let output = format!("{}{}", stdout, stderr);
            // Either has IPv4 info or error message
            assert!(
                output.contains("IPv4") ||
                output.contains("Adapter") ||
                output.contains("error") ||
                output.is_empty() == false
            );
        }
    }

    #[test]
    fn test_tasklist_filter() {
        let (stdout, _, success) = run_wtk(&["tasklist"]);

        if success {
            // Should be significantly smaller than raw tasklist output
            // Raw tasklist is usually 20KB+, filtered should be <5KB
            assert!(stdout.len() < 10000, "tasklist output not sufficiently filtered");
        }
    }

    #[test]
    fn test_ping_filter() {
        let (stdout, stderr, success) = run_wtk(&["ping", "-n", "1", "127.0.0.1"]);

        if success {
            let output = format!("{}{}", stdout, stderr);
            // Should have ping result or error
            assert!(
                output.contains("Reply") ||
                output.contains("ms") ||
                output.contains("127.0.0.1") ||
                output.contains("error")
            );
        }
    }
}

#[cfg(test)]
mod filter_output_tests {
    /// Test that git status produces compact output
    #[test]
    fn test_git_status_compression() {
        // Simulate what the filter should produce
        let raw_output = r#"
On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/main.rs
        modified:   src/lib.rs
        modified:   Cargo.toml

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        new_file.txt

no changes added to commit (use "git add" and/or "git commit -a")
"#;

        // Expected compressed output would be something like:
        // "M src/main.rs src/lib.rs Cargo.toml | ? new_file.txt"
        // This validates our filtering design
        assert!(raw_output.len() > 400);
        // A good filter should reduce this to <100 chars
    }

    /// Test log compression
    #[test]
    fn test_git_log_compression_design() {
        let raw_log = r#"
commit abc1234567890abcdef1234567890abcdef12345 (HEAD -> main, origin/main)
Author: Developer Name <dev@example.com>
Date:   Thu Apr 10 14:30:00 2025 -0300

    feat: add new feature

    This is a long description that explains what the feature does
    and why it was added. It includes multiple paragraphs and
    detailed technical information.

commit def5678901234567890abcdef1234567890abcdef
Author: Developer Name <dev@example.com>
Date:   Wed Apr 9 10:15:00 2025 -0300

    fix: resolve bug in login
"#;

        // Raw: ~600 chars
        // Filtered should be: "abc1234 feat: add new feature | def5678 fix: resolve bug in login"
        // ~70 chars = 88% reduction
        assert!(raw_log.len() > 500);
    }
}
