//! Template builder for fluent template construction
//!
//! This module provides a fluent builder API for creating complex templates
//! with custom sections, variables, and processing options. It integrates
//! with the template discovery system and delegates actual generation to the
//! TemplateEngine.

use anyhow::Result;

use super::config::{TemplateConfig, TemplateDefinition, TemplateVariant};
use super::context::{TemplateContext, TemplateContextBuilder, TemplateMetadata};
use super::discovery::TemplateDiscovery;
use super::engine::{TemplateEngine, TemplateReference};
use super::validation::{TemplateValidator, ValidationIssue, ValidationSeverity};
use crate::config::Config;

/// Template builder for fluent template construction
///
/// The TemplateBuilder provides a chainable interface for constructing templates
/// with custom configurations, sections, and processing options. It automatically
/// integrates with the template discovery system and delegates to TemplateEngine.
///
/// ## Usage Examples
///
/// ```no_run
/// use noter::core::template::engine::TemplateReference;
/// use noter::core::template::builder::TemplateBuilder;
/// use noter::config::Config;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let config = Config::default();
/// let content = TemplateBuilder::new("02101", &config)?
///     .with_title("Advanced Data Structures")
///     .with_reference(TemplateReference::assignment())
///     .with_sections(vec!["Problem 1".to_string(), "Analysis".to_string()])
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct TemplateBuilder {
    context_builder: TemplateContextBuilder,
    template_reference: TemplateReference,
    variant_override: Option<String>,
    processing_options: ProcessingOptions,
}

/// Processing options for template generation
#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    pub validate_before_build: bool,
    pub apply_transformations: bool,
    pub include_debug_info: bool,
    pub validation_level: ValidationLevel,
    pub fail_on_validation_errors: bool,
}

/// Level of validation to perform
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    /// Only validate critical errors that would prevent template generation
    Minimal,
    /// Standard validation including warnings and recommendations
    Standard,
    /// Comprehensive validation including style and best practices
    Comprehensive,
}

impl TemplateBuilder {
    /// Create a new template builder for the given course
    pub fn new(course_id: &str, config: &Config) -> Result<Self> {
        // Load template configuration
        let template_config = TemplateDiscovery::load_template_config(config)?;

        let context_builder = TemplateContext::builder()
            .with_course_id(course_id)
            .with_config(config.clone())
            .with_template_config(template_config);

        Ok(Self {
            context_builder,
            template_reference: TemplateReference::lecture(), // Default
            variant_override: None,
            processing_options: ProcessingOptions::default(),
        })
    }

    /// Set the template reference
    pub fn with_reference(mut self, template_ref: TemplateReference) -> Self {
        self.template_reference = template_ref;
        self
    }

    /// Set the template title
    pub fn with_title(mut self, title: &str) -> Self {
        self.context_builder = self.context_builder.with_title(title);
        self
    }

    /// Set custom sections for the template
    pub fn with_sections(mut self, sections: Vec<String>) -> Self {
        self.context_builder = self.context_builder.with_sections(sections);
        self
    }

    /// Add a template variable
    pub fn with_variable(mut self, key: &str, value: &str) -> Self {
        self.context_builder = self.context_builder.with_variable(key, value);
        self
    }

    /// Add a custom field
    pub fn with_custom_field(mut self, key: &str, value: &str) -> Self {
        self.context_builder = self.context_builder.with_custom_field(key, value);
        self
    }

    /// Override automatic variant selection
    pub fn with_variant(mut self, variant_name: &str) -> Self {
        self.variant_override = Some(variant_name.to_string());
        self
    }

    /// Configure processing options
    pub fn with_processing_options(mut self, options: ProcessingOptions) -> Self {
        self.processing_options = options;
        self
    }

