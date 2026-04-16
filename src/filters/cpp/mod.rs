//! C/C++ build system filters - make, cmake, gcc, g++, clang, ninja

use std::process::Command;
use regex::Regex;
use anyhow::Result;
use super::traits::{Filter, FilterResult};

// ============================================================================
// Make Filter
// ============================================================================

pub struct MakeFilter;

impl Filter for MakeFilter {
    fn name(&self) -> &'static str {
        "make"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "make" || cmd == "gmake" || cmd == "mingw32-make"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_make_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_make_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut compiled = Vec::new();
    let mut linked = Vec::new();

    let error_re = Regex::new(r"error[:\s]|Error[:\s]|undefined reference|fatal error").unwrap();
    let warning_re = Regex::new(r"warning[:\s]|Warning[:\s]").unwrap();
    let compile_re = Regex::new(r"(?:Compiling|Building|CC|CXX|gcc|g\+\+|clang)\s+(\S+)").unwrap();
    let link_re = Regex::new(r"(?:Linking|LD|AR)\s+(\S+)").unwrap();

    for line in combined.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Capture errors (high priority)
        if error_re.is_match(trimmed) {
            errors.push(trimmed.to_string());
        }
        // Capture warnings
        else if warning_re.is_match(trimmed) {
            warnings.push(trimmed.to_string());
        }
        // Track compiled files
        else if let Some(caps) = compile_re.captures(trimmed) {
            if let Some(file) = caps.get(1) {
                compiled.push(file.as_str().to_string());
            }
        }
        // Track linked outputs
        else if let Some(caps) = link_re.captures(trimmed) {
            if let Some(file) = caps.get(1) {
                linked.push(file.as_str().to_string());
            }
        }
        // Keep "make: Nothing to be done" type messages
        else if trimmed.starts_with("make:") || trimmed.starts_with("make[") {
            if !trimmed.contains("Entering") && !trimmed.contains("Leaving") {
                errors.push(trimmed.to_string()); // Not really errors but important messages
            }
        }
    }

    let mut result = Vec::new();

    if !compiled.is_empty() {
        result.push(format!("Compiled({}): {}", compiled.len(),
            if compiled.len() <= 10 { compiled.join(", ") }
            else { format!("{}, ... +{} more", compiled[..5].join(", "), compiled.len() - 5) }
        ));
    }

    if !linked.is_empty() {
        result.push(format!("Linked: {}", linked.join(", ")));
    }

    if !warnings.is_empty() {
        result.push(format!("Warnings({}):", warnings.len()));
        result.extend(warnings.into_iter().take(10));
    }

    if !errors.is_empty() {
        result.push("Errors:".to_string());
        result.extend(errors.into_iter().take(20));
    }

    if result.is_empty() {
        "Build complete".to_string()
    } else {
        result.join("\n")
    }
}

// ============================================================================
// CMake Filter
// ============================================================================

pub struct CmakeFilter;

impl Filter for CmakeFilter {
    fn name(&self) -> &'static str {
        "cmake"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "cmake" || cmd == "cmake.exe"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new("cmake")
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        // Determine CMake mode based on args
        let is_build = args.iter().any(|a| a == "--build");
        let is_install = args.iter().any(|a| a == "--install");

        let filtered = if is_build {
            filter_cmake_build(&stdout, &stderr)
        } else if is_install {
            filter_cmake_install(&stdout, &stderr)
        } else {
            filter_cmake_configure(&stdout, &stderr)
        };

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_cmake_configure(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut result = Vec::new();
    let mut found_items = Vec::new();
    let mut errors = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Capture configuration results
        if trimmed.starts_with("-- Found") {
            if let Some(pkg) = trimmed.strip_prefix("-- Found ") {
                let name = pkg.split(':').next().unwrap_or(pkg).trim();
                found_items.push(name.to_string());
            }
        }
        // Capture errors
        else if trimmed.contains("CMake Error") || trimmed.contains("error:") {
            errors.push(trimmed.to_string());
        }
        // Capture warnings (but limit)
        else if trimmed.contains("CMake Warning") {
            result.push(trimmed.to_string());
        }
        // Capture generator info
        else if trimmed.contains("Build files have been written") {
            result.push(trimmed.to_string());
        }
        // Capture selected compiler
        else if trimmed.contains("The C compiler") || trimmed.contains("The CXX compiler") {
            result.push(trimmed.replace("-- ", ""));
        }
    }

