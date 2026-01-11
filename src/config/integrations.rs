use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ObsidianVaultStructure {
    /// Course folder name
    course_folder: String,
    /// Attachments folder name
    attachments_folder: String,
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
