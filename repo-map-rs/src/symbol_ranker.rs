use crate::tree_sitter_utils::{Symbol, SymbolKind};
use std::collections::HashMap;
use std::path::Path;

/// Ranks symbols by importance for inclusion in the repo map
pub struct SymbolRanker {
    /// Weight for symbol kind priority
    kind_weight: f64,
    /// Weight for reference count
    reference_weight: f64,
    /// Weight for being in important files
    file_weight: f64,
}

impl Default for SymbolRanker {
    fn default() -> Self {
        Self {
            kind_weight: 1.0,
            reference_weight: 2.0,
            file_weight: 0.5,
        }
    }
}

impl SymbolRanker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rank symbols by importance, returning them sorted by score (highest first)
    pub fn rank_symbols(&self, symbols: &[Symbol]) -> Vec<RankedSymbol> {
        // Count references to each symbol
        let reference_counts = self.count_references(symbols);
        
        // Calculate scores for each symbol
        let mut ranked_symbols: Vec<RankedSymbol> = symbols
            .iter()
            .filter(|symbol| self.should_include_symbol(symbol))
            .map(|symbol| {
                let score = self.calculate_score(symbol, &reference_counts);
                RankedSymbol {
                    symbol: symbol.clone(),
                    score,
                }
            })
            .collect();

        // Sort by score (highest first)
        ranked_symbols.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Remove duplicates (keep highest scored version)
        self.deduplicate_symbols(ranked_symbols)
    }

    /// Count how many times each symbol is referenced
    fn count_references(&self, symbols: &[Symbol]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for symbol in symbols {
            if symbol.kind == SymbolKind::Call {
                *counts.entry(symbol.name.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }

    /// Calculate the importance score for a symbol
    fn calculate_score(&self, symbol: &Symbol, reference_counts: &HashMap<String, usize>) -> f64 {
        let mut score = 0.0;

        // Base score from symbol kind priority
        score += symbol.kind.priority() as f64 * self.kind_weight;

        // Bonus for being referenced by other code
        if let Some(&ref_count) = reference_counts.get(&symbol.name) {
            score += ref_count as f64 * self.reference_weight;
        }

        // Bonus for being in important files
        if self.is_important_file(&symbol.file_path) {
            score += 50.0 * self.file_weight;
        }

        // Penalty for very long names (likely generated or internal)
        if symbol.name.len() > 50 {
            score *= 0.5;
        }

        // Bonus for common important patterns
        if self.is_important_symbol_name(&symbol.name) {
            score += 20.0;
        }

        score
    }

    /// Check if a symbol should be included in the ranking
    fn should_include_symbol(&self, symbol: &Symbol) -> bool {
        // Skip call references for now (they're used for counting but not displayed)
        if symbol.kind == SymbolKind::Call {
            return false;
        }

        // Skip very short names (likely not meaningful)
        if symbol.name.len() < 2 {
            return false;
        }

        // Skip symbols that look like internal/generated code
        if symbol.name.starts_with('_') && symbol.name.len() > 2 && symbol.name.chars().nth(1) == Some('_') {
            return false;
        }

        true
    }

    /// Check if a file is considered important (main files, lib files, etc.)
    fn is_important_file(&self, file_path: &Path) -> bool {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            matches!(file_name,
                // Rust
                "main.rs" | "lib.rs" | "mod.rs" |
                // Python
                "main.py" | "__init__.py" | "setup.py" |
                // JavaScript/TypeScript
                "index.js" | "main.js" | "app.js" | "index.ts" | "main.ts" |
                // Java
                "Main.java" | "Application.java" | "App.java" |
                // Kotlin
                "Main.kt" | "Application.kt" | "App.kt" |
                // Go
                "main.go" |
                // PHP
                "index.php" | "main.php" | "app.php" |
                // C/C++
                "main.c" | "main.cpp" | "main.cc" |
                // C#
                "Program.cs" | "Main.cs" | "Application.cs" |
                // Ruby
                "main.rb" | "application.rb" | "app.rb" |
                // Swift
                "main.swift" | "AppDelegate.swift" |
                // Scala
                "Main.scala" | "Application.scala"
            )
        } else {
            false
        }
    }

    /// Check if a symbol name indicates it's important
    fn is_important_symbol_name(&self, name: &str) -> bool {
        matches!(name.to_lowercase().as_str(),
            "main" | "new" | "init" | "create" | "build" | "run" | "execute" |
            "get" | "set" | "add" | "remove" | "update" | "delete" |
            "parse" | "render" | "process" | "handle"
        )
    }

    /// Remove duplicate symbols, keeping the highest scored version
    fn deduplicate_symbols(&self, mut ranked_symbols: Vec<RankedSymbol>) -> Vec<RankedSymbol> {
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        for ranked_symbol in ranked_symbols.drain(..) {
            let key = format!("{}:{}", ranked_symbol.symbol.name, ranked_symbol.symbol.file_path.display());
            
            if let Some(existing_score) = seen.get(&key) {
                if ranked_symbol.score > *existing_score {
                    // Replace with higher scored version
                    let key_clone = key.clone();
                    seen.insert(key, ranked_symbol.score);
                    // Remove the old one and add the new one
                    result.retain(|rs: &RankedSymbol| {
                        let rs_key = format!("{}:{}", rs.symbol.name, rs.symbol.file_path.display());
                        rs_key != key_clone
                    });
                    result.push(ranked_symbol);
                }
            } else {
                seen.insert(key, ranked_symbol.score);
                result.push(ranked_symbol);
            }
        }

        // Re-sort after deduplication
        result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}

/// A symbol with its calculated importance score
#[derive(Debug, Clone)]
pub struct RankedSymbol {
    pub symbol: Symbol,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_symbol(name: &str, kind: SymbolKind, file: &str, line: usize) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            line,
            file_path: PathBuf::from(file),
        }
    }

    #[test]
    fn test_symbol_ranking() {
        let symbols = vec![
            create_test_symbol("MyClass", SymbolKind::Class, "main.rs", 1),
            create_test_symbol("helper_func", SymbolKind::Function, "utils.rs", 10),
            create_test_symbol("main", SymbolKind::Function, "main.rs", 50),
            create_test_symbol("MyClass", SymbolKind::Call, "other.rs", 20), // Reference to MyClass
        ];

        let ranker = SymbolRanker::new();
        let ranked = ranker.rank_symbols(&symbols);

        // Should have 3 symbols (excluding the call)
        assert_eq!(ranked.len(), 3);

        // main function should score high (important name + important file)
        let main_symbol = ranked.iter().find(|rs| rs.symbol.name == "main").unwrap();
        assert!(main_symbol.score > 100.0);

        // MyClass should score high (class + referenced + important file)
        let class_symbol = ranked.iter().find(|rs| rs.symbol.name == "MyClass").unwrap();
        assert!(class_symbol.score > 100.0);
    }

    #[test]
    fn test_reference_counting() {
        let symbols = vec![
            create_test_symbol("func1", SymbolKind::Function, "main.rs", 1),
            create_test_symbol("func1", SymbolKind::Call, "other.rs", 10),
            create_test_symbol("func1", SymbolKind::Call, "other.rs", 20),
            create_test_symbol("func2", SymbolKind::Function, "main.rs", 30),
        ];

        let ranker = SymbolRanker::new();
        let ref_counts = ranker.count_references(&symbols);

        assert_eq!(ref_counts.get("func1"), Some(&2));
        assert_eq!(ref_counts.get("func2"), None);
    }

    #[test]
    fn test_important_file_detection() {
        let ranker = SymbolRanker::new();
        
        assert!(ranker.is_important_file(&PathBuf::from("main.rs")));
        assert!(ranker.is_important_file(&PathBuf::from("src/main.rs")));
        assert!(ranker.is_important_file(&PathBuf::from("lib.rs")));
        assert!(ranker.is_important_file(&PathBuf::from("main.py")));
        assert!(!ranker.is_important_file(&PathBuf::from("utils.rs")));
        assert!(!ranker.is_important_file(&PathBuf::from("helper.py")));
    }

    #[test]
    fn test_important_symbol_names() {
        let ranker = SymbolRanker::new();
        
        assert!(ranker.is_important_symbol_name("main"));
        assert!(ranker.is_important_symbol_name("new"));
        assert!(ranker.is_important_symbol_name("init"));
        assert!(!ranker.is_important_symbol_name("random_helper"));
        assert!(!ranker.is_important_symbol_name("xyz"));
    }
}
