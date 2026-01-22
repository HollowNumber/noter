//! Search engine for note content
//!
//! Handles searching through files with various options and filters.

use crate::core::directories::DirectoryScanner;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub max_results: usize,
    pub context_lines: usize,
    pub file_extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchIndex {
    pub word_map: HashMap<String, Vec<SearchLocation>>,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchLocation {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
}

pub struct SearchEngine;

impl SearchEngine {
    const INDEX_FILE: &'static str = ".notes-search-index";

    pub fn search_in_directory<P: AsRef<Path>>(
        dir: P,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchMatch>> {
        let mut results = Vec::new();
        Self::search_recursive(dir.as_ref(), query, options, &mut results)?;

        // Limit results
        results.truncate(options.max_results);
        Ok(results)
    }

    fn search_recursive(
        dir: &Path,
        query: &str,
        options: &SearchOptions,
        results: &mut Vec<SearchMatch>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::search_recursive(&path, query, options, results)?;
            } else if Self::should_search_file(&path, options) {
                Self::search_in_file(&path, query, options, results)?;
            }
        }
        Ok(())
    }

    fn should_search_file(path: &Path, options: &SearchOptions) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            options.file_extensions.contains(&ext_str)
        } else {
            false
        }
    }

    fn search_in_file(
        path: &Path,
        query: &str,
        options: &SearchOptions,
        results: &mut Vec<SearchMatch>,
    ) -> Result<()> {
        let content = fs::read_to_string(path)?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(match_pos) = Self::find_match(line, query, options.case_sensitive) {
                results.push(SearchMatch {
                    file_path: path.to_path_buf(),
                    line_number: line_num + 1,
                    line_content: line.trim().to_string(),
                    match_start: match_pos,
                    match_end: match_pos + query.len(),
                });
            }
        }

        Ok(())
    }

    fn find_match(line: &str, query: &str, case_sensitive: bool) -> Option<usize> {
        if case_sensitive {
            line.find(query)
        } else {
            line.to_lowercase().find(&query.to_lowercase())
        }
    }

    pub fn build_index(notes_dir: &Path) -> Result<SearchIndex> {
        let mut word_map = HashMap::new();
        let files = DirectoryScanner::scan_directory_for_files(notes_dir, &["typ", "md"])?;

        for file_info in files {
            if let Ok(content) = fs::read_to_string(&file_info.path) {
                for (line_num, line) in content.lines().enumerate() {
                    for (col, word) in line.split_whitespace().enumerate() {
                        let word_clean = word.to_lowercase();
                        let location = SearchLocation {
                            file_path: file_info.path.clone(),
                            line_number: line_num + 1,
                            column: col,
                        };

                        word_map
                            .entry(word_clean)
                            .or_insert_with(Vec::new)
                            .push(location);
                    }
                }
            }
        }

        Ok(SearchIndex {
            word_map,
            last_updated: SystemTime::now(),
        })
    }

    pub fn search_indexed(&self, index: &SearchIndex, query: &str) -> Vec<SearchLocation> {
        let query_lower = query.to_lowercase();
        index
            .word_map
            .get(&query_lower)
            .cloned()
            .unwrap_or_default()
    }

    /// Get or create search index with automatic freshness checking
    pub fn get_or_build_index(notes_dir: &Path) -> Result<SearchIndex> {
        let index_path = notes_dir.join(Self::INDEX_FILE);

        // Try to load existing index
        if let Ok(index) = Self::load_index(&index_path) {
            // Check if index is still fresh
            if Self::is_index_fresh(&index, notes_dir)? {
                return Ok(index);
            }
        }

        // Build new index
        let index = Self::build_index(notes_dir)?;
        Self::save_index(&index, &index_path)?;
        Ok(index)
    }

    /// Check if index is newer than all files
    fn is_index_fresh(index: &SearchIndex, notes_dir: &Path) -> Result<bool> {
        let files = DirectoryScanner::scan_directory_for_files(notes_dir, &["typ", "md"])?;

        for file_info in files {
            if file_info.modified > index.last_updated {
                return Ok(false); // Found newer file
            }
        }
        Ok(true)
    }

    /// Save index to disk
    fn save_index(index: &SearchIndex, path: &Path) -> Result<()> {
        let serialized = serde_json::to_string(index)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Load index from disk
    fn load_index(path: &Path) -> Result<SearchIndex> {
        let content = fs::read_to_string(path)?;
        let index = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// Fast indexed search
    pub fn search_with_index(index: &SearchIndex, query: &str) -> Vec<SearchLocation> {
        index
            .word_map
            .get(&query.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path, files: &[(&str, &str)]) -> Result<()> {
        for (filename, content) in files {
            let file_path = dir.join(filename);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, content)?;
        }
        Ok(())
    }

    #[test]
    fn test_build_index() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test files
        create_test_files(
            temp_path,
            &[
                ("file1.typ", "hello world algorithms\ndata structures"),
                ("file2.md", "machine learning neural networks"),
                ("subdir/file3.typ", "programming rust systems"),
            ],
        )?;

        let index = SearchEngine::build_index(temp_path)?;

        // Debug: Print what words were actually indexed
        println!(
            "Indexed words: {:?}",
            index.word_map.keys().collect::<Vec<_>>()
        );

        // Check that words are indexed
        assert!(index.word_map.contains_key("hello"));
        assert!(index.word_map.contains_key("algorithms"));
        assert!(index.word_map.contains_key("machine"));

        // Check if the subdirectory file was processed
        assert!(
            index.word_map.contains_key("programming"),
            "Word 'programming' should be indexed"
        );
        assert!(
            index.word_map.contains_key("systems"),
            "Word 'systems' should be indexed"
        );
        assert!(
            index.word_map.contains_key("rust"),
            "Word 'rust' should be indexed"
        );

        // Check location information
        let hello_locations = index.word_map.get("hello").unwrap();
        assert_eq!(hello_locations.len(), 1);
        assert_eq!(hello_locations[0].line_number, 1);
        assert_eq!(hello_locations[0].column, 0);

        Ok(())
    }

    #[test]
    fn test_search_with_index() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(
            temp_path,
            &[
                (
                    "test1.typ",
                    "algorithms and data structures\nsorting algorithms",
                ),
                ("test2.md", "machine learning algorithms\nneural networks"),
            ],
        )?;

        let index = SearchEngine::build_index(temp_path)?;
        let results = SearchEngine::search_with_index(&index, "algorithms");

        assert_eq!(results.len(), 3); // Should find 3 occurrences

        // Check that results contain expected files
        let file_paths: Vec<_> = results
            .iter()
            .map(|r| r.file_path.file_name().unwrap().to_str().unwrap())
            .collect();

        assert!(file_paths.contains(&"test1.typ"));
        assert!(file_paths.contains(&"test2.md"));

        Ok(())
    }

    #[test]
    fn test_case_insensitive_search() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(temp_path, &[("test.typ", "ALGORITHMS and Data STRUCTURES")])?;

        let index = SearchEngine::build_index(temp_path)?;

        // Search with lowercase should find uppercase words
        let results = SearchEngine::search_with_index(&index, "algorithms");
        assert_eq!(results.len(), 1);

        let results = SearchEngine::search_with_index(&index, "data");
        assert_eq!(results.len(), 1);

        Ok(())
    }

    #[test]
    fn test_index_persistence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let index_path = temp_path.join(".notes-search-index");

        create_test_files(temp_path, &[("test.typ", "persistent indexing test")])?;

        // Build and save index
        let index = SearchEngine::build_index(temp_path)?;
        let serialized = serde_json::to_string(&index)?;
        fs::write(&index_path, serialized)?;

        // Load index from disk
        let loaded_index = SearchEngine::load_index(&index_path)?;

        assert_eq!(index.word_map.len(), loaded_index.word_map.len());
        assert!(loaded_index.word_map.contains_key("persistent"));
        assert!(loaded_index.word_map.contains_key("indexing"));

        Ok(())
    }

    #[test]
    fn test_index_freshness() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(temp_path, &[("test.typ", "initial content")])?;

        let index = SearchEngine::build_index(temp_path)?;

        // Index should be fresh immediately after creation
        assert!(SearchEngine::is_index_fresh(&index, temp_path)?);

        // Wait a bit and modify a file
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(temp_path.join("test.typ"), "modified content")?;

        // Index should now be stale
        assert!(!SearchEngine::is_index_fresh(&index, temp_path)?);

        Ok(())
    }

    #[test]
    fn test_get_or_build_index() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(temp_path, &[("test.typ", "test content for indexing")])?;

        // First call should build new index
        let index1 = SearchEngine::get_or_build_index(temp_path)?;
        assert!(index1.word_map.contains_key("test"));

        // Check that index file was created
        let index_path = temp_path.join(".notes-search-index");
        assert!(index_path.exists());

        // Second call should use existing index (if fresh)
        let index2 = SearchEngine::get_or_build_index(temp_path)?;
        assert_eq!(index1.word_map.len(), index2.word_map.len());

        Ok(())
    }

    #[test]
    fn test_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let index = SearchEngine::build_index(temp_path)?;
        assert!(index.word_map.is_empty());

        Ok(())
    }

    #[test]
    fn test_file_extensions_filter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(
            temp_path,
            &[
                ("test.typ", "typst file content"),
                ("test.md", "markdown file content"),
                ("test.txt", "text file content"), // Should be ignored
                ("test.rs", "rust file content"),  // Should be ignored
            ],
        )?;

        let index = SearchEngine::build_index(temp_path)?;

        // Should only index .typ and .md files
        assert!(index.word_map.contains_key("typst"));
        assert!(index.word_map.contains_key("markdown"));
        assert!(!index.word_map.contains_key("rust"));
        assert!(!index.word_map.contains_key("text"));

        Ok(())
    }

    #[test]
    fn test_search_no_results() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        create_test_files(temp_path, &[("test.typ", "algorithms and data structures")])?;

        let index = SearchEngine::build_index(temp_path)?;
        let results = SearchEngine::search_with_index(&index, "nonexistent");

        assert!(results.is_empty());

        Ok(())
    }

    #[test]
    fn test_large_index() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create many files to test performance
        let mut files = Vec::new();
        for i in 0..100 {
            files.push((
                format!("file_{}.typ", i),
                format!("content {} algorithms data structures", i),
            ));
        }

        let files_ref: Vec<_> = files
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_str()))
            .collect();

        create_test_files(temp_path, &files_ref)?;

        let start = std::time::Instant::now();
        let index = SearchEngine::build_index(temp_path)?;
        let build_time = start.elapsed();

        println!("Built index for {} files in {:?}", files.len(), build_time);

        assert!(index.word_map.contains_key("algorithms"));

        // Should have 100 occurrences of "algorithms"
        let algorithm_locations = index.word_map.get("algorithms").unwrap();
        assert_eq!(algorithm_locations.len(), 100);

        Ok(())
    }

    // Add a specific test to debug the subdirectory issue
    #[test]
    fn test_subdirectory_indexing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create a subdirectory structure
        create_test_files(
            temp_path,
            &[
                ("root_file.typ", "root content"),
                ("subdir/nested_file.typ", "nested rust content"),
                ("deep/nested/file.typ", "deep nested content"),
            ],
        )?;

        let index = SearchEngine::build_index(temp_path)?;

        // Debug: Print all indexed words
        println!(
            "All indexed words: {:?}",
            index.word_map.keys().collect::<Vec<_>>()
        );

        // Check that files from all levels are indexed
        assert!(index.word_map.contains_key("root"));
        assert!(index.word_map.contains_key("nested"));
        assert!(index.word_map.contains_key("rust"));
        assert!(index.word_map.contains_key("deep"));

        Ok(())
    }
}
