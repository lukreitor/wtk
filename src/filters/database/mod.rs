//! Database CLI filters (psql, mysql, sqlcmd, redis-cli, mongosh).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for PostgreSQL psql commands.
pub struct PsqlFilter;

impl Filter for PsqlFilter {
    fn name(&self) -> &'static str {
        "psql"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "psql" || cmd == "psql.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_psql(&stdout, &stderr, args);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for MySQL commands.
pub struct MysqlFilter;

impl Filter for MysqlFilter {
    fn name(&self) -> &'static str {
        "mysql"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "mysql" || cmd == "mysql.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_mysql(&stdout, &stderr, args);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for SQL Server sqlcmd commands.
pub struct SqlcmdFilter;

impl Filter for SqlcmdFilter {
    fn name(&self) -> &'static str {
        "sqlcmd"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "sqlcmd" || cmd == "sqlcmd.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_sqlcmd(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Redis CLI commands.
pub struct RedisCliFilter;

impl Filter for RedisCliFilter {
    fn name(&self) -> &'static str {
        "redis-cli"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "redis-cli" || cmd == "redis-cli.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_redis(&stdout, &stderr, args);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for MongoDB shell commands.
pub struct MongoshFilter;

impl Filter for MongoshFilter {
    fn name(&self) -> &'static str {
        "mongosh"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "mongosh" || cmd == "mongosh.exe" || cmd == "mongo" || cmd == "mongo.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_mongosh(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_psql(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Check for common operations
    if args.iter().any(|a| a == "-l" || a == "--list") {
        // List databases
        let lines: Vec<&str> = stdout.lines()
            .filter(|l| !l.trim().is_empty() && !l.contains("---") && !l.starts_with(" Name"))
            .collect();

        let db_count = lines.iter().filter(|l| l.contains('|')).count();
        if db_count > 0 {
            return format!("{} databases", db_count);
        }
    }

    // Check for query results
    if combined.contains(" row") || combined.contains(" rows") {
        let row_re = Regex::new(r"\((\d+) rows?\)").unwrap();
        if let Some(caps) = row_re.captures(&combined) {
            return format!("{} rows returned", &caps[1]);
        }
    }

    // Check for INSERT/UPDATE/DELETE
    if let Some(affected) = extract_affected_rows(&combined) {
        return affected;
    }

    // Check for errors
    if combined.contains("ERROR:") {
        let error_re = Regex::new(r"ERROR:\s*(.+)").unwrap();
        if let Some(caps) = error_re.captures(&combined) {
            return format!("✗ {}", truncate(&caps[1], 60));
        }
    }

