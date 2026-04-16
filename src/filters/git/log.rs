//! Git log filter.

use regex::Regex;

/// Filter git log output to compact form.
pub fn filter_log_output(output: &str) -> String {
    let commit_re = Regex::new(r"^commit ([a-f0-9]{40})").unwrap();
    let date_re = Regex::new(r"^Date:\s+(.+)$").unwrap();

    let mut result = Vec::new();
    let mut current_hash = String::new();
    let mut current_date = String::new();
    let mut current_message = String::new();

    for line in output.lines() {
        if let Some(caps) = commit_re.captures(line) {
            // Save previous commit if exists
            if !current_hash.is_empty() {
                result.push(format_commit(&current_hash, &current_date, &current_message));
            }
            current_hash = caps[1][..7].to_string(); // Short hash
            current_date.clear();
            current_message.clear();
        } else if let Some(caps) = date_re.captures(line) {
            current_date = format_date(&caps[1]);
        } else if line.starts_with("    ") && current_message.is_empty() {
            // First line of commit message
            current_message = line.trim().to_string();
        }
        // Skip Author line and other metadata
    }

    // Don't forget the last commit
    if !current_hash.is_empty() {
        result.push(format_commit(&current_hash, &current_date, &current_message));
    }

    result.join("\n")
}

fn format_commit(hash: &str, date: &str, message: &str) -> String {
    let truncated_msg = if message.len() > 60 {
        format!("{}...", &message[..57])
    } else {
        message.to_string()
    };

    format!("{} {} {}", hash, date, truncated_msg)
}

fn format_date(date_str: &str) -> String {
    // Parse and format date compactly
    // Input: "Mon Apr 14 10:30:00 2025 -0300"
    // Output: "Apr14"

    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() >= 3 {
        format!("{}{}", parts[1], parts[2])
    } else {
        date_str.to_string()
    }
}
