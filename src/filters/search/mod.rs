//! Search command filters (grep, rg/ripgrep).

use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

pub struct GrepFilter;

impl Filter for GrepFilter {
    fn name(&self) -> &'static str {
        "grep"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        matches!(cmd.as_str(), "grep" | "grep.exe" | "rg" | "rg.exe" | "ripgrep" | "ripgrep.exe")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let filtered = filter_grep_output(&stdout, &stderr);
        let raw = format!("{}{}", stdout, stderr);
        Ok(FilterResult::with_raw(filtered, raw, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        75
    }
}

fn parse_grep_line(line: &str) -> Option<(String, String)> {
    if line.is_empty() {
        return None;
    }

    // Handle Windows drive letter prefix (C:/path or C:\path)
    let (prefix_len, rest) = {
        let mut chars = line.chars();
        let first = chars.next()?;
        let second = chars.next().unwrap_or('\0');
        if first.is_ascii_alphabetic() && second == ':' {
            (2usize, &line[2..])
        } else {
            (0usize, line)
        }
    };

    let colon_pos = rest.find(':')?;
    let file = line[..prefix_len + colon_pos].to_string();
    let after_file = &rest[colon_pos + 1..];

    // Check for file:linenum:content format
    if let Some(second_colon) = after_file.find(':') {
        if after_file[..second_colon].trim().parse::<u32>().is_ok() {
            let content = after_file[second_colon + 1..].trim().to_string();
            return Some((file, content));
        }
    }

    Some((file, after_file.trim().to_string()))
}

fn filter_grep_output(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        let err = stderr.trim();
        if !err.is_empty() {
            return format!("error: {}", err.lines().next().unwrap_or(err));
        }
        return "No matches".to_string();
    }

    let mut by_file: HashMap<String, Vec<String>> = HashMap::new();
    let mut plain_lines: Vec<String> = Vec::new();

    for line in &lines {
        if let Some((file, content)) = parse_grep_line(line) {
            by_file.entry(file).or_default().push(content);
        } else {
            plain_lines.push(line.to_string());
        }
    }

    // If no file grouping found, show plain truncated output
    if by_file.is_empty() {
        let total = plain_lines.len();
        let mut result = vec![format!("{} matches", total)];
        for line in plain_lines.iter().take(20) {
            result.push(format!("  {}", truncate(line, 120)));
        }
        if total > 20 {
            result.push(format!("  ... +{} more", total - 20));
        }
        return result.join("\n");
    }

    let total_matches: usize = by_file.values().map(|v| v.len()).sum();
    let mut result = vec![format!(
        "{} matches in {} file{}",
        total_matches,
        by_file.len(),
        if by_file.len() == 1 { "" } else { "s" }
    )];

    let mut files: Vec<(&String, &Vec<String>)> = by_file.iter().collect();
    files.sort_by(|a, b| a.0.cmp(b.0));

    for (file, matches) in files.iter().take(15) {
        result.push(format!(
            "{}  ({} match{})",
            shorten_path(file),
            matches.len(),
            if matches.len() == 1 { "" } else { "es" }
        ));
        for m in matches.iter().take(3) {
            result.push(format!("  {}", truncate(m, 100)));
        }
        if matches.len() > 3 {
            result.push(format!("  ... +{} more", matches.len() - 3));
        }
    }

    if files.len() > 15 {
        result.push(format!("... +{} more files", files.len() - 15));
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let (file, content) = parse_grep_line("src/main.rs:fn main()").unwrap();
        assert_eq!(file, "src/main.rs");
        assert_eq!(content, "fn main()");
    }

    #[test]
    fn test_parse_with_linenum() {
        let (file, content) = parse_grep_line("src/main.rs:42:fn main()").unwrap();
        assert_eq!(file, "src/main.rs");
        assert_eq!(content, "fn main()");
    }

    #[test]
    fn test_parse_windows_path() {
        let (file, content) = parse_grep_line("C:/Users/src/main.rs:42:fn main()").unwrap();
        assert_eq!(file, "C:/Users/src/main.rs");
        assert_eq!(content, "fn main()");
    }

    #[test]
    fn test_no_match() {
        let result = filter_grep_output("", "");
        assert_eq!(result, "No matches");
    }

    #[test]
    fn test_groups_by_file() {
        let stdout = "src/a.rs:1:foo\nsrc/a.rs:2:bar\nsrc/b.rs:1:baz\n";
        let result = filter_grep_output(stdout, "");
        assert!(result.contains("3 matches in 2 files"));
        assert!(result.contains("src/a.rs"));
        assert!(result.contains("src/b.rs"));
    }
}
