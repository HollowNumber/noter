use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Current config version - increment when making breaking changes
///
/// When you modify the Config struct in a breaking way (rename fields, change types, etc.),
/// increment this version and add corresponding migration logic in `migrate()`.
///
/// Version history:
/// - `1.0.0`: Initial versioned config with automatic migration system
pub const CONFIG_VERSION: &str = "1.0.0";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub config_version: String,
    pub created_at: String,
    pub last_updated: String,
    pub migration_notes: String,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            config_version: CONFIG_VERSION.to_string(),
            created_at: now.clone(),
            last_updated: now,
            migration_notes: String::new(),
        }
    }
}
