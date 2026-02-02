use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SearchConfig {
    /// File extensions to search in
    pub file_extensions: Vec<String>,

    /// Maximum number of search results to show
    pub max_results: usize,

    /// Include file context lines around matches
    pub context_lines: usize,

    /// Case sensitive search
    pub case_sensitive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 50,
            context_lines: 2,
            case_sensitive: false,
            file_extensions: vec!["typ".to_string(), "md".to_string()],
        }
    }
}
