use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PathConfig {
    /// Base directory for notes
    pub notes_dir: String,

    /// Obsidian vault directory
    pub obsidian_dir: String,

    /// Templates directory
    pub templates_dir: String,

    /// Typst packages directory
    pub typst_packages_dir: String,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            notes_dir: "notes".to_string(),
            obsidian_dir: "obsidian-vault".to_string(),
            templates_dir: "templates".to_string(),
            // Use data_local_dir which maps to the right location on each OS:
            // Windows: %LOCALAPPDATA%
            // macOS: ~/Library/Application Support
            // Linux: ~/.local/share
            typst_packages_dir: dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("typst/packages/local")
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl PathConfig {
    /// Resolve all paths to absolute paths
    pub fn resolve_paths(&mut self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        self.notes_dir = Self::resolve_path(&self.notes_dir, &current_dir)?;
        self.obsidian_dir = Self::resolve_path(&self.obsidian_dir, &current_dir)?;
        self.templates_dir = Self::resolve_path(&self.templates_dir, &current_dir)?;

        Ok(())
    }

    fn resolve_path(path: &str, base: &Path) -> Result<String> {
        let path_buf = if Path::new(path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            base.join(path)
        };

        if cfg!(windows) {
            // On Windows, build absolute path manually to avoid \\?\ prefix
            let absolute = if path_buf.is_absolute() {
                path_buf
            } else {
                std::env::current_dir()?.join(&path_buf)
            };

            // Convert to string and normalize path separators
            let path_str = absolute.to_string_lossy().to_string();

            // Remove any \\?\ prefix if it somehow got added
            let clean_path = path_str
                .strip_prefix(r"\\?\")
                .unwrap_or(&path_str)
                .to_string();

            return Ok(clean_path.replace('/', "\\"));
        }
        // On Unix, canonicalize is safe
        Ok(path_buf
            .canonicalize()
            .unwrap_or(path_buf)
            .to_string_lossy()
            .to_string())
    }
}
