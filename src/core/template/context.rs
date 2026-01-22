//! Template context management and construction
//!
//! This module handles the creation and management of template contexts,
//! which contain all the metadata and variables needed for template generation.
//! It provides specialized context builders for different template types and
//! integrates with the new template configuration system.

use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;

use super::config::{EngineConfig, TemplateConfig};
use crate::config::Config;
use crate::core::status::StatusManager;

/// Rich context structure containing all metadata needed for template generation.
///
/// This structure encapsulates all the information required to generate a complete
/// Typst document, including course details, author information, template configuration,
/// and customizable sections.
///
/// ## Extended Fields
///
/// - `template_config`: Template package configuration loaded from `.noter-config.toml`
/// - `engine_config`: Engine capabilities and processing rules
/// - `template_dir`: Path to the template directory for resolving includes
/// - `variables`: Dynamic variables for template substitution
/// - `metadata`: Additional metadata for template processing
#[derive(Debug, Clone)]
pub struct TemplateContext {
    // Core template data
    pub course_id: String,
    pub course_name: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub semester: String,
    pub template_version: String,
    pub sections: Vec<String>,
    pub custom_fields: HashMap<String, String>,

    // Enhanced template system fields
    pub template_config: Option<TemplateConfig>,
    pub engine_config: EngineConfig,
    pub template_dir: String,
    pub variables: HashMap<String, String>,
    pub metadata: TemplateMetadata,
}

/// Additional metadata for template processing
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    pub course_type: String,
    pub assignment_type: Option<String>,
    pub creation_date: chrono::DateTime<Local>,
    pub template_source: String,
    pub variant_used: Option<String>,
    pub processing_flags: Vec<String>,
}

impl TemplateContext {
    /// Create a new context builder
    pub fn builder() -> TemplateContextBuilder {
        TemplateContextBuilder::new()
    }

    /// Build lecture context with template configuration support
    pub fn build_lecture_context(
        course_id: &str,
        config: &Config,
        template_config: &TemplateConfig,
        custom_title: Option<&str>,
    ) -> Result<Self> {
        let course_name = Self::resolve_course_name(course_id, config);
        let semester = StatusManager::get_current_semester(config);
        let course_type = Self::determine_course_type(course_id);

        let title = if let Some(custom_title) = custom_title {
            custom_title.to_string()
        } else {
            let date = Local::now();
            if config.note_preferences.include_date_in_title {
                format!("Lecture - {}", date.format("%B %d, %Y"))
            } else {
                "Lecture Notes".to_string()
            }
        };

        let engine_config = template_config.engine.clone().unwrap_or_default();
        let variables = Self::build_builtin_variables(course_id, &title, &config.author, &semester);

        Ok(Self {
            course_id: course_id.to_string(),
            course_name,
            title,
            author: config.author.clone(),
            date: Local::now().format("%Y-%m-%d").to_string(),
            semester,
            template_version: config.template_version.clone(),
            sections: config.note_preferences.lecture_sections.clone(),
            custom_fields: HashMap::new(),
            template_config: Some(template_config.clone()),
            engine_config,
            template_dir: config.paths.templates_dir.clone(),
            variables,
            metadata: TemplateMetadata {
                course_type,
                assignment_type: None,
                creation_date: Local::now(),
                template_source: "builtin".to_string(),
                variant_used: None,
                processing_flags: vec![],
            },
        })
    }

    /// Build assignment context with template configuration support
    pub fn build_assignment_context(
        course_id: &str,
        assignment_title: &str,
        config: &Config,
        template_config: &TemplateConfig,
    ) -> Result<Self> {
        use super::discovery::TemplateDiscovery;

        let course_name = Self::resolve_course_name(course_id, config);
        let semester = StatusManager::get_current_semester(config);
        let course_type = TemplateDiscovery::resolve_course_type(
            std::slice::from_ref(template_config),
            course_id,
            "general",
        );
        let assignment_type = Self::determine_assignment_type(assignment_title);

        let engine_config = template_config.engine.clone().unwrap_or_default();
        let variables =
            Self::build_builtin_variables(course_id, assignment_title, &config.author, &semester);

        Ok(Self {
            course_id: course_id.to_string(),
            course_name,
            title: assignment_title.to_string(),
            author: config.author.clone(),
            date: Local::now().format("%Y-%m-%d").to_string(),
            semester,
            template_version: config.template_version.clone(),
            sections: config.note_preferences.assignment_sections.clone(),
            custom_fields: HashMap::new(),
            template_config: Some(template_config.clone()),
            engine_config,
            template_dir: config.paths.templates_dir.clone(),
            variables,
            metadata: TemplateMetadata {
                course_type,
                assignment_type: Some(assignment_type),
                creation_date: Local::now(),
                template_source: "builtin".to_string(),
                variant_used: None,
                processing_flags: vec![],
            },
        })
    }

    /// Build custom context for builder pattern
    pub fn build_custom_context(
        course_id: &str,
        config: &Config,
        template_config: &TemplateConfig,
    ) -> Result<Self> {
        use super::discovery::TemplateDiscovery;

        let course_name = Self::resolve_course_name(course_id, config);
        let semester = StatusManager::get_current_semester(config);
        let course_type = TemplateDiscovery::resolve_course_type(
            std::slice::from_ref(template_config),
            course_id,
            "general",
        );

        let engine_config = template_config.engine.clone().unwrap_or_default();
        let variables = Self::build_builtin_variables(course_id, "", &config.author, &semester);

        Ok(Self {
            course_id: course_id.to_string(),
            course_name,
            title: String::new(),
            author: config.author.clone(),
            date: Local::now().format("%Y-%m-%d").to_string(),
            semester,
            template_version: config.template_version.clone(),
            sections: Vec::new(),
            custom_fields: HashMap::new(),
            template_config: Some(template_config.clone()),
            engine_config,
            template_dir: config.paths.templates_dir.clone(),
            variables,
            metadata: TemplateMetadata {
                course_type,
                assignment_type: None,
                creation_date: Local::now(),
                template_source: "custom".to_string(),
                variant_used: None,
                processing_flags: vec![],
            },
        })
    }

