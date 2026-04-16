//! Git diff filter.

use regex::Regex;

/// Filter git diff output to compact form.
pub fn filter_diff_output(output: &str) -> String {
    let file_re = Regex::new(r"^diff --git a/(.+) b/(.+)$").unwrap();
    let hunk_re = Regex::new(r"^@@ -(\d+),?\d* \+(\d+),?\d* @@(.*)$").unwrap();

    let mut result = Vec::new();
    let mut current_file = String::new();
    let mut additions = 0;
    let mut deletions = 0;
    let mut changes: Vec<String> = Vec::new();

    for line in output.lines() {
        if let Some(caps) = file_re.captures(line) {
            // Save previous file stats
            if !current_file.is_empty() {
                result.push(format_file_diff(&current_file, additions, deletions, &changes));
            }
            current_file = caps[2].to_string();
            additions = 0;
            deletions = 0;
            changes.clear();
        } else if let Some(caps) = hunk_re.captures(line) {
            let context = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            if !context.is_empty() {
                changes.push(format!("  @{}: {}", &caps[2], context.trim()));
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
        }
    }

    // Don't forget the last file
    if !current_file.is_empty() {
        result.push(format_file_diff(&current_file, additions, deletions, &changes));
    }

    if result.is_empty() {
        "No changes".to_string()
    } else {
        result.join("\n")
    }
}

fn format_file_diff(file: &str, additions: usize, deletions: usize, changes: &[String]) -> String {
    let mut lines = vec![format!(
        "{} +{} -{}",
        truncate_path(file),
        additions,
        deletions
    )];

    // Add first few change contexts
    for change in changes.iter().take(3) {
        lines.push(change.clone());
    }
    if changes.len() > 3 {
        lines.push(format!("  ... +{} more hunks", changes.len() - 3));
    }

    lines.join("\n")
}

fn truncate_path(path: &str) -> String {
    if path.len() <= 50 {
        path.to_string()
    } else {
        // Keep filename and truncate directories
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 2 {
            format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            format!("...{}", &path[path.len() - 47..])
        }
    }
}
