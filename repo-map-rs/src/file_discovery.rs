use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Supported programming languages and their file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs",   // Rust
    "py",   // Python
    "js",   // JavaScript
    "ts",   // TypeScript
    "jsx",  // React JSX
    "tsx",  // React TSX
    "go",   // Go
    "java", // Java
    "c",    // C
    "cpp",  // C++
    "cc",   // C++
    "cxx",  // C++
    "h",    // C/C++ headers
    "hpp",  // C++ headers
    "cs",   // C#
    "rb",   // Ruby
    "php",  // PHP
    "swift", // Swift
    "kt",   // Kotlin
    "scala", // Scala
    "clj",  // Clojure
    "hs",   // Haskell
    "ml",   // OCaml
    "elm",  // Elm
];

/// File discovery configuration
pub struct FileDiscovery {
    root: PathBuf,
    respect_gitignore: bool,
}

impl FileDiscovery {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            respect_gitignore: true,
        }
    }

    /// Find all source files in the given paths
    pub fn find_source_files(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if paths.is_empty() {
            // If no paths specified, scan the root directory
            files.extend(self.scan_directory(&self.root)?);
        } else {
            // Process each specified path
            for path in paths {
                let full_path = if path.is_absolute() {
                    path.clone()
                } else {
                    self.root.join(path)
                };

                if full_path.is_file() {
                    if self.is_source_file(&full_path) {
                        files.push(full_path);
                    }
                } else if full_path.is_dir() {
                    files.extend(self.scan_directory(&full_path)?);
                }
            }
        }

        // Sort files for consistent output
        files.sort();
        Ok(files)
    }

    /// Scan a directory for source files
    fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        let walker = WalkBuilder::new(dir)
            .git_ignore(self.respect_gitignore)
            .git_exclude(self.respect_gitignore)
            .git_global(self.respect_gitignore)
            .hidden(false) // Include hidden files for now
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_source_file(path) {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Check if a file is a supported source file
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return SUPPORTED_EXTENSIONS.contains(&ext_str);
            }
        }
        false
    }

    /// Get the language for a file based on its extension
    pub fn get_language(&self, path: &Path) -> Option<&'static str> {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str {
                    "rs" => Some("rust"),
                    "py" => Some("python"),
                    "js" | "jsx" => Some("javascript"),
                    "ts" | "tsx" => Some("typescript"),
                    "go" => Some("go"),
                    "java" => Some("java"),
                    "c" | "h" => Some("c"),
                    "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
                    "cs" => Some("c_sharp"),
                    "rb" => Some("ruby"),
                    "php" => Some("php"),
                    "swift" => Some("swift"),
                    "kt" => Some("kotlin"),
                    "scala" => Some("scala"),
                    "clj" => Some("clojure"),
                    "hs" => Some("haskell"),
                    "ml" => Some("ocaml"),
                    "elm" => Some("elm"),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_source_file() {
        let discovery = FileDiscovery::new(PathBuf::from("."));
        
        assert!(discovery.is_source_file(Path::new("test.rs")));
        assert!(discovery.is_source_file(Path::new("test.py")));
        assert!(discovery.is_source_file(Path::new("test.js")));
        assert!(!discovery.is_source_file(Path::new("test.txt")));
        assert!(!discovery.is_source_file(Path::new("README.md")));
    }

    #[test]
    fn test_get_language() {
        let discovery = FileDiscovery::new(PathBuf::from("."));
        
        assert_eq!(discovery.get_language(Path::new("test.rs")), Some("rust"));
        assert_eq!(discovery.get_language(Path::new("test.py")), Some("python"));
        assert_eq!(discovery.get_language(Path::new("test.js")), Some("javascript"));
        assert_eq!(discovery.get_language(Path::new("test.go")), Some("go"));
        assert_eq!(discovery.get_language(Path::new("test.txt")), None);
    }

    #[test]
    fn test_find_source_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();
        
        // Create test files
        fs::write(root.join("test.rs"), "fn main() {}")?;
        fs::write(root.join("test.py"), "def main(): pass")?;
        fs::write(root.join("README.md"), "# Test")?;
        
        let discovery = FileDiscovery::new(root.clone());
        let files = discovery.find_source_files(&[])?;
        
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "test.rs"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "test.py"));
        
        Ok(())
    }
}