    /// Add or update a template variable
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Get a template variable with fallback
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Apply variable transformations based on engine config
    pub fn apply_transformations(&mut self) -> Result<()> {
        // Apply transformations defined in engine config
        for _transformation in &self.engine_config.variables.transformations {
            // Implementation for applying transformations
            // This would handle things like uppercase, lowercase, date formatting, etc.
        }
        Ok(())
    }

    /// Validate context against engine requirements
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Basic validation
        if self.author.is_empty() {
            warnings.push("Author name is empty".to_string());
        }

        if self.course_name.is_empty() {
            warnings.push(format!("Course name not found for {}", self.course_id));
        }

        // Engine-specific validation
        if self.engine_config.validation.validate_variables {
            for required_var in &self.engine_config.variables.builtin_variables {
                if !self.variables.contains_key(required_var) {
                    warnings.push(format!("Required variable '{}' is missing", required_var));
                }
            }
        }

        Ok(warnings)
    }

    // Helper methods
    fn resolve_course_name(course_id: &str, config: &Config) -> String {
        config.get_course_name(course_id)
    }

    fn determine_course_type(course_id: &str) -> String {
        match course_id {
            code if code.starts_with("01") => "math".to_string(),
            code if code.starts_with("02") => "programming".to_string(),
            code if code.starts_with("25") => "physics".to_string(),
            code if code.starts_with("22") => "electronics".to_string(),
            code if code.starts_with("28") => "environment".to_string(),
            code if code.starts_with("31") => "mechanics".to_string(),
            _ => "general".to_string(),
        }
    }

    fn determine_assignment_type(title: &str) -> String {
        let title_lower = title.to_lowercase();

        if title_lower.contains("programming") || title_lower.contains("code") {
            "programming".to_string()
        } else if title_lower.contains("analysis") || title_lower.contains("research") {
            "research".to_string()
        } else if title_lower.contains("problem") || title_lower.contains("exercise") {
            "theoretical".to_string()
        } else if title_lower.contains("lab") || title_lower.contains("experiment") {
            "practical".to_string()
        } else {
            "general".to_string()
        }
    }

    fn build_builtin_variables(
        course_id: &str,
        title: &str,
        author: &str,
        semester: &str,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        variables.insert("course_id".to_string(), course_id.to_string());
        variables.insert("title".to_string(), title.to_string());
        variables.insert("author".to_string(), author.to_string());
        variables.insert("semester".to_string(), semester.to_string());
        variables.insert(
            "date".to_string(),
            Local::now().format("%Y-%m-%d").to_string(),
        );
        variables.insert("year".to_string(), Local::now().format("%Y").to_string());

        variables
    }
}

impl Default for TemplateMetadata {
    fn default() -> Self {
        Self {
            course_type: "general".to_string(),
            assignment_type: None,
            creation_date: Local::now(),
            template_source: "unknown".to_string(),
            variant_used: None,
            processing_flags: vec![],
        }
    }
}

/// Builder for creating template contexts with fluent API
#[derive(Clone, Debug)]
pub struct TemplateContextBuilder {
    course_id: Option<String>,
    config: Option<Config>,
    template_config: Option<TemplateConfig>,
    title: Option<String>,
    custom_fields: HashMap<String, String>,
    sections: Option<Vec<String>>,
    variables: HashMap<String, String>,
}

impl TemplateContextBuilder {
    pub fn new() -> Self {
        Self {
            course_id: None,
            config: None,
            template_config: None,
            title: None,
            custom_fields: HashMap::new(),
            sections: None,
            variables: HashMap::new(),
        }
    }

    pub fn get_config(&self) -> Option<&Config> {
        self.config.as_ref()
    }

    pub fn with_course_id(mut self, course_id: &str) -> Self {
        self.course_id = Some(course_id.to_string());
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_template_config(mut self, template_config: TemplateConfig) -> Self {
        self.template_config = Some(template_config);
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_sections(mut self, sections: Vec<String>) -> Self {
        self.sections = Some(sections);
        self
    }

    pub fn with_variable(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_custom_field(mut self, key: &str, value: &str) -> Self {
        self.custom_fields
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> Result<TemplateContext> {
        let course_id = self
            .course_id
            .ok_or_else(|| anyhow::anyhow!("Course ID is required"))?;
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config is required"))?;
        let template_config = self.template_config.unwrap_or_default();

        let mut context =
            TemplateContext::build_custom_context(&course_id, &config, &template_config)?;

        if let Some(title) = self.title {
            context.title = title;
        }

        if let Some(sections) = self.sections {
            context.sections = sections;
        }

        // Merge custom fields and variables
        context.custom_fields.extend(self.custom_fields);
        context.variables.extend(self.variables);

        Ok(context)
    }
}

impl Default for TemplateContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            metadata: super::config::TemplateMetadata {
                name: "".to_string(),
                version: "".to_string(),
                description: None,
                repository: None,
                author: None,
                license: None,
            },
            templates: vec![],
            variants: None,
            course_mapping: None,
            engine: None,
        }
    }
}
