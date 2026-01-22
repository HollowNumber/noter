use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::path::PathBuf;

use super::Config;
use super::metadata::CONFIG_VERSION;

impl Config {
    /// Load configuration from file or create default with automatic migration
    ///
    /// This method handles three scenarios:
    /// 1. **No config exists**: Creates a new default config
    /// 2. **Compatible config**: Loads and potentially migrates to current version
    /// 3. **Incompatible config**: Creates backup and recovers what it can
    ///
    /// The migration happens automatically and transparently to the user.
    ///
    /// # Returns
    ///
    /// Returns the loaded and migrated configuration with resolved paths.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use noter::config::Config;
    ///
    /// let config = Config::load()?;
    /// println!("Author: {}", config.author);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;

            // Try to load with lenient deserialization (serde(default) helps here)
            match serde_json::from_str::<Config>(&content) {
                Ok(mut config) => {
                    // Check if migration is needed
                    if Self::needs_migration(&config) {
                        eprintln!("Config format has changed. Migrating...");
                        config = Self::migrate(config)?;
                        config.save()?;
                        eprintln!("Config migrated successfully!");
                    }
                    config
                }
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                    eprintln!("Creating backup and recovering values from old config...");

                    // Backup old config
                    let backup_path = config_path.with_extension("json.backup");
                    fs::copy(&config_path, &backup_path)?;
                    eprintln!("Old config backed up to: {}", backup_path.display());

                    // Try to extract what we can from old config
                    let config = Self::recover_from_old_config(&content)?;
                    config.save()?;
                    eprintln!("New config created with recovered values!");
                    config
                }
            }
        } else {
            // Create default config and save it
            let config = Config::default();
            config.save()?;
            config
        };

        // Resolve relative paths to absolute paths
        config.paths.resolve_paths()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get the path to the config file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .map(|h| h.join(".config"))
            .context("Failed to determine home directory")?;

        Ok(config_dir.join("noter").join("config.json"))
    }

    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let config_file = Self::config_file_path()?;
        Ok(config_file.parent().unwrap().to_path_buf())
    }

    /// Completely remove configuration file and start fresh
    pub fn cleanse() -> Result<()> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            fs::remove_file(&config_path)?;
        }

        Ok(())
    }

    /// Check if config needs migration based on version
    ///
    /// Compares the config's version against the current version constant.
    /// Returns `true` if migration is needed.
    fn needs_migration(config: &Config) -> bool {
        config.metadata.config_version != CONFIG_VERSION
    }

    /// Migrate config from old version to current version
    ///
    /// Applies version-specific migration logic to update the config structure.
    /// Each version transition should have its own migration case that handles
    /// the specific changes made in that version.
    ///
    /// # Developer Notes
    ///
    /// When adding a new version:
    /// 1. Update `CONFIG_VERSION` constant
    /// 2. Add a new match arm for the previous version
    /// 3. Implement migration logic (field renames, type changes, etc.)
    /// 4. Update migration_notes with a description of changes
    fn migrate(mut config: Config) -> Result<Self> {
        let old_version = config.metadata.config_version.as_str();

        // Perform version-specific migrations
        match old_version {
            "" | "0.0.0" => {
                // Migration from initial version (no version tracking)
                config.metadata.migration_notes =
                    "Migrated from initial version to 1.0.0".to_string();
            }
            // Add more version-specific migrations as needed
            // "1.0.0" => {
            //     // Example: rename field, restructure data, etc.
            //     config.metadata.migration_notes =
            //         "Migrated from 1.0.0 to 1.1.0".to_string();
            // }
            v => {
                config.metadata.migration_notes =
                    format!("Migration from version {} to {}", v, CONFIG_VERSION);
            }
        }

        // Update version and timestamp
        config.metadata.config_version = CONFIG_VERSION.to_string();
        config.metadata.last_updated = chrono::Utc::now().to_rfc3339();

        Ok(config)
    }

    /// Recover values from an incompatible old config
    ///
    /// This is a fallback mechanism when serde deserialization completely fails.
    /// It attempts to extract as many user settings as possible from the old
    /// config JSON and applies them to a new default config.
    ///
    /// This method is called when:
    /// - Required fields are missing
    /// - Field types have changed incompatibly
    /// - Struct layout has changed dramatically
    fn recover_from_old_config(content: &str) -> Result<Self> {
        // Parse as generic JSON value to safely extract fields
        let old_value: serde_json::Value = serde_json::from_str(content)?;
        let mut new_config = Config::default();

        // Safely extract old values that are important to preserve
        if let Some(author) = old_value.get("author").and_then(|v| v.as_str()) {
            new_config.author = author.to_string();
        }

        if let Some(editor) = old_value.get("preferred_editor").and_then(|v| v.as_str()) {
            new_config.preferred_editor = Some(editor.to_string());
        }

        if let Some(courses) = old_value.get("courses") {
            if let Ok(courses) = serde_json::from_value(courses.clone()) {
                new_config.courses = courses;
            }
        }

        if let Some(template_version) = old_value.get("template_version").and_then(|v| v.as_str()) {
            new_config.template_version = template_version.to_string();
        }

        // Try to recover nested structs if they're compatible
        if let Some(paths) = old_value.get("paths") {
            if let Ok(paths) = serde_json::from_value(paths.clone()) {
                new_config.paths = paths;
            }
        }

        if let Some(note_prefs) = old_value.get("note_preferences") {
            if let Ok(note_prefs) = serde_json::from_value(note_prefs.clone()) {
                new_config.note_preferences = note_prefs;
            }
        }

        if let Some(typst) = old_value.get("typst") {
            if let Ok(typst) = serde_json::from_value(typst.clone()) {
                new_config.typst = typst;
            }
        }

        if let Some(search) = old_value.get("search") {
            if let Ok(search) = serde_json::from_value(search.clone()) {
                new_config.search = search;
            }
        }

        if let Some(templates) = old_value.get("templates") {
            if let Ok(templates) = serde_json::from_value(templates.clone()) {
                new_config.templates = templates;
            }
        }

        if let Some(obsidian) = old_value.get("obsidian_integration") {
            if let Ok(obsidian) = serde_json::from_value(obsidian.clone()) {
                new_config.obsidian_integration = obsidian;
            }
        }

        // Set metadata for recovered config
        new_config.metadata.migration_notes =
            "Recovered from incompatible config format".to_string();
        new_config.metadata.config_version = CONFIG_VERSION.to_string();
        new_config.metadata.created_at = chrono::Utc::now().to_rfc3339();
        new_config.metadata.last_updated = chrono::Utc::now().to_rfc3339();

        Ok(new_config)
    }
}
