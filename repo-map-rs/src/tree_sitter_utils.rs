use anyhow::{anyhow, Result};
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

/// Get the tree-sitter language for a given language name
pub fn get_language(language: &str) -> Result<Language> {
    match language {
        "rust" => Ok(tree_sitter_rust::LANGUAGE.into()),
        "python" => Ok(tree_sitter_python::LANGUAGE.into()),
        "javascript" => Ok(tree_sitter_javascript::LANGUAGE.into()),
        "typescript" => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Ok(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "go" => Ok(tree_sitter_go::LANGUAGE.into()),
        "swift" => Ok(tree_sitter_swift::LANGUAGE.into()),
        "java" => Ok(tree_sitter_java::LANGUAGE.into()),
        "php" => Ok(tree_sitter_php::language()),
        "lua" => Ok(tree_sitter_lua::LANGUAGE.into()),
        "kotlin" => Ok(tree_sitter_kotlin::language()),
        "c" => Ok(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Ok(tree_sitter_cpp::language()),
        "csharp" | "c#" => Ok(tree_sitter_c_sharp::language()),
        "ruby" => Ok(tree_sitter_ruby::language()),
        "scala" => Ok(tree_sitter_scala::LANGUAGE.into()),
        _ => Err(anyhow!("Unsupported language: {}", language)),
    }
}

/// Get the tree-sitter query for a given language
pub fn get_query(language: &str) -> Result<&'static str> {
    match language {
        "rust" => Ok(include_str!("queries/rust-tags.scm")),
        "python" => Ok(include_str!("queries/python-tags.scm")),
        "javascript" => Ok(include_str!("queries/javascript-tags.scm")),
        "typescript" | "tsx" => Ok(include_str!("queries/typescript-tags.scm")),
        "go" => Ok(include_str!("queries/go-tags.scm")),
        "swift" => Ok(include_str!("queries/swift-tags.scm")),
        "java" => Ok(include_str!("queries/java-tags.scm")),
        "php" => Ok(include_str!("queries/php-tags.scm")),
        "lua" => Ok(include_str!("queries/lua-tags.scm")),
        "kotlin" => Ok(include_str!("queries/kotlin-tags.scm")),
        "c" => Ok(include_str!("queries/c-tags.scm")),
        "cpp" => Ok(include_str!("queries/cpp-tags.scm")),
        "csharp" | "c#" => Ok(include_str!("queries/c_sharp-tags.scm")),
        "ruby" => Ok(include_str!("queries/ruby-tags.scm")),
        "scala" => Ok(include_str!("queries/scala-tags.scm")),
        _ => Err(anyhow!("No query available for language: {}", language)),
    }
}

/// Create a parser for a given language
pub fn create_parser(language: &str) -> Result<Parser> {
    let lang = get_language(language)?;
    let mut parser = Parser::new();
    parser.set_language(&lang).map_err(|e| anyhow!("Failed to set language: {}", e))?;
    Ok(parser)
}

/// Create a query for a given language
pub fn create_query(language: &str) -> Result<Query> {
    let lang = get_language(language)?;
    let query_str = get_query(language)?;
    Query::new(&lang, query_str).map_err(|e| anyhow!("Failed to create query: {}", e))
}

/// Parse a file and extract symbols using tree-sitter
pub fn parse_file(file_path: &Path, language: &str, content: &str) -> Result<Vec<Symbol>> {
    let mut parser = create_parser(language)?;
    let query = create_query(language)?;
    
    let tree = parser.parse(content, None)
        .ok_or_else(|| anyhow!("Failed to parse file: {}", file_path.display()))?;
    
    let mut cursor = QueryCursor::new();
    let captures = cursor.captures(&query, tree.root_node(), content.as_bytes());
    
    let mut symbols = Vec::new();
    
    for (m, _) in captures {
        let match_ = m;
        for capture in match_.captures {
            let node = capture.node;
            let capture_name = &query.capture_names()[capture.index as usize];
            
            // Extract symbol information
            let start_point = node.start_position();
            let line = start_point.row + 1; // Convert to 1-based line numbers
            let name = node.utf8_text(content.as_bytes())
                .unwrap_or("<unknown>")
                .to_string();
            
            // Determine symbol kind from capture name
            let kind = determine_symbol_kind(capture_name);
            
            symbols.push(Symbol {
                name,
                kind,
                line,
                file_path: file_path.to_path_buf(),
            });
        }
    }
    
    Ok(symbols)
}

/// Determine the symbol kind from the tree-sitter capture name
fn determine_symbol_kind(capture_name: &str) -> SymbolKind {
    if capture_name.contains("class") {
        SymbolKind::Class
    } else if capture_name.contains("function") {
        SymbolKind::Function
    } else if capture_name.contains("method") {
        SymbolKind::Method
    } else if capture_name.contains("interface") || capture_name.contains("trait") {
        SymbolKind::Interface
    } else if capture_name.contains("module") {
        SymbolKind::Module
    } else if capture_name.contains("macro") {
        SymbolKind::Macro
    } else if capture_name.contains("call") {
        SymbolKind::Call
    } else {
        SymbolKind::Other
    }
}

/// Represents a symbol found in source code
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub file_path: std::path::PathBuf,
}

/// Types of symbols that can be extracted
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Class,
    Function,
    Method,
    Interface,
    Module,
    Macro,
    Call,
    Other,
}

impl SymbolKind {
    /// Get the priority of this symbol kind for ranking
    pub fn priority(&self) -> u32 {
        match self {
            SymbolKind::Class => 100,
            SymbolKind::Interface => 90,
            SymbolKind::Function => 80,
            SymbolKind::Method => 70,
            SymbolKind::Module => 60,
            SymbolKind::Macro => 50,
            SymbolKind::Call => 10,
            SymbolKind::Other => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_language() {
        assert!(get_language("rust").is_ok());
        assert!(get_language("python").is_ok());
        assert!(get_language("javascript").is_ok());
        assert!(get_language("go").is_ok());
        assert!(get_language("unknown").is_err());
    }

    #[test]
    fn test_get_query() {
        assert!(get_query("rust").is_ok());
        assert!(get_query("python").is_ok());
        assert!(get_query("javascript").is_ok());
        assert!(get_query("go").is_ok());
        assert!(get_query("unknown").is_err());
    }

    #[test]
    fn test_symbol_kind_priority() {
        assert!(SymbolKind::Class.priority() > SymbolKind::Function.priority());
        assert!(SymbolKind::Function.priority() > SymbolKind::Method.priority());
        assert!(SymbolKind::Method.priority() > SymbolKind::Call.priority());
    }

    #[test]
    fn test_parse_rust_file() -> Result<()> {
        let content = r#"
struct MyStruct {
    field: i32,
}

impl MyStruct {
    fn new() -> Self {
        Self { field: 0 }
    }
    
    fn method(&self) -> i32 {
        self.field
    }
}

fn main() {
    let s = MyStruct::new();
    println!("{}", s.method());
}
"#;
        
        let symbols = parse_file(&PathBuf::from("test.rs"), "rust", content)?;
        
        // Should find struct, impl, functions, and method calls
        assert!(!symbols.is_empty());
        
        // Check that we found some expected symbols
        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"MyStruct"));
        assert!(symbol_names.contains(&"new"));
        assert!(symbol_names.contains(&"method"));
        assert!(symbol_names.contains(&"main"));
        
        Ok(())
    }
}
