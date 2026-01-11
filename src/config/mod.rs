//! Configuration management
//!
//! This module handles all configuration aspects including loading, saving,
//! validation, and migration. Configuration is split across multiple submodules
//! for better organization.

pub mod integrations;
pub mod metadata;
pub mod paths;
pub mod persistence;
pub mod preferences;
pub mod search;
pub mod semester;
pub mod templates;
pub mod typst;
pub mod validation;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export commonly used types
pub use integrations::ObsidianIntegrationConfig;
pub use metadata::Metadata;
pub use paths::PathConfig;
pub use preferences::NotePreferences;
pub use search::SearchConfig;
pub use semester::SemesterFormat;
pub use templates::{TemplateRepository, UserTemplateConfig};
pub use typst::TypstConfig;

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
    pub courses: HashMap<String, String>,

    /// Obsidian integration settings
    pub obsidian_integration: ObsidianIntegrationConfig,

    /// Metadata (Not used by user)
    pub metadata: Metadata,
}

impl Default for Config {
    fn default() -> Self {
        // Create default courses
        let mut default_courses = HashMap::new();

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
            semester_format: SemesterFormat::default(),
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

impl Config {
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
                .replace("{}", &year.to_string())
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
}

/// Helper function to load config
pub fn get_config() -> Result<Config> {
    Config::load()
}

/// Helper function to update author
pub fn update_author(new_author: String) -> Result<()> {
    let mut config = Config::load()?;
    config.set_author(new_author)
}

/// Helper function to update editor
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