    let mut output = Vec::new();

    if !found_items.is_empty() {
        output.push(format!("Found({}): {}", found_items.len(), found_items.join(", ")));
    }

    output.extend(result.into_iter().take(5));

    if !errors.is_empty() {
        output.push("Errors:".to_string());
        output.extend(errors.into_iter().take(10));
    }

    if output.is_empty() {
        "Configuration complete".to_string()
    } else {
        output.join("\n")
    }
}

fn filter_cmake_build(stdout: &str, stderr: &str) -> String {
    // CMake build often calls make/ninja, use similar filtering
    filter_make_output(stdout, stderr)
}

fn filter_cmake_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut installed = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-- Installing:") || trimmed.starts_with("-- Up-to-date:") {
            if let Some(path) = trimmed.split(':').nth(1) {
                let file = path.trim().rsplit(['/', '\\']).next().unwrap_or(path.trim());
                installed.push(file.to_string());
            }
        }
    }

    if installed.is_empty() {
        "Install complete".to_string()
    } else {
        format!("Installed({}): {}", installed.len(),
            if installed.len() <= 10 { installed.join(", ") }
            else { format!("{}, ... +{} more", installed[..5].join(", "), installed.len() - 5) }
        )
    }
}

// ============================================================================
// GCC Filter
// ============================================================================

pub struct GccFilter;

impl Filter for GccFilter {
    fn name(&self) -> &'static str {
        "gcc"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "gcc" || cmd == "gcc.exe" || cmd == "cc"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_compiler_output(&stdout, &stderr, "gcc");

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// ============================================================================
// G++ Filter
// ============================================================================

pub struct GppFilter;

impl Filter for GppFilter {
    fn name(&self) -> &'static str {
        "g++"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "g++" || cmd == "g++.exe" || cmd == "c++"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_compiler_output(&stdout, &stderr, "g++");

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// ============================================================================
// Clang Filter
// ============================================================================

pub struct ClangFilter;

impl Filter for ClangFilter {
    fn name(&self) -> &'static str {
        "clang"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "clang" || cmd == "clang++" || cmd == "clang.exe" || cmd == "clang++.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_compiler_output(&stdout, &stderr, "clang");

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Shared compiler output filter for gcc/g++/clang
fn filter_compiler_output(stdout: &str, stderr: &str, _compiler: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);

    // If no output at all, compilation succeeded
    if combined.trim().is_empty() {
        return "Compiled successfully".to_string();
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut notes = Vec::new();
    let mut current_file = String::new();

    // Regex for compiler messages: file:line:col: type: message
    let msg_re = Regex::new(r"^([^:]+):(\d+):(\d+):\s*(error|warning|note):\s*(.+)$").unwrap();
    let simple_error_re = Regex::new(r"^([^:]+):\s*(fatal error|error):\s*(.+)$").unwrap();

    for line in combined.lines() {
        let trimmed = line.trim();

        if let Some(caps) = msg_re.captures(trimmed) {
            let file = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let line_num = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let msg_type = caps.get(4).map(|m| m.as_str()).unwrap_or("");
            let message = caps.get(5).map(|m| m.as_str()).unwrap_or("");

            // Extract just filename
            let filename = file.rsplit(['/', '\\']).next().unwrap_or(file);

            if filename != current_file {
                current_file = filename.to_string();
            }

            let formatted = format!("{}:{}: {}", filename, line_num, message);

            match msg_type {
                "error" => errors.push(formatted),
                "warning" => warnings.push(formatted),
                "note" => notes.push(formatted),
                _ => {}
            }
        } else if let Some(caps) = simple_error_re.captures(trimmed) {
            let file = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let message = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let filename = file.rsplit(['/', '\\']).next().unwrap_or(file);
            errors.push(format!("{}: {}", filename, message));
        }
        // Capture linker errors
        else if trimmed.contains("undefined reference") || trimmed.contains("ld returned") {
            errors.push(trimmed.to_string());
        }
    }

    let mut result = Vec::new();

    if !errors.is_empty() {
        result.push(format!("Errors({}):", errors.len()));
        result.extend(errors.into_iter().take(15));
    }

    if !warnings.is_empty() {
        result.push(format!("Warnings({}):", warnings.len()));
        result.extend(warnings.into_iter().take(10));
    }

    // Only include first few notes
    if !notes.is_empty() && result.len() < 20 {
        result.extend(notes.into_iter().take(3));
    }

    if result.is_empty() {
        "Compiled successfully".to_string()
    } else {
        result.join("\n")
    }
}

// ============================================================================
// Ninja Filter
// ============================================================================

pub struct NinjaFilter;

impl Filter for NinjaFilter {
    fn name(&self) -> &'static str {
        "ninja"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "ninja" || cmd == "ninja.exe"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new("ninja")
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_ninja_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_ninja_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut built_count = 0;

