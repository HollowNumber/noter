//! Search command implementation
//!
//! Thin command layer that uses core search engine and ui formatters.

use anyhow::Result;
use std::path::Path;

use crate::config::{Config, get_config};
use crate::core::directories::DirectoryScanner;
use crate::core::search_engine::{SearchEngine, SearchLocation, SearchMatch, SearchOptions};
use crate::display::formatters::Formatters;
use crate::display::output::{OutputManager, Status};

pub fn search_notes(query: &str) -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, &format!("Searching for '{}'", query));

    let notes_path = Path::new(&config.paths.notes_dir);
    if !notes_path.exists() {
        OutputManager::print_status(
            Status::Warning,
            &format!("No notes directory found at: {}", config.paths.notes_dir),
        );
        return Ok(());
    }

    // Get search results using your existing SearchEngine
    let results = if should_use_index(notes_path)? {
        search_with_index(notes_path, query, &config)?
    } else {
        search_without_index(query, &config)?
    };

    display_search_results(results, query, &config)?;
    Ok(())
}

/// Search using index - returns Vec<SearchMatch>
fn search_with_index(notes_path: &Path, query: &str, config: &Config) -> Result<Vec<SearchMatch>> {
    let index = SearchEngine::get_or_build_index(notes_path)?;
    let locations = SearchEngine::search_with_index(&index, query);

    // Convert SearchLocations to SearchMatch
    let mut results = Vec::new();
    for location in locations {
        if let Ok(search_match) = build_search_match_from_location(location, query, config) {
            results.push(search_match);
        }
    }

    // Limit results
    results.truncate(config.search.max_results);
    Ok(results)
}

/// Search without index - use your existing method
fn search_without_index(query: &str, config: &Config) -> Result<Vec<SearchMatch>> {
    let search_options = SearchOptions {
        case_sensitive: config.search.case_sensitive,
        max_results: config.search.max_results,
        context_lines: config.search.context_lines,
        file_extensions: config.search.file_extensions.clone(),
    };

    SearchEngine::search_in_directory(&config.paths.notes_dir, query, &search_options)
}