    /// Enable/disable validation before build
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.processing_options.validate_before_build = enabled;
        self
    }

    /// Set validation level
    pub fn with_validation_level(mut self, level: ValidationLevel) -> Self {
        self.processing_options.validation_level = level;
        self
    }

    /// Enable/disable failing on validation errors
    pub fn with_fail_on_errors(mut self, fail: bool) -> Self {
        self.processing_options.fail_on_validation_errors = fail;
        self
    }

    /// Enable/disable variable transformations
    pub fn with_transformations(mut self, enabled: bool) -> Self {
        self.processing_options.apply_transformations = enabled;
        self
    }

    /// Enable/disable debug information
    pub fn with_debug_info(mut self, enabled: bool) -> Self {
        self.processing_options.include_debug_info = enabled;
        self
    }

    /// Build the template content - returns only the generated Typst code
    pub fn build(&self) -> Result<String> {
        // Build the context
        let mut context = self.context_builder.clone().build()?;

        // Apply validation if enabled
        if self.processing_options.validate_before_build {
            let validation_result = self.validate_template(&context)?;

            // Handle validation results based on configuration
            if !validation_result.issues.is_empty() {
                self.handle_validation_issues(&validation_result.issues)?;
            }
        }

        // Apply processing options before rendering
        if self.processing_options.apply_transformations {
            context.apply_transformations()?;
        }

        // Apply variant override if specified
        let template_ref = if let Some(variant_name) = &self.variant_override {
            self.template_reference.clone().with_variant(variant_name)
        } else {
            self.template_reference.clone()
        };

        // Delegate to TemplateEngine for actual generation
        TemplateEngine::render_template(&context, &template_ref)
    }

    /// Build with validation report (for debugging and analysis)
    pub fn build_with_validation(&self) -> Result<TemplateOutputWithValidation> {
        let context = self.context_builder.clone().build()?;

        // Always perform validation for this method
        let validation_result = self.validate_template(&context)?;

        // Build content (may still succeed with warnings)
        let content = if validation_result.has_errors()
            && self.processing_options.fail_on_validation_errors
        {
            return Err(anyhow::anyhow!(
                "Template validation failed with {} errors",
                validation_result.error_count()
            ));
        } else {
            self.build()?
        };

        Ok(TemplateOutputWithValidation {
            content,
            validation_result,
            context_summary: ContextSummary::from_context(&context),
        })
    }

    /// Build with metadata information (for advanced use cases)
    pub fn build_with_metadata(&self) -> Result<TemplateOutput> {
        let context = self.context_builder.clone().build()?;
        let content = self.build()?;
        let context_summary = ContextSummary::from_context(&context);

        Ok(TemplateOutput {
            content,
            metadata: context.metadata.clone(),
            context_summary,
        })
    }

    /// Validate the template without building it
    pub fn validate(&self) -> Result<ValidationResult> {
        let context = self.context_builder.clone().build()?;
        self.validate_template(&context)
    }

    /// Internal validation method
    fn validate_template(&self, context: &TemplateContext) -> Result<ValidationResult> {
        let mut all_issues = Vec::new();

        // Get template definition and variant for validation
        let template_config = context
            .template_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No template configuration available for validation"))?;

        // Find the template definition based on reference
        let template_def =
            self.find_template_definition(template_config, &self.template_reference)?;

        // Find variant if specified
        let variant = self.find_template_variant(template_config, &template_def)?;

        // Perform validation based on level
        match self.processing_options.validation_level {
            ValidationLevel::Minimal => {
                // Only critical validation
                let issues = TemplateValidator::validate_template_context(
                    context,
                    &template_def,
                    variant.as_ref(),
                )?;
                all_issues.extend(
                    issues
                        .into_iter()
                        .filter(|issue| issue.severity == ValidationSeverity::Error),
                );
            }
            ValidationLevel::Standard => {
                // Standard validation
                let issues = TemplateValidator::validate_template_context(
                    context,
                    &template_def,
                    variant.as_ref(),
                )?;
                all_issues.extend(issues);
            }
            ValidationLevel::Comprehensive => {
                // Comprehensive validation
                let context_issues = TemplateValidator::validate_template_context(
                    context,
                    &template_def,
                    variant.as_ref(),
                )?;
                all_issues.extend(context_issues);

                // Additional system-level validation - we need to get config from the builder
                if let Some(config) = &self.context_builder.get_config() {
                    if let Ok(system_issues) = TemplateValidator::validate_system(config) {
                        all_issues.extend(system_issues);
                    }

                    // Validate template configuration
                    if let Ok(config_issues) =
                        TemplateValidator::validate_template_config(template_config)
                    {
                        all_issues.extend(config_issues);
                    }
                }
            }
        }

        Ok(ValidationResult { issues: all_issues })
    }

    /// Handle validation issues based on processing options
    fn handle_validation_issues(&self, issues: &[ValidationIssue]) -> Result<()> {
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .collect();
        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .collect();

        // Always fail on errors if fail_on_validation_errors is true
        if !errors.is_empty() && self.processing_options.fail_on_validation_errors {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| format!("{}: {}", e.category, e.message))
                .collect();
            return Err(anyhow::anyhow!(
                "Template validation errors:\n{}",
                error_messages.join("\n")
            ));
        }

        // Print warnings and info if debug info is enabled
        if self.processing_options.include_debug_info {
            if !warnings.is_empty() {
                eprintln!("Template validation warnings:");
                for warning in &warnings {
                    eprintln!("  Warning [{}]: {}", warning.category, warning.message);
                    if let Some(suggestion) = &warning.suggestion {
                        eprintln!("    Suggestion: {}", suggestion);
                    }
                }
            }

            let info: Vec<_> = issues
                .iter()
                .filter(|i| i.severity == ValidationSeverity::Info)
                .collect();
            if !info.is_empty() {
                eprintln!("Template validation info:");
                for info in &info {
                    eprintln!("  Info [{}]: {}", info.category, info.message);
                }
            }
        }

        Ok(())
    }

    /// Find template definition by reference
    fn find_template_definition(
        &self,
        config: &TemplateConfig,
        reference: &TemplateReference,
    ) -> Result<TemplateDefinition> {
        // config.templates is a Vec<TemplateDefinition>, not Option<Vec<TemplateDefinition>>
        for template in &config.templates {
            if template.name == reference.name {
                return Ok(template.clone());
            }
        }
        Err(anyhow::anyhow!(
            "Template '{}' not found in configuration",
            reference.name
        ))
    }

    /// Find template variant if specified
    fn find_template_variant(
        &self,
        config: &TemplateConfig,
        template_def: &TemplateDefinition,
    ) -> Result<Option<TemplateVariant>> {
        if let Some(variant_name) = &self.variant_override {
            if let Some(variants) = &config.variants {
                for variant in variants {
                    if variant.name == *variant_name && variant.template == template_def.name {
                        return Ok(Some(variant.clone()));
                    }
                }
            }
            return Err(anyhow::anyhow!(
                "Variant '{}' not found for template '{}'",
                variant_name,
                template_def.name
            ));
        }
        Ok(None)
    }
}