    // Ninja progress format: [X/Y] action
    let progress_re = Regex::new(r"^\[(\d+)/(\d+)\]").unwrap();

    for line in combined.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Count build steps
        if let Some(caps) = progress_re.captures(trimmed) {
            if let (Some(current), Some(total)) = (caps.get(1), caps.get(2)) {
                let c: usize = current.as_str().parse().unwrap_or(0);
                let t: usize = total.as_str().parse().unwrap_or(0);
                if c == t {
                    built_count = t;
                }
            }
        }
        // Capture errors
        else if trimmed.contains("error:") || trimmed.contains("FAILED:") {
            errors.push(trimmed.to_string());
        }
        // Capture warnings
        else if trimmed.contains("warning:") {
            warnings.push(trimmed.to_string());
        }
        // Ninja-specific messages
        else if trimmed.starts_with("ninja:") {
            if trimmed.contains("no work") {
                return "Nothing to do".to_string();
            }
            errors.push(trimmed.to_string());
        }
    }

    let mut result = Vec::new();

    if built_count > 0 {
        result.push(format!("Built {} targets", built_count));
    }

    if !errors.is_empty() {
        result.push(format!("Errors({}):", errors.len()));
        result.extend(errors.into_iter().take(15));
    }

    if !warnings.is_empty() {
        result.push(format!("Warnings({}):", warnings.len()));
        result.extend(warnings.into_iter().take(10));
    }

    if result.is_empty() {
        "Build complete".to_string()
    } else {
        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_filter_matches() {
        let filter = MakeFilter;
        assert!(filter.matches("make"));
        assert!(filter.matches("gmake"));
        assert!(filter.matches("mingw32-make"));
        assert!(!filter.matches("cmake"));
    }

    #[test]
    fn test_cmake_filter_matches() {
        let filter = CmakeFilter;
        assert!(filter.matches("cmake"));
        assert!(!filter.matches("make"));
    }

    #[test]
    fn test_gcc_filter_matches() {
        let filter = GccFilter;
        assert!(filter.matches("gcc"));
        assert!(filter.matches("cc"));
        assert!(!filter.matches("g++"));
    }

    #[test]
    fn test_clang_filter_matches() {
        let filter = ClangFilter;
        assert!(filter.matches("clang"));
        assert!(filter.matches("clang++"));
    }

    #[test]
    fn test_ninja_filter_matches() {
        let filter = NinjaFilter;
        assert!(filter.matches("ninja"));
        assert!(!filter.matches("make"));
    }

    #[test]
    fn test_compiler_output_filtering() {
        let stderr = r#"
main.c:10:5: error: expected ';' after expression
main.c:15:10: warning: unused variable 'x'
main.c:20:5: note: declared here
"#;
        let result = filter_compiler_output("", stderr, "gcc");
        assert!(result.contains("Errors(1)"));
        assert!(result.contains("Warnings(1)"));
    }
}
