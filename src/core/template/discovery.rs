//! Template discovery and loading
//!
//! Handles finding, loading, and validating template configurations
//! from various sources using a multi-blueprint approach.

use super::config::{TemplateConfig, TemplateDefinition, TemplateVariant};
use super::constants::TOML_FILE_NAME;
use crate::config::Config;
use anyhow::Result;
use semver::Version;
use std::path::{Path, PathBuf};

/// Represents a template that has been discovered and is available for use
#[derive(Debug, Clone)]
pub struct AvailableTemplate {
    /// Template definition from configuration
    pub definition: TemplateDefinition,

    /// Associated variants for this template
    pub variants: Vec<TemplateVariant>,

    /// Path to the template file
    pub file_path: String,

    /// Template source information
    pub source: TemplateSource,

    /// Whether the template is currently accessible
    pub is_accessible: bool,

    /// Metadata about the template package
    pub package_info: Option<TemplatePackageInfo>,
}

/// Information about where a template comes from
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Built-in template (part of the application)
    Builtin,

    /// Local template repository
    Local { path: String },

    /// Remote repository (GitHub, etc.)
    Remote { repository: String, version: String },

    /// User-created custom template
    Custom {
        created_by: String,
        created_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Package metadata for installed templates
#[derive(Debug, Clone)]
pub struct TemplatePackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub install_path: String,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct TemplateDiscovery;

impl TemplateDiscovery {
    /// Load all template configurations from template packages
    pub fn load_template_configs(user_config: &Config) -> Result<Vec<TemplateConfig>> {
        let typst_packages_dir = Path::new(&user_config.paths.typst_packages_dir);

        let template_package_dirs = Self::find_all_template_packages(typst_packages_dir)?;
        let mut configs = Vec::new();

        for package_dir in template_package_dirs {
            let config_path = package_dir.join(TOML_FILE_NAME);

            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let package_config: TemplateConfig = toml::from_str(&content)?;

                configs.push(package_config);
            }
        }

        Ok(configs)
    }
    // TODO: This loads ALL templates, but realistically we should only load the one the user wants as primary.

    /// Backwards compatibility - returns newest config or default
    pub fn load_template_config(user_config: &Config) -> Result<TemplateConfig> {
        let mut configs = Self::load_template_configs(user_config)?;

        if configs.is_empty() {
            return Ok(TemplateConfig::default());
        }

        // Sort by semantic version to get the newest one
        configs.sort_by(|a, b| {
            Self::compare_template_versions(&a.metadata.version, &b.metadata.version)
        });

        // Take the last one (highest version)
        Ok(configs.into_iter().last().unwrap())
    }

    /// Compare semantic versions using the semver crate
    fn compare_template_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let version_a = Version::parse(a).unwrap_or_else(|_| {
            // Fallback for invalid versions - treat as 0.0.0
            Version::new(0, 0, 0)
        });

        let version_b = Version::parse(b).unwrap_or_else(|_| Version::new(0, 0, 0));

        version_a.cmp(&version_b)
    }

    /// Find a specific template across all configs
    pub fn find_template<'a>(
        configs: &'a [TemplateConfig],
        template_name: &str,
    ) -> Option<(&'a TemplateDefinition, &'a TemplateConfig)> {
        for config in configs {
            for template in &config.templates {
                if template.name == template_name {
                    return Some((template, config));
                }
            }
        }
        None
    }

    /// Find variants for a template across all configs
    pub fn find_variants_for_template<'a>(
        configs: &'a [TemplateConfig],
        template_name: &str,
    ) -> Vec<&'a TemplateVariant> {
        let mut variants = Vec::new();

        for config in configs {
            if let Some(config_variants) = &config.variants {
                variants.extend(
                    config_variants
                        .iter()
                        .filter(|v| v.template == template_name),
                );
            }
        }

        variants
    }

    /// Get all available templates from all configs
    pub fn get_all_templates(
        configs: &[TemplateConfig],
    ) -> Vec<(&TemplateDefinition, &TemplateConfig)> {
        let mut all_templates = Vec::new();

        for config in configs {
            for template in &config.templates {
                all_templates.push((template, config));
            }
        }

        all_templates
    }

    /// Find template with specific name and optional source package preference
    pub fn find_template_with_preference<'a>(
        configs: &'a [TemplateConfig],
        template_name: &str,
        preferred_package: Option<&str>,
    ) -> Option<(&'a TemplateDefinition, &'a TemplateConfig)> {
        // First try to find in preferred package if specified
        if let Some(package_name) = preferred_package {
            for config in configs {
                if config.metadata.name == package_name {
                    for template in &config.templates {
                        if template.name == template_name {
                            return Some((template, config));
                        }
                    }
                }
            }
        }

        // Fallback to any package with that template
        Self::find_template(configs, template_name)
    }

    /// Discover all available templates
    pub fn discover_templates(user_config: &Config) -> Result<Vec<AvailableTemplate>> {
        let configs = Self::load_template_configs(user_config)?;
        let mut available_templates = Vec::new();

        for config in &configs {
            let package_dir = Self::find_package_directory_for_config(user_config, config)?;

            for template_def in &config.templates {
                let variants = Self::find_variants_for_template(&configs, &template_def.name)
                    .into_iter()
                    .cloned()
                    .collect();

                let file_path = package_dir.join(&template_def.file);

                let available_template = AvailableTemplate {
                    definition: template_def.clone(),
                    variants,
                    file_path: file_path.to_string_lossy().to_string(),
                    source: TemplateSource::Local {
                        path: package_dir.to_string_lossy().to_string(),
                    },
                    is_accessible: file_path.exists(),
                    package_info: Some(Self::extract_package_info(config, &package_dir)),
                };

                available_templates.push(available_template);
            }
        }

        Ok(available_templates)
    }

    /// Find all template packages in the typst packages directory
    fn find_all_template_packages(typst_packages_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut template_packages = Vec::new();

        if !typst_packages_dir.exists() {
            return Ok(template_packages);
        }

        // Look for directories that contain .noter.config.toml files
        for entry in std::fs::read_dir(typst_packages_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let config_file = path.join(TOML_FILE_NAME);
                if config_file.exists() {
                    template_packages.push(path.clone());
                }

                // Also check version subdirectories (like dtu-template/0.2.0/)
                if let Ok(version_dirs) = std::fs::read_dir(&path) {
                    for version_entry in version_dirs.flatten() {
                        let version_path = version_entry.path();
                        if version_path.is_dir() {
                            let version_config = version_path.join(TOML_FILE_NAME);
                            if version_config.exists() {
                                template_packages.push(version_path);
                            }
                        }
                    }
                }
            }
        }

        Ok(template_packages)
    }

    /// Find the package directory for a specific config (for file resolution)
    fn find_package_directory_for_config(
        user_config: &Config,
        target_config: &TemplateConfig,
    ) -> Result<PathBuf> {
        let typst_packages_dir = Path::new(&user_config.paths.typst_packages_dir);
        let package_dirs = Self::find_all_template_packages(typst_packages_dir)?;

        // Find the directory that contains this specific config
        for package_dir in package_dirs.clone() {
            let config_path = package_dir.join(TOML_FILE_NAME);
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                if let Ok(config) = toml::from_str::<TemplateConfig>(&content) {
                    if config.metadata.name == target_config.metadata.name
                        && config.metadata.version == target_config.metadata.version
                    {
                        return Ok(package_dir);
                    }
                }
            }
        }

        // Fallback: return first package directory if exact match not found
        package_dirs
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No template packages found"))
    }

    /// Extract package information from config and directory
    fn extract_package_info(config: &TemplateConfig, package_dir: &PathBuf) -> TemplatePackageInfo {
        TemplatePackageInfo {
            name: config.metadata.name.clone(),
            version: config.metadata.version.clone(),
            description: config.metadata.description.clone(),
            author: config.metadata.author.clone(),
            license: config.metadata.license.clone(),
            install_path: package_dir.to_string_lossy().to_string(),
            last_updated: std::fs::metadata(package_dir)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|system_time| {
                    chrono::DateTime::from_timestamp(
                        system_time
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                        0,
                    )
                }),
        }
    }

    pub fn get_all_variants(configs: &[TemplateConfig]) -> Vec<TemplateVariant> {
        configs
            .iter()
            .flat_map(|c| c.variants.iter().flatten())
            .cloned()
            .collect()
    }

    /// Get course type from any available course mapping in configs
    pub fn resolve_course_type(
        configs: &[TemplateConfig],
        course_id: &str,
        fallback: &str,
    ) -> String {
        // Try each config's course mapping
        for config in configs.iter() {
            if let Some(course_mapping) = &config.course_mapping {
                // Check for exact course ID match first
                if let Some(mapped_type) = course_mapping.get(course_id) {
                    return mapped_type.clone();
                }

                // Then check for pattern matches (like "01xxx")
                for (pattern, course_type) in course_mapping {
                    if Self::matches_course_pattern(course_id, pattern) {
                        return course_type.clone();
                    }
                }
            }
        }

        // Fallback to provided default
        fallback.to_string()
    }

    /// Simple pattern matching for course IDs (like "01xxx" matches "01005")
    fn matches_course_pattern(course_id: &str, pattern: &str) -> bool {
        if course_id.len() != pattern.len() {
            return false;
        }

        course_id
            .chars()
            .zip(pattern.chars())
            .all(|(c, p)| p == 'x' || p == 'X' || p == c)
    }

    /// Get import statement for a specific config
    pub fn get_import_statement(config: &TemplateConfig) -> String {
        let package_name = &config.metadata.name;
        let version = &config.metadata.version;
        format!("#import \"@local/{}:{}\":*", package_name, version)
    }

    /// Find best variant for a template given course type
    pub fn find_best_variant(
        configs: &[TemplateConfig],
        template_name: &str,
        course_type: &str,
    ) -> Option<TemplateVariant> {
        let variants = Self::find_variants_for_template(configs, template_name);

        // Filter variants that match the course type
        let matching_variants: Vec<_> = variants
            .into_iter()
            .filter(|variant| {
                variant.course_types.contains(&course_type.to_string())
                    || variant.course_types.contains(&"all".to_string())
            })
            .collect();

        // Return the first matching variant
        matching_variants.first().map(|v| (*v).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_load_template_configs() -> Result<()> {
        let config = Config::default();

        match TemplateDiscovery::load_template_configs(&config) {
            Ok(configs) => {
                println!("✅ Loaded {} template configurations", configs.len());

                for (i, template_config) in configs.iter().enumerate() {
                    println!(
                        "📦 Package {}: {} v{}",
                        i + 1,
                        template_config.metadata.name,
                        template_config.metadata.version
                    );
                    println!("   Templates: {}", template_config.templates.len());

                    for template in &template_config.templates {
                        println!("     - {} ({})", template.display_name, template.name);
                    }

                    if let Some(variants) = &template_config.variants {
                        println!("   Variants: {}", variants.len());
                        for variant in variants {
                            println!("     - {} for {}", variant.display_name, variant.template);
                        }
                    }
                }

                // Test finding a template across all configs
                if let Some((template, source_config)) =
                    TemplateDiscovery::find_template(&configs, "lecture-note")
                {
                    println!(
                        "✅ Found template 'lecture-note' in package '{}'",
                        source_config.metadata.name
                    );
                    println!("   Function: {}", template.function);
                    println!("   File: {}", template.file);
                }
            }
            Err(e) => {
                println!("⚠️  Could not load template configs: {}", e);
                println!("💡 This is expected if no template packages are installed");
            }
        }

        Ok(())
    }

    #[test]
    fn test_discover_templates() -> Result<()> {
        let config = Config::default();

        match TemplateDiscovery::discover_templates(&config) {
            Ok(available_templates) => {
                println!("✅ Discovered {} templates", available_templates.len());

                for template in &available_templates {
                    println!("📄 {}", template.definition.display_name);
                    println!("   Name: {}", template.definition.name);
                    println!("   File: {}", template.file_path);
                    println!("   Accessible: {}", template.is_accessible);
                    println!("   Variants: {}", template.variants.len());

                    if let Some(ref package_info) = template.package_info {
                        println!(
                            "   Package: {} v{}",
                            package_info.name, package_info.version
                        );
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("⚠️  Template discovery failed: {}", e);
            }
        }

        Ok(())
    }

    #[test]
    fn test_course_type_resolution() {
        use super::super::config::{TemplateConfig, TemplateMetadata};
        use std::collections::HashMap;

        // Create mock configs with different course mappings
        let mut course_mapping1 = HashMap::new();
        course_mapping1.insert("01xxx".to_string(), "math".to_string());
        course_mapping1.insert("02xxx".to_string(), "programming".to_string());

        let mut course_mapping2 = HashMap::new();
        course_mapping2.insert("25xxx".to_string(), "physics".to_string());

        let config1 = TemplateConfig {
            metadata: TemplateMetadata {
                name: "config1".to_string(),
                version: "1.0".to_string(),
                description: None,
                repository: None,
                author: None,
                license: None,
            },
            templates: vec![],
            variants: None,
            course_mapping: Some(course_mapping1),
            engine: None,
        };

        let config2 = TemplateConfig {
            metadata: TemplateMetadata {
                name: "config2".to_string(),
                version: "1.0".to_string(),
                description: None,
                repository: None,
                author: None,
                license: None,
            },
            templates: vec![],
            variants: None,
            course_mapping: Some(course_mapping2),
            engine: None,
        };

        let configs = vec![config1, config2];

        // Test course type resolution
        assert_eq!(
            TemplateDiscovery::resolve_course_type(&configs, "01005", "unknown"),
            "math"
        );
        assert_eq!(
            TemplateDiscovery::resolve_course_type(&configs, "02101", "unknown"),
            "programming"
        );
        assert_eq!(
            TemplateDiscovery::resolve_course_type(&configs, "25200", "unknown"),
            "physics"
        );
        assert_eq!(
            TemplateDiscovery::resolve_course_type(&configs, "99999", "unknown"),
            "unknown"
        );

        println!("✅ Course type resolution works correctly");
    }

    #[test]
    fn test_pattern_matching() {
        assert!(TemplateDiscovery::matches_course_pattern("01005", "01xxx"));
        assert!(TemplateDiscovery::matches_course_pattern("02101", "02XXX"));
        assert!(!TemplateDiscovery::matches_course_pattern("01005", "02xxx"));
        assert!(!TemplateDiscovery::matches_course_pattern("1005", "01xxx")); // Wrong length

        println!("✅ Pattern matching works correctly");
    }
}
