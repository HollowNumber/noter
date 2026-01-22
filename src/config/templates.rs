use serde::{Deserialize, Serialize};

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
