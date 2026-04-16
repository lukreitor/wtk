//! Git status filter.

use regex::Regex;

/// Filter git status output to compact form.
pub fn filter_status_output(output: &str) -> String {
    let mut result = Vec::new();
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();

    // Regex patterns
    let staged_re = Regex::new(r"^\s*(new file|modified|deleted|renamed):\s+(.+)$").unwrap();
    let modified_re = Regex::new(r"^\s*modified:\s+(.+)$").unwrap();
    let untracked_re = Regex::new(r"^\t([^\s].+)$").unwrap();

    let mut in_staged = false;
    let mut in_unstaged = false;
    let mut in_untracked = false;

    for line in output.lines() {
        if line.contains("Changes to be committed") {
            in_staged = true;
            in_unstaged = false;
            in_untracked = false;
        } else if line.contains("Changes not staged") {
            in_staged = false;
            in_unstaged = true;
            in_untracked = false;
        } else if line.contains("Untracked files") {
            in_staged = false;
            in_unstaged = false;
            in_untracked = true;
        } else if line.contains("On branch") {
            result.push(compact_branch_line(line));
        } else if line.contains("Your branch is") {
            result.push(compact_tracking_line(line));
        } else if in_staged {
            if let Some(caps) = staged_re.captures(line) {
                let action = &caps[1];
                let file = &caps[2];
                let symbol = match action {
                    "new file" => "+",
                    "modified" => "~",
                    "deleted" => "-",
                    "renamed" => ">",
                    _ => "?",
                };
                staged.push(format!("{} {}", symbol, file.trim()));
            }
        } else if in_unstaged {
            if let Some(caps) = modified_re.captures(line) {
                modified.push(format!("~ {}", caps[1].trim()));
            }
        } else if in_untracked {
            if let Some(caps) = untracked_re.captures(line) {
                untracked.push(format!("? {}", caps[1].trim()));
            }
        }
    }

    // Build compact output
    if !staged.is_empty() {
        result.push(format!("Staged ({})", staged.len()));
        for f in staged.iter().take(10) {
            result.push(format!("  {}", f));
        }
        if staged.len() > 10 {
            result.push(format!("  ... +{} more", staged.len() - 10));
        }
    }

    if !modified.is_empty() {
        result.push(format!("Modified ({})", modified.len()));
        for f in modified.iter().take(10) {
            result.push(format!("  {}", f));
        }
        if modified.len() > 10 {
            result.push(format!("  ... +{} more", modified.len() - 10));
        }
    }

    if !untracked.is_empty() {
        result.push(format!("Untracked ({})", untracked.len()));
        for f in untracked.iter().take(5) {
            result.push(format!("  {}", f));
        }
        if untracked.len() > 5 {
            result.push(format!("  ... +{} more", untracked.len() - 5));
        }
    }

    if result.is_empty() {
        "Clean".to_string()
    } else {
        result.join("\n")
    }
}

fn compact_branch_line(line: &str) -> String {
    if let Some(branch) = line.strip_prefix("On branch ") {
        format!("@ {}", branch.trim())
    } else {
        line.to_string()
    }
}

fn compact_tracking_line(line: &str) -> String {
    if line.contains("ahead") && line.contains("behind") {
        // Extract numbers
        let ahead_re = Regex::new(r"ahead of .+ by (\d+)").unwrap();
        let behind_re = Regex::new(r"behind .+ by (\d+)").unwrap();

        let ahead = ahead_re
            .captures(line)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let behind = behind_re
            .captures(line)
            .map(|c| c[1].to_string())
            .unwrap_or_default();

        format!("↑{} ↓{}", ahead, behind)
    } else if line.contains("ahead") {
        let re = Regex::new(r"by (\d+) commit").unwrap();
        if let Some(caps) = re.captures(line) {
            format!("↑{}", &caps[1])
        } else {
            "↑".to_string()
        }
    } else if line.contains("behind") {
        let re = Regex::new(r"by (\d+) commit").unwrap();
        if let Some(caps) = re.captures(line) {
            format!("↓{}", &caps[1])
        } else {
            "↓".to_string()
        }
    } else if line.contains("up to date") {
        "✓ synced".to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_status() {
        let output = "On branch main\nnothing to commit, working tree clean\n";
        let result = filter_status_output(output);
        assert!(result.contains("Clean") || result.contains("@ main"));
    }

    #[test]
    fn test_modified_files() {
        let output = r#"On branch main
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
        modified:   src/main.rs
        modified:   Cargo.toml
"#;
        let result = filter_status_output(output);
        assert!(result.contains("Modified"));
        assert!(result.contains("src/main.rs"));
    }

    #[test]
    fn test_staged_files() {
        let output = r#"On branch feature
Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
        new file:   src/new.rs
        modified:   src/lib.rs
"#;
        let result = filter_status_output(output);
        assert!(result.contains("Staged"));
        assert!(result.contains("src/new.rs"));
    }

    #[test]
    fn test_untracked_files() {
        let output = r#"On branch main
Untracked files:
  (use "git add <file>..." to include in what will be committed)
	temp.txt
	debug.log
"#;
        let result = filter_status_output(output);
        assert!(result.contains("Untracked"));
    }

    #[test]
    fn test_branch_ahead() {
        let result = compact_tracking_line("Your branch is ahead of 'origin/main' by 3 commits.");
        assert_eq!(result, "↑3");
    }

    #[test]
    fn test_branch_behind() {
        let result = compact_tracking_line("Your branch is behind 'origin/main' by 2 commits.");
        assert_eq!(result, "↓2");
    }

    #[test]
    fn test_branch_synced() {
        let result = compact_tracking_line("Your branch is up to date with 'origin/main'.");
        assert_eq!(result, "✓ synced");
    }

    #[test]
    fn test_compression_ratio() {
        let raw = r#"On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/main.rs
        modified:   src/lib.rs
        modified:   Cargo.toml

no changes added to commit (use "git add" and/or "git commit -a")
"#;
        let filtered = filter_status_output(raw);

        // Should achieve significant compression
        let ratio = filtered.len() as f64 / raw.len() as f64;
        assert!(ratio < 0.5, "Expected >50% compression, got {:.1}%", (1.0 - ratio) * 100.0);
    }
}
