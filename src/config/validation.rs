use anyhow::Result;
use std::path::Path;

use super::Config;

impl Config {
    /// Validate configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        if self.author == "Your Name" {
            warnings.push("Author name is set to default value".to_string());
        }

        if self.search.max_results == 0 {
            warnings.push("Max search results is set to 0".to_string());
        }

        // Check if template directory exists
        if !Path::new(&self.paths.templates_dir).exists() {
            warnings.push(format!(
                "Template directory '{}' doesn't exist",
                self.paths.templates_dir
            ));
        }

        Ok(warnings)
    }
}