/// Result of template validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count()
    }

    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    /// Format validation result as a human-readable report
    pub fn format_report(&self) -> String {
        TemplateValidator::format_validation_report(&self.issues)
    }
}

/// Template output with validation information
#[derive(Debug, Clone)]
pub struct TemplateOutputWithValidation {
    pub content: String,
    pub validation_result: ValidationResult,
    pub context_summary: ContextSummary,
}

/// Output structure containing template content and metadata
#[derive(Debug, Clone)]
pub struct TemplateOutput {
    pub content: String,
    pub metadata: TemplateMetadata,
    pub context_summary: ContextSummary,
}

/// Summary of the template context for debugging and logging
#[derive(Debug, Clone)]
pub struct ContextSummary {
    pub course_id: String,
    pub course_name: String,
    pub title: String,
    pub template_type: String,
    pub sections_count: usize,
    pub variables_count: usize,
    pub variant_used: Option<String>,
}

impl ContextSummary {
    fn from_context(context: &TemplateContext) -> Self {
        Self {
            course_id: context.course_id.clone(),
            course_name: context.course_name.clone(),
            title: context.title.clone(),
            template_type: context.metadata.course_type.clone(),
            sections_count: context.sections.len(),
            variables_count: context.variables.len(),
            variant_used: context.metadata.variant_used.clone(),
        }
    }
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            validate_before_build: true,
            apply_transformations: true,
            include_debug_info: false,
            validation_level: ValidationLevel::Standard,
            fail_on_validation_errors: false, // Allow warnings to pass through
        }
    }
}

/// Builder for processing options with fluent API
pub struct ProcessingOptionsBuilder {
    options: ProcessingOptions,
}

impl ProcessingOptionsBuilder {
    pub fn new() -> Self {
        Self {
            options: ProcessingOptions::default(),
        }
    }

    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.options.validate_before_build = enabled;
        self
    }

    pub fn with_validation_level(mut self, level: ValidationLevel) -> Self {
        self.options.validation_level = level;
        self
    }

    pub fn with_fail_on_errors(mut self, fail: bool) -> Self {
        self.options.fail_on_validation_errors = fail;
        self
    }

    pub fn with_transformations(mut self, enabled: bool) -> Self {
        self.options.apply_transformations = enabled;
        self
    }

    pub fn with_debug_info(mut self, enabled: bool) -> Self {
        self.options.include_debug_info = enabled;
        self
    }

    pub fn build(self) -> ProcessingOptions {
        self.options
    }
}

impl Default for ProcessingOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_options_builder() {
        let options = ProcessingOptionsBuilder::new()
            .with_validation(true)
            .with_validation_level(ValidationLevel::Comprehensive)
            .with_fail_on_errors(true)
            .with_transformations(false)
            .build();

        assert!(options.validate_before_build);
        assert_eq!(options.validation_level, ValidationLevel::Comprehensive);
        assert!(options.fail_on_validation_errors);
        assert!(!options.apply_transformations);
    }

    #[test]
    fn test_processing_options_default() {
        let options = ProcessingOptions::default();

        assert!(options.validate_before_build);
        assert!(options.apply_transformations);
        assert!(!options.include_debug_info);
        assert_eq!(options.validation_level, ValidationLevel::Standard);
        assert!(!options.fail_on_validation_errors);
    }

    #[test]
    fn test_validation_result_methods() {
        let issues = [
            ValidationIssue {
                severity: ValidationSeverity::Error,
                category: "test".to_string(),
                message: "Test error".to_string(),
                suggestion: None,
                location: None,
            },
            ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "test".to_string(),
                message: "Test warning".to_string(),
                suggestion: None,
                location: None,
            },
        ];

        let result = ValidationResult {
            issues: issues.into(),
        };

        assert!(result.has_errors());
        assert!(result.has_warnings());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert!(!result.is_clean());
    }
}
