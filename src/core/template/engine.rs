//! Core template engine implementation
//!
//! The main TemplateEngine that handles template generation,
//! discovery, and rendering.

use super::config::{TemplateConfig, TemplateDefinition, TemplateVariant};
use super::context::TemplateContext;
use super::discovery::TemplateDiscovery;
use crate::config::Config;
use anyhow::{Result, anyhow};
use chrono::{Datelike, Local};

pub struct TemplateEngine;

impl TemplateEngine {
    /// Generate a lecture template
    pub fn generate_lecture_template(
        course_id: &str,
        config: &Config,
        custom_title: Option<&str>,
    ) -> Result<String> {
        let template_config = TemplateDiscovery::load_template_config(config)?;
        let context = TemplateContext::build_lecture_context(
            course_id,
            config,
            &template_config,
            custom_title,
        )?;

        let template_ref = TemplateReference::lecture();
        Self::render_template(&context, &template_ref)
    }

    /// Main template rendering function
    pub fn render_template(
        context: &TemplateContext,
        template_ref: &TemplateReference,
    ) -> Result<String> {
        // Get the template definition based on reference
        let template_def = Self::get_template_definition(context, template_ref)?;

        // Select best variant if supported, or use the specified variant
        let variant = Self::select_variant_for_template(context, &template_def, template_ref)?;

        // Generate the complete Typst document
        Self::generate_typst_document(context, &template_def, variant.as_ref())
    }

    /// Generate the complete Typst document
    fn generate_typst_document(
        context: &TemplateContext,
        template_def: &TemplateDefinition,
        variant: Option<&TemplateVariant>,
    ) -> Result<String> {
        let mut document = String::new();

        // Generate import statement
        document.push_str(&Self::generate_import_statement(context)?);
        document.push_str("\n\n");

        // Generate show rule with template function call
        document.push_str(&Self::generate_show_rule(context, template_def, variant)?);
        document.push_str("\n\n");

        // Generate sections from template configuration
        if !context.sections.is_empty() {
            document.push_str(&Self::generate_sections_from_context(context)?);
        } else {
            document.push_str(&Self::generate_sections_from_template(
                template_def,
                variant,
            )?);
        }

        Ok(document)
    }

    /// Generate sections from context (custom sections)
    fn generate_sections_from_context(context: &TemplateContext) -> Result<String> {
        let mut sections = String::new();

        for section in &context.sections {
            sections.push_str(&format!("= {}\n\n", section));
        }

        Ok(sections)
    }

    /// Generate the Typst import statement
    fn generate_import_statement(context: &TemplateContext) -> Result<String> {
        let template_config = context
            .template_config
            .as_ref()
            .ok_or_else(|| anyhow!("No template configuration available"))?;

        let package_name = &template_config.metadata.name;
        let version = &template_config.metadata.version;

        Ok(format!("#import \"@local/{}:{}\":*", package_name, version))
    }

    /// Generate the show rule with standard template parameters
    fn generate_show_rule(
        context: &TemplateContext,
        template_def: &TemplateDefinition,
        variant: Option<&TemplateVariant>,
    ) -> Result<String> {
        // Determine which function to call
        let function_name = if let Some(variant) = variant {
            variant.function.as_ref().unwrap_or(&template_def.function)
        } else {
            &template_def.function
        };

        // Generate explicit datetime constructor for consistent date preservation
        let now = Local::now();
        let date_str = format!(
            "date: datetime(year: {}, month: {}, day: {})",
            now.year(),
            now.month(),
            now.day()
        );

        // Build the standard parameters that all templates expect
        let params = [
            format!("course: \"{}\"", context.course_id),
            format!("course-name: \"{}\"", context.course_name),
            format!("title: \"{}\"", context.title),
            date_str,
            format!("author: \"{}\"", context.author),
            format!("semester: \"{}\"", context.semester),
        ];

        let params_str = params.join(",\n  ");

        Ok(format!(
            "#show: {}.with(\n  {}\n)",
            function_name, params_str
        ))
    }

    /// Generate sections based on template configuration
    fn generate_sections_from_template(
        template_def: &TemplateDefinition,
        variant: Option<&TemplateVariant>,
    ) -> Result<String> {
        // Get the sections that should be generated
        let sections = Self::get_template_sections(template_def, variant);

        let mut content = String::new();

        for (i, section) in sections.iter().enumerate() {
            if i > 0 {
                content.push_str("\n\n");
            }

            // Generate section header with empty content for user to fill
            content.push_str(&format!("= {}\n\n", section));
        }

        Ok(content)
    }

