# Rust Repo Map CLI Tool - Implementation Plan

## Overview
Create a Rust CLI tool that replicates the functionality of the Python tree-sitter repo map implementation from aider. The tool should generate a hierarchical view of code symbols (functions, classes, methods) from source files.

## Key Features to Implement

### 1. CLI Interface (using clap)
- `--size` / `--map-tokens`: Token limit for output (default: 1024)
- `--refresh`: Refresh strategy (auto, always, files, manual) (default: auto)
- `--multiplier-no-files`: Multiplier when no files specified (default: 2.0)
- `--verbose`: Enable verbose output
- `--root`: Root directory to analyze (default: current directory)
- `[FILES]...`: Optional list of files/directories to analyze

### 2. Core Components

#### File Discovery
- Recursively find source files in directories
- Support common programming languages (Rust, Python, JavaScript, Go, etc.)
- Filter by file extensions
- Respect .gitignore patterns

#### Tree-sitter Integration
- Parse source files using tree-sitter
- Load appropriate language parsers
- Execute tree-sitter queries to extract symbols
- Support multiple languages with query files

#### Symbol Extraction
- Extract functions, classes, methods, structs, enums, etc.
- Capture symbol names, types, line numbers, and definitions
- Store symbols with metadata (file, line, kind, name)

#### Symbol Ranking
- Rank symbols by importance/frequency of references
- Consider cross-file references
- Prioritize public/exported symbols
- Weight by symbol type (classes > functions > variables)

#### Tree Rendering
- Generate hierarchical output similar to Python version
- Format: `filename:\n│symbol definition\n⋮\n│another symbol\n⋮`
- Include context lines for symbol definitions
- Use Unicode box-drawing characters for visual structure

#### Token Management
- Count tokens in generated output
- Binary search to find optimal symbol count within token limit
- Dynamically adjust based on --size parameter

### 3. Project Structure
```
src/
├── main.rs              # CLI entry point and argument parsing
├── lib.rs               # Library exports
├── file_discovery.rs    # File finding and filtering
├── tree_sitter_utils.rs # Tree-sitter parsing and queries
├── symbol_extractor.rs  # Symbol extraction logic
├── symbol_ranker.rs     # Symbol ranking algorithms
├── tree_renderer.rs     # Output formatting and rendering
├── token_counter.rs     # Token counting utilities
└── queries/             # Tree-sitter query files
    ├── rust.scm
    ├── python.scm
    ├── javascript.scm
    └── ...
```

### 4. Dependencies
- `clap` - CLI argument parsing
- `tree-sitter` - Source code parsing
- `tree-sitter-*` - Language-specific parsers
- `walkdir` - Directory traversal
- `ignore` - .gitignore support
- `anyhow` - Error handling
- `serde` - Serialization (for caching)
- `tokio` - Async runtime (if needed)

### 5. Implementation Phases

#### Phase 1: Basic CLI and File Discovery
- Set up Cargo project with dependencies
- Implement CLI argument parsing with clap
- Basic file discovery and filtering
- Simple output to verify file finding works

#### Phase 2: Tree-sitter Integration
- Add tree-sitter parsing for one language (Rust)
- Implement basic symbol extraction
- Create simple output format
- Test with sample Rust files

#### Phase 3: Symbol Ranking and Tree Rendering
- Implement symbol ranking algorithm
- Create tree rendering with proper formatting
- Add token counting and size management
- Test output format matches Python version

#### Phase 4: Multi-language Support
- Add support for Python, JavaScript, Go
- Implement query file loading system
- Test with various codebases

#### Phase 5: Advanced Features
- Add caching for performance
- Implement all refresh strategies
- Add verbose output and debugging
- Performance optimization

### 6. Output Format
The tool should produce output similar to:
```
src/main.rs:
│fn main() {
│    let args = Args::parse();
⋮
│struct Args {
│    #[arg(long, default_value = "1024")]
│    size: usize,
⋮

src/lib.rs:
│pub struct RepoMap {
│    root: PathBuf,
│    size: usize,
⋮
│impl RepoMap {
│    pub fn new(root: PathBuf, size: usize) -> Self {
⋮
│    pub fn generate(&self) -> Result<String> {
⋮
```

### 7. Testing Strategy
- Unit tests for each component
- Integration tests with sample codebases
- Compare output with Python version
- Performance benchmarks
- Test with various programming languages

### 8. Success Criteria
- CLI interface matches Python version functionality
- Output format is compatible and readable
- Performance is acceptable for large codebases
- Supports major programming languages
- Token counting works accurately
- Symbol ranking produces meaningful results
