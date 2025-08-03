use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use aider_repo_map::{FileDiscovery, parse_file, SymbolRanker, TreeRenderer, TokenCounter, normalize_display_path};
use std::fs;


#[cfg(test)]
mod cli_tests {
    use aider_repo_map::normalize_display_path;

    #[test]
    fn test_normalize_display_path_strips_dot_slash() {
        assert_eq!(normalize_display_path("./src/main.rs"), "src/main.rs");
        assert_eq!(normalize_display_path("src/main.rs"), "src/main.rs");
        assert_eq!(normalize_display_path("./README.md"), "README.md");
        assert_eq!(normalize_display_path("README.md"), "README.md");
    }
}

#[derive(Parser)]
#[command(name = "repo-map")]
#[command(about = "Generate a repository map showing code structure and symbols")]
#[command(version)]
struct Args {
    /// Token limit for the output (similar to --map-tokens in aider)
    #[arg(long, default_value = "1024")]
    size: usize,

    /// Control how often the repo map is refreshed
    #[arg(long, default_value = "auto")]
    #[arg(value_parser = ["auto", "always", "files", "manual"])]
    refresh: String,

    /// Multiplier for map tokens when no files are specified
    #[arg(long, default_value = "2.0")]
    multiplier_no_files: f64,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Root directory to analyze
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Files or directories to analyze
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        eprintln!("Repo map configuration:");
        eprintln!("  Size: {} tokens", args.size);
        eprintln!("  Refresh: {}", args.refresh);
        eprintln!("  Multiplier: {}", args.multiplier_no_files);
        // Normalize root display by stripping leading ./ if present
        let root_display = normalize_display_path(args.root.display().to_string());
        eprintln!("  Root: {}", root_display);
        eprintln!("  Files: {:?}", args.files);
    }

    // Initialize file discovery
    let file_discovery = FileDiscovery::new(args.root.clone());

    // Find source files
    let files = file_discovery.find_source_files(&args.files)?;

    if args.verbose {
        eprintln!("Found {} source files:", files.len());
        for file in &files {
            if let Some(lang) = file_discovery.get_language(file) {
                // Normalize display path by stripping leading ./ if present
                let display_path = normalize_display_path(file.display().to_string());
                eprintln!("  {} ({})", display_path, lang);
            } else {
                let display_path = normalize_display_path(file.display().to_string());
                eprintln!("  {} (unknown)", display_path);
            }
        }
    }

    // Parse files and extract symbols
    let mut all_symbols = Vec::new();

    for file in &files {
        if let Some(language) = file_discovery.get_language(file) {
            match fs::read_to_string(file) {
                Ok(content) => {
                    match parse_file(file, language, &content) {
                        Ok(symbols) => {
                            if args.verbose {
                                let display_path = normalize_display_path(file.display().to_string());
                                eprintln!("  Found {} symbols in {}", symbols.len(), display_path);
                            }
                            all_symbols.extend(symbols);
                        }
                        Err(e) => {
                            if args.verbose {
                                let display_path = normalize_display_path(file.display().to_string());
                                eprintln!("  Failed to parse {}: {}", display_path, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    if args.verbose {
                        let display_path = normalize_display_path(file.display().to_string());
                        eprintln!("  Failed to read {}: {}", display_path, e);
                    }
                }
            }
        }
    }

    if args.verbose {
        eprintln!("Total symbols found: {}", all_symbols.len());
    }

    // Rank symbols by importance
    let ranker = SymbolRanker::new();
    let ranked_symbols = ranker.rank_symbols(&all_symbols);

    if args.verbose {
        eprintln!("Ranked symbols: {}", ranked_symbols.len());
    }

    // Use binary search to find optimal number of symbols within token limit
    let renderer = TreeRenderer::new();
    let optimal_count = find_optimal_symbol_count(&ranked_symbols, &renderer, args.size)?;

    if args.verbose {
        eprintln!("Using {} symbols to fit within {} tokens", optimal_count, args.size);
    }

    // Generate and output the repo map
    let repo_map = renderer.render_tree(&ranked_symbols, optimal_count)?;
    let token_count = TokenCounter::count_tokens(&repo_map);

    if args.verbose {
        eprintln!("Generated repo map with {} tokens", token_count);
    }

    println!("{}", repo_map);

    Ok(())
}

/// Use binary search to find the optimal number of symbols that fit within the token limit
fn find_optimal_symbol_count(
    ranked_symbols: &[aider_repo_map::RankedSymbol],
    renderer: &TreeRenderer,
    max_tokens: usize
) -> Result<usize> {
    if ranked_symbols.is_empty() {
        return Ok(0);
    }

    let mut left = 1;
    let mut right = ranked_symbols.len();
    let mut best_count = 1;

    while left <= right {
        let mid = (left + right) / 2;

        // Try rendering with 'mid' symbols
        match renderer.render_tree(ranked_symbols, mid) {
            Ok(output) => {
                let token_count = TokenCounter::count_tokens(&output);

                if token_count <= max_tokens {
                    best_count = mid;
                    left = mid + 1; // Try more symbols
                } else {
                    right = mid - 1; // Try fewer symbols
                }
            }
            Err(_) => {
                // If rendering fails, try fewer symbols
                right = mid - 1;
            }
        }
    }

    Ok(best_count)
}
