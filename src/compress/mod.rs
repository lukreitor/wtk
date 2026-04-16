//! Compression algorithms for token optimization.

/// Group similar items together.
pub fn group_by_key<T, K, F>(items: &[T], key_fn: F) -> Vec<(&K, Vec<&T>)>
where
    K: Eq + std::hash::Hash,
    F: Fn(&T) -> &K,
{
    use std::collections::HashMap;

    let mut groups: HashMap<&K, Vec<&T>> = HashMap::new();
    for item in items {
        let key = key_fn(item);
        groups.entry(key).or_default().push(item);
    }

    groups.into_iter().collect()
}

/// Truncate a string to a maximum length.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Deduplicate consecutive lines, adding counts.
pub fn dedup_with_counts(lines: &[&str]) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current = lines[0];
    let mut count = 1;

    for line in &lines[1..] {
        if *line == current {
            count += 1;
        } else {
            if count > 1 {
                result.push(format!("{} (x{})", current, count));
            } else {
                result.push(current.to_string());
            }
            current = line;
            count = 1;
        }
    }

    // Don't forget the last group
    if count > 1 {
        result.push(format!("{} (x{})", current, count));
    } else {
        result.push(current.to_string());
    }

    result
}

/// Remove common boilerplate patterns.
pub fn remove_boilerplate(s: &str) -> String {
    let patterns = [
        // npm
        "npm WARN",
        "npm notice",
        "added",
        "removed",
        "packages in",
        // git
        "hint:",
        "warning:",
        // general
        "...",
    ];

    s.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !patterns.iter().any(|p| trimmed.starts_with(p))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a number with K/M suffixes.
pub fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");
    }

    #[test]
    fn test_dedup() {
        let lines = vec!["a", "a", "a", "b", "b", "c"];
        let result = dedup_with_counts(&lines);
        assert_eq!(result, vec!["a (x3)", "b (x2)", "c"]);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1500000), "1.5M");
    }
}