    /// Get sections that should be created based on template and variant
    fn get_template_sections(
        template_def: &TemplateDefinition,
        variant: Option<&TemplateVariant>,
    ) -> Vec<String> {
        if let Some(variant) = variant {
            // If variant has override sections, use those
            if let Some(ref override_sections) = variant.override_sections {
                return override_sections.clone();
            }

            // Otherwise, start with template default sections and add additional ones
            let mut sections = template_def.default_sections.clone();
            if let Some(ref additional_sections) = variant.additional_sections {
                sections.extend(additional_sections.clone());
            }
            sections
        } else {
            // Use template's default sections
            template_def.default_sections.clone()
        }
    }

    /// Get template definition for the given reference
    fn get_template_definition(
        context: &TemplateContext,
        template_ref: &TemplateReference,
    ) -> Result<TemplateDefinition> {
        let template_config = context
            .template_config
            .as_ref()
            .ok_or_else(|| anyhow!("No template configuration available"))?;

        template_config
            .templates
            .iter()
            .find(|t| t.name == template_ref.name)
            .cloned()
            .ok_or_else(|| anyhow!("Template '{}' not found", template_ref.name))
    }

    fn select_variant_for_template(
        context: &TemplateContext,
        template_def: &TemplateDefinition,
        template_ref: &TemplateReference,
    ) -> Result<Option<TemplateVariant>> {
        // If a specific variant is requested, try to find it
        if let Some(variant_name) = &template_ref.variant {
            return Self::find_specific_variant(context, template_def, variant_name);
        }

        // Only look for variants that belong to this template
        if let Some(template_config) = &context.template_config {
            if let Some(variants) = &template_config.variants {
                let course_type = Self::resolve_course_type(context, template_config);

                // Filter variants to only those belonging to the current template
                let matching_variants: Vec<_> = variants
                    .iter()
                    .filter(|variant| variant.template == template_def.name) // THIS IS KEY!
                    .filter(|variant| {
                        variant.course_types.contains(&course_type)
                            || variant.course_types.contains(&"all".to_string())
                    })
                    .collect();

                // Return the best matching variant for this template, or None
                if let Some(best_variant) = matching_variants.first() {
                    return Ok(Some((*best_variant).clone()));
                }
            }
        }

        // No variant found, use base template
        Ok(None)
    }

    fn find_specific_variant(
        context: &TemplateContext,
        template_def: &TemplateDefinition,
        variant_name: &str,
    ) -> Result<Option<TemplateVariant>> {
        if let Some(template_config) = &context.template_config {
            if let Some(variants) = &template_config.variants {
                for variant in variants {
                    // Must match both the template AND the variant name
                    if variant.template == template_def.name && variant.name == variant_name {
                        return Ok(Some(variant.clone()));
                    }
                }
            }
        }

        anyhow::bail!(
            "Variant '{}' not found for template '{}'",
            variant_name,
            template_def.name
        );
    }

    /// Resolve course type using template's course mapping or context
    fn resolve_course_type(context: &TemplateContext, template_config: &TemplateConfig) -> String {
        // First, try template's course mapping
        if let Some(course_mapping) = &template_config.course_mapping {
            // Check for exact course ID match first
            if let Some(mapped_type) = course_mapping.get(&context.course_id) {
                return mapped_type.clone();
            }

            // Then check for pattern matches (like "01xxx")
            for (pattern, course_type) in course_mapping {
                if Self::matches_course_pattern(&context.course_id, pattern) {
                    return course_type.clone();
                }
            }
        }

        // Fallback to context course type
        context.metadata.course_type.clone()
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
}

#[derive(Debug, Clone)]
#[deprecated(
    since = "0.4.0",
    note = "Use the TemplateReference to create a template variant"
)]
pub enum TemplateType {
    Lecture,
    Assignment,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct TemplateReference {
    pub name: String,
    pub variant: Option<String>,
}

impl TemplateReference {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variant: None,
        }
    }

    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    // Convenience constructors for common types
    pub fn lecture() -> Self {
        Self::new("note")
    }

    pub fn assignment() -> Self {
        Self::new("assignment")
    }

    pub fn lab_report() -> Self {
        Self::new("lab-report")
    }

    pub fn thesis() -> Self {
        Self::new("thesis")
    }
}