/// Convert SearchLocation to SearchMatch
fn build_search_match_from_location(
    location: SearchLocation,
    query: &str,
    config: &Config,
) -> Result<SearchMatch> {
    let content = std::fs::read_to_string(&location.file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if location.line_number == 0 || location.line_number > lines.len() {
        return Err(anyhow::anyhow!("Invalid line number"));
    }

    let line_content = lines[location.line_number - 1].to_string();

    // Find the query in the line content
    let query_lower = query.to_lowercase();
    let line_lower = line_content.to_lowercase();

    let (match_start, match_end) = if config.search.case_sensitive {
        if let Some(pos) = line_content.find(query) {
            (pos, pos + query.len())
        } else {
            (0, 0)
        }
    } else if let Some(pos) = line_lower.find(&query_lower) {
        (pos, pos + query_lower.len())
    } else {
        (0, 0)
    };

    Ok(SearchMatch {
        file_path: location.file_path,
        line_number: location.line_number,
        line_content,
        match_start,
        match_end,
    })
}

/// Display results using your existing formatter
fn display_search_results(results: Vec<SearchMatch>, query: &str, config: &Config) -> Result<()> {
    if results.is_empty() {
        OutputManager::print_status(Status::Info, "No results found");
    } else {
        let formatted_results = Formatters::format_search_results(&results, query);
        println!("{}", formatted_results);

        if results.len() >= config.search.max_results {
            OutputManager::print_status(
                Status::Info,
                &format!(
                    "Showing first {} results (limit reached)",
                    config.search.max_results
                ),
            );
        }
    }
    Ok(())
}

pub fn rebuild_index(force: bool) -> Result<()> {
    let config = get_config()?;
    let notes_path = Path::new(&config.paths.notes_dir);

    if !notes_path.exists() {
        OutputManager::print_status(
            Status::Error,
            &format!("Notes directory not found: {}", config.paths.notes_dir),
        );
        return Ok(());
    }

    // Debug: Print the directory being scanned
    println!("Scanning directory: {}", notes_path.display());

    // Check if we have enough files to warrant an index
    let files = DirectoryScanner::scan_directory_for_files(notes_path, &["typ", "md"])?;

    // Debug: Print found files
    println!("Files found:");
    for file in &files {
        println!("  - {}", file.path.display());
    }

    if files.is_empty() {
        OutputManager::print_status(Status::Info, "No files found to index");

        // Debug: List all files in the directory (regardless of extension)
        println!("All files in directory:");
        if let Ok(entries) = std::fs::read_dir(notes_path) {
            for entry in entries.flatten() {
                println!("  - {}", entry.path().display());
            }
        }

        return Ok(());
    }

    if !force && files.len() <= 50 {
        OutputManager::print_status(
            Status::Info,
            &format!(
                "Only {} files found. Index is typically used for 50+ files. Use --force to rebuild anyway.",
                files.len()
            ),
        );
        return Ok(());
    }

    OutputManager::print_status(
        Status::Loading,
        &format!("Rebuilding search index for {} files...", files.len()),
    );

    // Remove existing index file if it exists
    let index_path = notes_path.join(".notes-search-index");
    if index_path.exists() {
        std::fs::remove_file(&index_path)?;
        OutputManager::print_status(Status::Info, "Removed existing index");
    }

    // Build new index
    let start_time = std::time::Instant::now();
    let index = SearchEngine::build_index(notes_path)?;
    let duration = start_time.elapsed();

    // Save the new index
    let serialized = serde_json::to_string(&index)?;
    std::fs::write(&index_path, serialized)?;

    OutputManager::print_status(
        Status::Success,
        &format!(
            "Search index rebuilt successfully in {:.2}s! Indexed {} files with {} words.",
            duration.as_secs_f64(),
            files.len(),
            index.word_map.len()
        ),
    );

    Ok(())
}

/// Decide whether to use index based on collection size
fn should_use_index(notes_path: &Path) -> Result<bool> {
    let files = DirectoryScanner::scan_directory_for_files(notes_path, &["typ", "md"])?;
    Ok(files.len() > 50) // Use index for collections with 50+ files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_should_use_index() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create few files - should not use index
        for i in 0..10 {
            fs::write(temp_path.join(format!("file{}.typ", i)), "test content")?;
        }
        assert!(!should_use_index(temp_path)?);

        // Create many files - should use index
        for i in 10..60 {
            fs::write(temp_path.join(format!("file{}.typ", i)), "test content")?;
        }
        assert!(should_use_index(temp_path)?);

        Ok(())
    }

    #[test]
    fn test_search_with_index_vs_without() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(
            temp_path.join("test1.typ"),
            "algorithms and data structures\nsorting algorithms",
        )?;
        fs::write(
            temp_path.join("test2.md"),
            "machine learning algorithms\nneural networks",
        )?;

        let mut config = Config::default();
        config.paths.notes_dir = temp_path.to_string_lossy().to_string();

        // Search with index
        let indexed_results = search_with_index(temp_path, "algorithms", &config)?;

        // Search without index
        let direct_results = search_without_index("algorithms", &config)?;

        // Results should be similar (may differ slightly in ordering/format)
        assert!(!indexed_results.is_empty());
        assert!(!direct_results.is_empty());

        // Both should find occurrences in both files
        let indexed_files: std::collections::HashSet<_> = indexed_results
            .iter()
            .map(|r| r.file_path.file_name().unwrap())
            .collect();
        let direct_files: std::collections::HashSet<_> = direct_results
            .iter()
            .map(|r| r.file_path.file_name().unwrap())
            .collect();

        assert!(indexed_files.len() >= 2); // Should find in both files
        assert!(direct_files.len() >= 2);

        Ok(())
    }

    #[test]
    fn test_build_search_match_from_location() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let file_path = temp_path.join("test.typ");

        fs::write(&file_path, "line one\nthis has algorithms\nline three")?;

        let location = SearchLocation {
            file_path: file_path.clone(),
            line_number: 2,
            column: 2,
        };

        let config = Config::default();
        let search_match = build_search_match_from_location(location, "algorithms", &config)?;

        assert_eq!(search_match.file_path, file_path);
        assert_eq!(search_match.line_number, 2);
        assert_eq!(search_match.line_content, "this has algorithms");
        assert!(search_match.match_start > 0);
        assert!(search_match.match_end > search_match.match_start);

        Ok(())
    }
}
