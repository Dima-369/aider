use crate::symbol_ranker::RankedSymbol;
use crate::tree_sitter_utils::{Symbol, SymbolKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

/// Renders symbols into a tree-like repository map format
pub struct TreeRenderer {
    /// Maximum number of context lines to show around symbols
    context_lines: usize,
}

impl Default for TreeRenderer {
    fn default() -> Self {
        Self {
            context_lines: 2,
        }
    }
}

impl TreeRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render ranked symbols into a tree-like repository map
    pub fn render_tree(&self, ranked_symbols: &[RankedSymbol], max_symbols: usize) -> Result<String> {
        let symbols_to_render = &ranked_symbols[..max_symbols.min(ranked_symbols.len())];
        
        // Group symbols by file
        let mut files_map: HashMap<PathBuf, Vec<&RankedSymbol>> = HashMap::new();
        for ranked_symbol in symbols_to_render {
            files_map
                .entry(ranked_symbol.symbol.file_path.clone())
                .or_insert_with(Vec::new)
                .push(ranked_symbol);
        }

        let mut output = String::new();
        let mut sorted_files: Vec<_> = files_map.keys().collect();
        sorted_files.sort();

        for (i, file_path) in sorted_files.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            let symbols = &files_map[*file_path];
            output.push_str(&self.render_file(file_path, symbols)?);
        }

        Ok(output)
    }

    /// Render symbols for a single file
    fn render_file(&self, file_path: &Path, ranked_symbols: &[&RankedSymbol]) -> Result<String> {
        let mut output = String::new();
        
        // File header: strip leading ./ from display if present
        let mut display_path = file_path.display().to_string();
        if let Some(stripped) = display_path.strip_prefix("./") {
            display_path = stripped.to_string();
        }
        output.push_str(&format!("{}:\n", display_path));

        // Read file content for context
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Sort symbols by line number
        let mut symbols: Vec<_> = ranked_symbols.iter().map(|rs| &rs.symbol).collect();
        symbols.sort_by_key(|s| s.line);

        // Render each symbol with context
        for (i, symbol) in symbols.iter().enumerate() {
            if i > 0 {
                output.push_str("⋮\n");
            }

            output.push_str(&self.render_symbol_with_context(symbol, &lines)?);
        }

        Ok(output)
    }

    /// Render a single symbol with surrounding context
    fn render_symbol_with_context(&self, symbol: &Symbol, lines: &[&str]) -> Result<String> {
        let mut output = String::new();
        
        let symbol_line = symbol.line.saturating_sub(1); // Convert to 0-based
        
        // Determine context range
        let start_line = symbol_line.saturating_sub(self.context_lines);
        let end_line = (symbol_line + self.context_lines + 1).min(lines.len());

        // Find the actual symbol definition span
        let symbol_span = self.find_symbol_span(symbol, lines, symbol_line);

        for line_idx in start_line..end_line {
            if line_idx < lines.len() {
                let line_content = lines[line_idx];
                
                // Skip empty lines at the beginning and end of context
                if line_content.trim().is_empty() && 
                   (line_idx == start_line || line_idx == end_line - 1) {
                    continue;
                }

                // Add line prefix
                if line_idx >= symbol_span.0 && line_idx <= symbol_span.1 {
                    output.push('│');
                } else {
                    output.push(' ');
                }

                output.push_str(line_content);
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Find the span (start_line, end_line) of a symbol definition
    fn find_symbol_span(&self, symbol: &Symbol, lines: &[&str], symbol_line: usize) -> (usize, usize) {
        let start_line = symbol_line;
        let end_line;

        // For different symbol kinds, try to find the end of the definition
        match symbol.kind {
            SymbolKind::Class | SymbolKind::Interface => {
                // Find the end of the class/interface definition
                end_line = self.find_block_end(lines, symbol_line);
            }
            SymbolKind::Function | SymbolKind::Method => {
                // Find the end of the function signature (usually just the first line or until {)
                end_line = self.find_function_signature_end(lines, symbol_line);
            }
            _ => {
                // For other symbols, just use the single line
                end_line = symbol_line;
            }
        }

        (start_line, end_line)
    }

    /// Find the end of a block definition (for classes, interfaces, etc.)
    fn find_block_end(&self, lines: &[&str], start_line: usize) -> usize {
        // Simple heuristic: find the first line that starts with a closing brace or is empty
        for i in start_line..lines.len().min(start_line + 10) {
            let line = lines[i].trim();
            if line.ends_with('{') {
                return i;
            }
            if line.is_empty() && i > start_line {
                return i - 1;
            }
        }
        start_line
    }

    /// Find the end of a function signature
    fn find_function_signature_end(&self, lines: &[&str], start_line: usize) -> usize {
        // Look for the end of the function signature (usually ends with { or :)
        for i in start_line..lines.len().min(start_line + 5) {
            let line = lines[i].trim();
            if line.ends_with('{') || line.ends_with(':') || line.ends_with(';') {
                return i;
            }
        }
        start_line
    }
}

/// Simple token counter for estimating output size
pub struct TokenCounter;

impl TokenCounter {
    /// Estimate the number of tokens in a text string
    /// This is a simple approximation - in a real implementation you'd use tiktoken or similar
    pub fn count_tokens(text: &str) -> usize {
        // Simple heuristic: roughly 4 characters per token
        // This is very approximate but good enough for our purposes
        let char_count = text.chars().count();
        (char_count as f64 / 4.0).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_ranker::RankedSymbol;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_symbol(name: &str, kind: SymbolKind, file: &str, line: usize) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            line,
            file_path: PathBuf::from(file),
        }
    }

    fn create_ranked_symbol(symbol: Symbol, score: f64) -> RankedSymbol {
        RankedSymbol { symbol, score }
    }

    #[test]
    fn test_token_counting() {
        let text = "Hello world, this is a test string with some words.";
        let tokens = TokenCounter::count_tokens(text);
        
        // Should be roughly 13 tokens (52 chars / 4)
        assert!(tokens >= 10 && tokens <= 16);
    }

    #[test]
    fn test_render_tree() -> Result<()> {
        // Create a temporary file with test content
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "class TestClass:")?;
        writeln!(temp_file, "    def __init__(self):")?;
        writeln!(temp_file, "        pass")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "    def method(self):")?;
        writeln!(temp_file, "        return 42")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "def function():")?;
        writeln!(temp_file, "    print('hello')")?;

        let file_path = temp_file.path().to_path_buf();

        let ranked_symbols = vec![
            create_ranked_symbol(
                create_test_symbol("TestClass", SymbolKind::Class, &file_path.to_string_lossy(), 1),
                100.0
            ),
            create_ranked_symbol(
                create_test_symbol("method", SymbolKind::Method, &file_path.to_string_lossy(), 5),
                80.0
            ),
            create_ranked_symbol(
                create_test_symbol("function", SymbolKind::Function, &file_path.to_string_lossy(), 8),
                70.0
            ),
        ];

        let renderer = TreeRenderer::new();
        let output = renderer.render_tree(&ranked_symbols, 10)?;

        // Check that output contains expected elements
        assert!(output.contains("class TestClass:"));
        assert!(output.contains("def method(self):"));
        assert!(output.contains("def function():"));
        assert!(output.contains("│"));  // Should have tree characters
        assert!(output.contains("⋮"));  // Should have ellipsis between symbols

        Ok(())
    }

    #[test]
    fn test_find_symbol_span() {
        let lines = vec![
            "class MyClass:",
            "    def __init__(self):",
            "        pass",
            "",
            "    def method(self):",
            "        return 42",
        ];

        let renderer = TreeRenderer::new();
        
        // Test class span
        let class_symbol = create_test_symbol("MyClass", SymbolKind::Class, "test.py", 1);
        let span = renderer.find_symbol_span(&class_symbol, &lines, 0);
        // The class should span from line 0 to line 2 (until the empty line)
        assert_eq!(span, (0, 2));

        // Test method span
        let method_symbol = create_test_symbol("method", SymbolKind::Method, "test.py", 5);
        let span = renderer.find_symbol_span(&method_symbol, &lines, 4);
        assert_eq!(span, (4, 4)); // Just the method declaration line
    }

    #[test]
    fn test_render_file_header_strips_dot_slash() -> Result<()> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with content
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn foo() {{}}")?;

        // Build a ranked symbol pointing to the temp file
        let file_path = temp_file.path().to_path_buf();
        let symbol = create_test_symbol("foo", SymbolKind::Function, &file_path.to_string_lossy(), 1);
        let ranked_symbol = create_ranked_symbol(symbol, 1.0);

        let renderer = TreeRenderer::new();
        let output = renderer.render_tree(&[ranked_symbol], 1)?;

        // Ensure header does not start with ./ even if present
        let first_line = output.lines().next().unwrap_or("");
        assert!(!first_line.starts_with("./"), "Header should not start with ./, got: {}", first_line);
        assert!(first_line.ends_with(":"), "Header should end with colon, got: {}", first_line);

        Ok(())
    }
}
