use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Current config version - increment when making breaking changes
///
/// When you modify the Config struct in a breaking way (rename fields, change types, etc.),
/// increment this version and add corresponding migration logic in `migrate()`.
///
/// Version history:
/// - `1.0.0`: Initial versioned config with automatic migration system
const CONFIG_VERSION: &str = "1.0.0";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// User's name for templates
    pub author: String,

    /// Preferred text editor
    pub preferred_editor: Option<String>,

    /// DTU template version to use
    pub template_version: String,

    /// Semester format preference
    pub semester_format: SemesterFormat,

    /// Default note structure preferences
    pub note_preferences: NotePreferences,

    /// Paths configuration
    pub paths: PathConfig,

    /// Template source configuration
    pub templates: UserTemplateConfig,

    /// Typst compilation settings
    pub typst: TypstConfig,

    /// Search preferences
    pub search: SearchConfig,

    /// User's DTU courses
    pub courses: std::collections::HashMap<String, String>,

    /// Obsidian integration settings
    pub obsidian_integration: ObsidianIntegrationConfig,

    /// Metadata (Not used by user)
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub config_version: String,
    pub created_at: String,
    pub last_updated: String,
    pub migration_notes: String,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            config_version: CONFIG_VERSION.to_string(),
            created_at: now.clone(),
            last_updated: now,
            migration_notes: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NotePreferences {
    /// Whether to automatically open files after creation
    pub auto_open_file: bool,

    /// Whether to open the file directory after creation
    pub auto_open_dir: bool,

    /// Include date in lecture note titles
    pub include_date_in_title: bool,

    /// Default sections for lecture notes
    pub lecture_sections: Vec<String>,

    /// Default sections for assignments
    pub assignment_sections: Vec<String>,

    /// Whether to create backup of existing files
    pub create_backups: bool,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UserTemplateConfig {
    /// Custom template repositories (user-defined)
    pub custom_repositories: Vec<TemplateRepository>,

    /// Fallback to official DTU template if custom templates fail
    pub use_official_fallback: bool,

    /// Cache templates locally for faster access
    pub enable_caching: bool,

    /// Auto-update templates on startup
    pub auto_update: bool,

    /// Template preference order (repository names)
    pub preference_order: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TemplateRepository {
    /// Display name for the repository
    pub name: String,

    /// GitHub repository in format "owner/repo"
    pub repository: String,

    /// Specific version/tag to use (None for latest)
    pub version: Option<String>,

    /// Branch to use if not using releases
    pub branch: Option<String>,

    /// Subdirectory within the repo containing templates
    pub template_path: Option<String>,

    /// Whether this repository is enabled
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ObsidianVaultStructure {
    /// Course folder name
    course_folder: String,
    /// Attachments folder name
    attachments_folder: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ObsidianIntegrationConfig {
    /// Whether Obsidian integration is enabled
    pub enabled: bool,
    /// Create course index
    pub create_course_index: bool,
    /// Create daily notes (currently unused)
    pub create_daily_notes: bool,
    /// Vault structure (currently unused)
    pub vault_structure: Option<ObsidianVaultStructure>,
    /// Link format
    pub link_format: String,
    /// Tag format
    pub tag_format: String,
}

impl Default for ObsidianVaultStructure {
    fn default() -> Self {
        Self {
            course_folder: "Courses".to_string(),
            attachments_folder: "attachments".to_string(),
        }
    }
}

impl Default for ObsidianIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            create_course_index: true,
            create_daily_notes: false,
            vault_structure: None,
            link_format: "wiki".into(),
            tag_format: "#course/{{course_id}}".into(),
        }
    }
}

impl Default for TemplateRepository {
    fn default() -> Self {
        Self {
            name: String::new(),
            repository: String::new(),
            version: None,
            branch: None,
            template_path: None,
            enabled: true,
        }
    }
}

impl Default for UserTemplateConfig {
    fn default() -> Self {
        Self {
            custom_repositories: Vec::new(),
            use_official_fallback: true,
            enable_caching: true,
            auto_update: false,
            preference_order: vec!["official".to_string()],
        }
    }
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

    fn resolve_path(path: &str, base: &std::path::Path) -> Result<String> {
        let path_buf = if std::path::Path::new(path).is_absolute() {
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TypstConfig {
    /// Additional compile arguments
    pub compile_args: Vec<String>,

    /// Watch mode arguments
    pub watch_args: Vec<String>,

    /// Whether to clean PDFs before compiling
    pub clean_before_compile: bool,

    /// Output directory for PDFs (relative to source)
    pub output_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SearchConfig {
    /// Maximum number of search results to show
    pub max_results: usize,

    /// Include file context lines around matches
    pub context_lines: usize,

    /// Case sensitive search
    pub case_sensitive: bool,

    /// File extensions to search in
    pub file_extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SemesterFormat {
    /// "2024 Spring", "2024 Fall"
    YearSeason,
    /// "Spring 2024", "Fall 2024"
    SeasonYear,
    /// "S24", "F24"
    ShortForm,
    /// Custom format string
    Custom(String),
}

impl Default for Config {
    fn default() -> Self {
        // Create default courses
        let mut default_courses = std::collections::HashMap::new();

        // Add some common DTU courses as defaults
        let common_courses = [
            ("01005", "Advanced Engineering Mathematics 1"),
            ("01006", "Advanced Engineering Mathematics 2"),
            ("01017", "Discrete Mathematics"),
            ("02101", "Introduction to Programming"),
            ("02102", "Algorithms and Data Structures"),
            ("25200", "Classical Physics 1"),
            ("22100", "Electronics 1"),
        ];

        for (id, name) in common_courses {
            default_courses.insert(id.to_string(), name.to_string());
        }

        Self {
            author: "Your Name".to_string(),
            preferred_editor: None,
            template_version: env!("CARGO_PKG_VERSION").to_string(),
            semester_format: SemesterFormat::YearSeason,
            note_preferences: NotePreferences::default(),
            paths: PathConfig::default(),
            templates: UserTemplateConfig::default(),
            typst: TypstConfig::default(),
            search: SearchConfig::default(),
            courses: default_courses,
            obsidian_integration: ObsidianIntegrationConfig::default(),
            metadata: Metadata::default(),
        }
    }
}

impl Default for NotePreferences {
    fn default() -> Self {
        Self {
            auto_open_file: true,
            auto_open_dir: false,
            include_date_in_title: true,
            lecture_sections: vec![
                "Key Concepts".to_string(),
                "Mathematical Framework".to_string(),
                "Examples".to_string(),
                "Important Points".to_string(),
                "Questions & Follow-up".to_string(),
                "Connections to Previous Material".to_string(),
                "Next Class Preview".to_string(),
            ],
            assignment_sections: vec![
                "Problem 1".to_string(),
                "Problem 2".to_string(),
                "Problem 3".to_string(),
            ],
            create_backups: false,
        }
    }
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
                        eprintln!("⚠️  Config format has changed. Migrating...");
                        config = Self::migrate(config)?;
                        config.save()?;
                        eprintln!("✓ Config migrated successfully!");
                    }
                    config
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load config: {}", e);
                    eprintln!("Creating backup and recovering values from old config...");

                    // Backup old config
                    let backup_path = config_path.with_extension("json.backup");
                    fs::copy(&config_path, &backup_path)?;
                    eprintln!("Old config backed up to: {}", backup_path.display());

                    // Try to extract what we can from old config
                    let config = Self::recover_from_old_config(&content)?;
                    config.save()?;
                    eprintln!("✓ New config created with recovered values!");
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

    /// Check if config needs migration based on version
    ///
    /// Compares the config's version against the current version constant.
    /// Returns `true` if migration is needed.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to check
    fn needs_migration(config: &Config) -> bool {
        config.metadata.config_version != CONFIG_VERSION
    }

    /// Migrate config from old version to current version
    ///
    /// Applies version-specific migration logic to update the config structure.
    /// Each version transition should have its own migration case that handles
    /// the specific changes made in that version.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to migrate
    ///
    /// # Returns
    ///
    /// Returns the migrated configuration with updated version and timestamp.
    ///
    /// # Developer Notes
    ///
    /// When adding a new version:
    /// 1. Update `CURRENT_VERSION` constant
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
    ///
    /// # Arguments
    ///
    /// * `content` - The raw JSON string of the old config
    ///
    /// # Returns
    ///
    /// Returns a new default config with as many old values recovered as possible.
    ///
    /// # Developer Notes
    ///
    /// When adding new important fields that should be preserved during recovery,
    /// add extraction logic here similar to existing fields.
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

    /// Completely remove configuration file and start fresh
    pub fn cleanse() -> Result<()> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            fs::remove_file(&config_path)?;
        }

        Ok(())
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

        Ok(config_dir.join("dtu-notes").join("config.json"))
    }

    /// Get the config directory path
    #[allow(dead_code)]
    pub fn config_dir() -> Result<PathBuf> {
        let config_file = Self::config_file_path()?;
        Ok(config_file.parent().unwrap().to_path_buf())
    }

    /// Update author name
    pub fn set_author(&mut self, author: String) -> Result<()> {
        self.author = author;
        self.save()
    }

    /// Update preferred editor
    pub fn set_editor(&mut self, editor: Option<String>) -> Result<()> {
        self.preferred_editor = editor;
        self.save()
    }

    /// Get formatted semester string
    pub fn format_semester(&self, year: i32, is_spring: bool) -> String {
        match &self.semester_format {
            SemesterFormat::YearSeason => {
                format!("{} {}", year, if is_spring { "Spring" } else { "Fall" })
            }
            SemesterFormat::SeasonYear => {
                format!("{} {}", if is_spring { "Spring" } else { "Fall" }, year)
            }
            SemesterFormat::ShortForm => {
                format!("{}{}", if is_spring { "S" } else { "F" }, year % 100)
            }
            SemesterFormat::Custom(format) => format
                .replace("{year}", &year.to_string())
                .replace("{season}", if is_spring { "Spring" } else { "Fall" })
                .replace("{s}", if is_spring { "S" } else { "F" })
                .replace("{yy}", &format!("{:02}", year % 100)),
        }
    }

    /// Add a course
    pub fn add_course(&mut self, course_id: String, course_name: String) -> Result<()> {
        self.courses.insert(course_id, course_name);
        self.save()
    }

    /// Remove a course
    pub fn remove_course(&mut self, course_id: &str) -> Result<bool> {
        let removed = self.courses.remove(course_id).is_some();
        self.save()?;
        Ok(removed)
    }

    /// Get course name
    pub fn get_course_name(&self, course_id: &str) -> String {
        self.courses.get(course_id).cloned().unwrap_or_default()
    }

    /// List all courses
    pub fn list_courses(&self) -> Vec<(String, String)> {
        let mut courses: Vec<(String, String)> = self
            .courses
            .iter()
            .map(|(id, name)| (id.clone(), name.clone()))
            .collect();
        courses.sort_by(|a, b| a.0.cmp(&b.0));
        courses
    }

    /// Get list of preferred editors in order
    pub fn get_editor_list(&self) -> Vec<String> {
        let mut editors = Vec::new();

        // Add preferred editor first if set
        if let Some(ref preferred) = self.preferred_editor {
            editors.push(preferred.clone());
        }

        // Add default editors based on OS
        if cfg!(windows) {
            editors.extend(["code", "notepad"].iter().map(|s| s.to_string()));
        } else {
            editors.extend(
                ["code", "nvim", "vim", "nano"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }

        // Remove duplicates while preserving order
        let mut unique_editors = Vec::new();
        for editor in editors {
            if !unique_editors.contains(&editor) {
                unique_editors.push(editor);
            }
        }

        unique_editors
    }

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
        if !std::path::Path::new(&self.paths.templates_dir).exists() {
            warnings.push(format!(
                "Template directory '{}' doesn't exist",
                self.paths.templates_dir
            ));
        }

        Ok(warnings)
    }
}

/// Helper functions for other modules to use
pub fn get_config() -> Result<Config> {
    Config::load()
}

pub fn update_author(new_author: String) -> Result<()> {
    let mut config = Config::load()?;
    config.set_author(new_author)
}

pub fn update_editor(new_editor: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    config.set_editor(new_editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.author, "Your Name");
        assert!(config.note_preferences.auto_open_file);
    }

    #[test]
    fn test_semester_formatting() {
        let config = Config::default();
        assert_eq!(config.format_semester(2024, true), "2024 Spring");
        assert_eq!(config.format_semester(2024, false), "2024 Fall");
    }

    #[test]
    fn test_editor_list() {
        let mut config = Config::default();
        config.preferred_editor = Some("emacs".to_string());

        let editors = config.get_editor_list();
        assert_eq!(editors[0], "emacs");
    }

    #[test]
    fn test_config_file_path() {
        let config_path = Config::config_file_path().unwrap();
        assert!(config_path.ends_with("config.json"));
    }
}