    filter_query_result(stdout, stderr)
}

fn filter_mysql(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Show databases
    if args.iter().any(|a| a.to_lowercase().contains("show databases")) {
        let lines: Vec<&str> = stdout.lines()
            .filter(|l| !l.trim().is_empty() && !l.contains("Database") && !l.contains("+--"))
            .collect();

        return format!("{} databases", lines.len());
    }

    // Check for query results
    if combined.contains(" row in set") || combined.contains(" rows in set") {
        let row_re = Regex::new(r"(\d+) rows? in set").unwrap();
        if let Some(caps) = row_re.captures(&combined) {
            return format!("{} rows returned", &caps[1]);
        }
    }

    // Check for affected rows
    if combined.contains("Query OK") {
        let affected_re = Regex::new(r"Query OK, (\d+) rows? affected").unwrap();
        if let Some(caps) = affected_re.captures(&combined) {
            return format!("✓ {} rows affected", &caps[1]);
        }
        return "✓ Query OK".to_string();
    }

    // Check for errors
    if combined.contains("ERROR") {
        let error_re = Regex::new(r"ERROR \d+.*?:\s*(.+)").unwrap();
        if let Some(caps) = error_re.captures(&combined) {
            return format!("✗ {}", truncate(&caps[1], 60));
        }
    }

    filter_query_result(stdout, stderr)
}

fn filter_sqlcmd(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Check for affected rows
    if combined.contains("rows affected") {
        let affected_re = Regex::new(r"\((\d+) rows? affected\)").unwrap();
        if let Some(caps) = affected_re.captures(&combined) {
            return format!("✓ {} rows affected", &caps[1]);
        }
    }

    // Check for errors
    if combined.contains("Msg ") && combined.contains("Level ") {
        let error_re = Regex::new(r"Msg \d+.*\n(.+)").unwrap();
        if let Some(caps) = error_re.captures(&combined) {
            return format!("✗ {}", truncate(&caps[1].trim(), 60));
        }
    }

    filter_query_result(stdout, stderr)
}

fn filter_redis(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let cmd = args.first().map(|s| s.to_uppercase()).unwrap_or_default();

    match cmd.as_str() {
        "PING" => {
            if stdout.trim() == "PONG" {
                return "✓ PONG".to_string();
            }
        }
        "SET" => {
            if stdout.trim() == "OK" {
                return "✓ SET OK".to_string();
            }
        }
        "GET" => {
            let value = stdout.trim();
            if value == "(nil)" {
                return "nil".to_string();
            }
            return truncate(value, 100);
        }
        "KEYS" => {
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() {
                return "0 keys".to_string();
            }
            let mut result = vec![format!("{} keys", lines.len())];
            for line in lines.iter().take(10) {
                result.push(format!("  {}", line.trim()));
            }
            if lines.len() > 10 {
                result.push(format!("  ... +{} more", lines.len() - 10));
            }
            return result.join("\n");
        }
        "INFO" => {
            // Extract key stats
            let version_re = Regex::new(r"redis_version:(.+)").unwrap();
            let memory_re = Regex::new(r"used_memory_human:(.+)").unwrap();
            let keys_re = Regex::new(r"keys=(\d+)").unwrap();

            let version = version_re.captures(&combined).map(|c| c[1].trim().to_string()).unwrap_or_default();
            let memory = memory_re.captures(&combined).map(|c| c[1].trim().to_string()).unwrap_or_default();
            let keys = keys_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_else(|| "0".to_string());

            let mut result = Vec::new();
            if !version.is_empty() {
                result.push(format!("Redis {}", version));
            }
            if !memory.is_empty() {
                result.push(format!("  Memory: {}", memory));
            }
            result.push(format!("  Keys: {}", keys));
            return result.join("\n");
        }
        "DBSIZE" => {
            let size_re = Regex::new(r"\(integer\) (\d+)").unwrap();
            if let Some(caps) = size_re.captures(&combined) {
                return format!("{} keys", &caps[1]);
            }
            return stdout.trim().to_string();
        }
        "FLUSHDB" | "FLUSHALL" => {
            if stdout.trim() == "OK" {
                return "✓ flushed".to_string();
            }
        }
        _ => {}
    }

    // Check for errors
    if combined.starts_with("(error)") || combined.contains("ERR ") {
        return format!("✗ {}", truncate(&combined.replace("(error)", "").trim(), 60));
    }

    filter_generic(stdout, stderr)
}

fn filter_mongosh(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ANSI codes and MongoDB shell decorations
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    // Check for insertions
    if cleaned.contains("acknowledged") && cleaned.contains("insertedId") {
        let count_re = Regex::new(r"insertedCount:\s*(\d+)").unwrap();
        if let Some(caps) = count_re.captures(&cleaned) {
            return format!("✓ inserted {} documents", &caps[1]);
        }
        return "✓ inserted".to_string();
    }

    // Check for updates
    if cleaned.contains("modifiedCount") {
        let modified_re = Regex::new(r"modifiedCount:\s*(\d+)").unwrap();
        if let Some(caps) = modified_re.captures(&cleaned) {
            return format!("✓ modified {} documents", &caps[1]);
        }
    }

    // Check for deletions
    if cleaned.contains("deletedCount") {
        let deleted_re = Regex::new(r"deletedCount:\s*(\d+)").unwrap();
        if let Some(caps) = deleted_re.captures(&cleaned) {
            return format!("✓ deleted {} documents", &caps[1]);
        }
    }

    // Check for find results (array of documents)
    if cleaned.contains("[") && cleaned.contains("_id") {
        let doc_count = cleaned.matches("_id").count();
        if doc_count > 0 {
            return format!("{} documents", doc_count);
        }
    }

    // Check for errors
    if cleaned.contains("MongoError") || cleaned.contains("MongoServerError") {
        let error_re = Regex::new(r"Mongo(?:Server)?Error:\s*(.+)").unwrap();
        if let Some(caps) = error_re.captures(&cleaned) {
            return format!("✗ {}", truncate(&caps[1], 60));
        }
    }

    filter_generic(stdout, stderr)
}

fn extract_affected_rows(output: &str) -> Option<String> {
    // PostgreSQL: INSERT 0 1, UPDATE 5, DELETE 3
    let pg_insert_re = Regex::new(r"INSERT \d+ (\d+)").unwrap();
    let pg_update_re = Regex::new(r"UPDATE (\d+)").unwrap();
    let pg_delete_re = Regex::new(r"DELETE (\d+)").unwrap();

    if let Some(caps) = pg_insert_re.captures(output) {
        return Some(format!("✓ inserted {} rows", &caps[1]));
    }
    if let Some(caps) = pg_update_re.captures(output) {
        return Some(format!("✓ updated {} rows", &caps[1]));
    }
    if let Some(caps) = pg_delete_re.captures(output) {
        return Some(format!("✓ deleted {} rows", &caps[1]));
    }

    None
}

fn filter_query_result(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Count rows in tabular output
    let lines: Vec<&str> = combined.lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.contains("---") && !l.contains("===") && !l.starts_with('+'))
        .collect();

    if lines.is_empty() {
        return "0 rows".to_string();
    }

    // Check if it looks like tabular data
    let has_separator = combined.contains('|') || combined.contains('\t');

    if has_separator && lines.len() > 1 {
        // First line is likely header
        let data_rows = lines.len() - 1;
        let mut result = vec![format!("{} rows", data_rows)];

        // Show first few rows
        for line in lines.iter().skip(1).take(5) {
            result.push(format!("  {}", truncate(line.trim(), 70)));
        }

        if data_rows > 5 {
            result.push(format!("  ... +{} more", data_rows - 5));
        }

        return result.join("\n");
    }

    filter_generic(stdout, stderr)
}

fn filter_generic(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr.to_string()
    } else if !stderr.is_empty() {
        format!("{}\n{}", stdout, stderr)
    } else {
        stdout.to_string()
    };

    let lines: Vec<&str> = combined.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 20 {
        let mut result: Vec<String> = lines.iter().take(15).map(|s| s.to_string()).collect();
        result.push(format!("... +{} more lines", lines.len() - 15));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
