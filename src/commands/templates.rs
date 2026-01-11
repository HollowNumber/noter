//! Template management commands
//!
//! Handles template status, updates, and custom template creation using the new template engine system.

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::config::{Config, get_config};
use crate::core::template::config::{TemplateConfig, TemplateVariant};
use crate::core::template::fetcher::Fetcher;
use crate::core::template::{
    builder::TemplateBuilder, discovery::TemplateDiscovery, engine::TemplateReference,
};
use crate::core::validation::Validator;
use crate::display::output::{OutputManager, Status};

/// Show template status and version information
pub fn template_status() -> Result<()> {
    let config = get_config()?;
    OutputManager::print_status(Status::Loading, "Checking template status...");

    display_template_discovery_status(&config);
    display_github_template_status(&config);
    display_command_examples();

    Ok(())
}

fn display_template_discovery_status(config: &Config) {
    match TemplateDiscovery::load_template_configs(config) {
        Ok(template_configs) => {
            if template_configs.is_empty() {
                OutputManager::print_status(Status::Warning, "No template configurations found");
                return;
            }

            display_template_system_header();
            display_all_template_packages(&template_configs);
            display_all_available_templates(&template_configs);
            display_all_template_variants(&template_configs);
            display_consolidated_course_mapping(&template_configs);
        }
        Err(e) => display_template_discovery_error(&e),
    }
}

fn display_template_system_header() {
    println!();
    println!("{} Template System Status", "📋".blue());
    println!();
}

fn display_all_template_packages(template_configs: &[TemplateConfig]) {
    println!("Template Packages:");
    for (index, config) in template_configs.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("  Package {}:", index + 1);
        println!("    Name: {}", config.metadata.name.bright_white());
        println!("    Version: {}", config.metadata.version.green());
        if let Some(description) = &config.metadata.description {
            println!("    Description: {}", description.dimmed());
        }
        if let Some(repository) = &config.metadata.repository {
            println!("    Repository: {}", repository.bright_blue());
        }
        if let Some(author) = &config.metadata.author {
            println!("    Author: {}", author);
        }
    }
}

fn display_all_available_templates(template_configs: &[TemplateConfig]) {
    let all_templates = TemplateDiscovery::get_all_templates(template_configs);

    if all_templates.is_empty() {
        return;
    }

    println!();
    println!("Available Templates:");
    for (template, config) in all_templates {
        println!(
            "  {} {} - {}",
            "•".bright_green(),
            template.display_name.bright_white(),
            template.description.dimmed()
        );
        println!("    Template: {}", template.name.yellow());
        println!("    Source: {}", config.metadata.name.cyan());
        if !template.default_sections.is_empty() {
            println!(
                "    Sections: {}",
                template.default_sections.join(", ").dimmed()
            );
        }
        if let Some(course_types) = &template.course_types {
            println!("    Course Types: {}", course_types.join(", ").dimmed());
        }
    }
}

fn display_all_template_variants(template_configs: &[TemplateConfig]) {
    let all_variants: Vec<(&TemplateVariant, &TemplateConfig)> = template_configs
        .iter()
        .flat_map(|config| {
            config
                .variants
                .as_ref()
                .map(|variants| variants.iter().map(move |variant| (variant, config)))
                .into_iter()
                .flatten()
        })
        .collect();

    if all_variants.is_empty() {
        return;
    }

    println!();
    println!("Template Variants:");
    for (variant, config) in all_variants {
        println!(
            "  {} {} - {}",
            "•".bright_yellow(),
            variant.display_name.bright_white(),
            format!("for {}", variant.course_types.join(", ")).dimmed()
        );
        println!("    Base: {}", variant.template.yellow());
        println!("    Source: {}", config.metadata.name.cyan());
    }
}

fn display_consolidated_course_mapping(template_configs: &[TemplateConfig]) {
    let all_mappings: Vec<(&String, &String, &TemplateConfig)> = template_configs
        .iter()
        .flat_map(|config| {
            config
                .course_mapping
                .as_ref()
                .map(|mapping| {
                    mapping
                        .iter()
                        .map(move |(pattern, course_type)| (pattern, course_type, config))
                })
                .into_iter()
                .flatten()
        })
        .collect();

    if all_mappings.is_empty() {
        return;
    }

    println!();
    println!("Course Type Mapping:");
    for (pattern, course_type, config) in all_mappings {
        println!(
            "  {} {} -> {} {}",
            "•".bright_cyan(),
            pattern.yellow(),
            course_type.green(),
            format!("({})", config.metadata.name).dimmed()
        );
    }
}

fn display_template_discovery_error(error: &anyhow::Error) {
    OutputManager::print_status(Status::Warning, "Template config not found or invalid");
    println!("Error: {}", error.to_string().dimmed());
    println!();
    println!("This might be because:");
    println!("  • Templates haven't been installed yet");
    println!("  • Template configuration is corrupted");
    println!("  • Template directory is missing");
    println!();
    println!("Try: {}", "noter template update".bright_white());
}

fn display_github_template_status(config: &Config) {
    println!();
    OutputManager::print_status(Status::Loading, "Checking installation status...");

    match Fetcher::check_template_status(config) {
        Ok(statuses) => {
            if statuses.is_empty() {
                println!("  No template repositories configured");
            } else {
                display_installed_templates(statuses);
            }
        }
        Err(e) => {
            println!(
                "  Error checking installation status: {}",
                e.to_string().dimmed()
            );
        }
    }
}

fn display_installed_templates(statuses: Vec<(String, Option<String>)>) {
    println!("Installed Templates:");
    for (repo_name, version_opt) in statuses {
        let (status_icon, status_text, status_color): (
            &str,
            &str,
            fn(&str) -> colored::ColoredString,
        ) = if version_opt.is_some() {
            ("✅", "installed", |s| s.green())
        } else {
            ("❌", "not installed", |s| s.red())
        };

        println!(
            "  {} {} ({})",
            status_icon,
            repo_name.bright_white(),
            status_color(status_text)
        );

        match version_opt {
            Some(version) => println!("    Version: {}", version.bright_blue()),
            None => println!("    {}", "Run 'noter template update' to install".dimmed()),
        }
    }
}

fn display_command_examples() {
    println!();
    OutputManager::print_command_examples(&[
        (
            "noter template update",
            "Update templates to latest version",
        ),
        (
            "noter template create 02101 \"My Template\" -t lecture",
            "Create custom template",
        ),
        ("noter template reinstall", "Reinstall all templates"),
    ]);
}

/// Update templates to the latest version
pub fn update_template() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, "Checking for template updates...");

    // Update templates
    let results = Fetcher::update_templates(&config)?;

    if results.is_empty() {
        OutputManager::print_status(
            Status::Warning,
            "No templates were updated (no repositories configured?)",
        );
        println!();
        println!("To add template repositories, use:");
        println!(
            "  {}",
            "noter config add-template-repo <name> <owner/repo>".bright_white()
        );
        return Ok(());
    }

    for result in results {
        OutputManager::print_status(
            Status::Success,
            &format!(
                "Updated template: {} -> {}",
                result
                    .installed_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("template"),
                result.version.green()
            ),
        );

        println!(
            "  Installed at: {}",
            result.installed_path.display().to_string().dimmed()
        );
    }

    // Verify the update worked by checking template discovery
    println!();
    OutputManager::print_status(Status::Loading, "Verifying template installation...");

    match TemplateDiscovery::load_template_config(&config) {
        Ok(template_config) => {
            OutputManager::print_status(
                Status::Success,
                &format!(
                    "Template system ready ({})",
                    template_config.metadata.version.green()
                ),
            );
        }
        Err(e) => {
            OutputManager::print_status(
                Status::Warning,
                &format!("Template verification failed: {e}"),
            );
        }
    }

    Ok(())
}

/// Force reinstall all templates
pub fn reinstall_template() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, "Reinstalling templates...");

    // This would implement a full reinstall by clearing the templates directory
    // and re-downloading everything
    let templates_dir = Path::new(&config.paths.templates_dir);

    if templates_dir.exists() {
        OutputManager::print_status(Status::Info, "Removing existing templates...");
        fs::remove_dir_all(templates_dir)?;
    }

    // Re-create directory
    fs::create_dir_all(templates_dir)?;

    // Re-download templates
    let results = Fetcher::update_templates(&config)?;

    if results.is_empty() {
        OutputManager::print_status(Status::Error, "No templates were installed");
        return Ok(());
    }

    for result in results {
        OutputManager::print_status(
            Status::Success,
            &format!(
                "Reinstalled: {} ({})",
                result
                    .installed_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("template"),
                result.version.green()
            ),
        );
    }

    OutputManager::print_status(Status::Success, "Template reinstallation complete");

    Ok(())
}

/// Create a custom template using the new TemplateBuilder
pub fn create_custom_template(
    course_id: &str,
    title: &str,
    template_type: &str,
    sections: Option<&str>,
) -> Result<()> {
    let config = get_config()?;

    // Validate course ID
    Validator::validate_course_id(course_id)?;

    OutputManager::print_status(
        Status::Loading,
        &format!(
            "Creating custom {} template for {}",
            template_type.bright_blue(),
            course_id.yellow()
        ),
    );

    // Parse template reference from template type
    let template_ref = match template_type.to_lowercase().as_str() {
        "lecture" | "l" | "note" => TemplateReference::lecture(),
        "assignment" | "a" => TemplateReference::assignment(),
        "lab" | "lab-report" => TemplateReference::lab_report(),
        "thesis" | "project" => TemplateReference::thesis(),
        custom => TemplateReference::new(custom),
    };

    // Build template using new TemplateBuilder
    let mut builder = TemplateBuilder::new(course_id, &config)?
        .with_title(title)
        .with_reference(template_ref);

    // Parse custom sections if provided
    if let Some(sections_str) = sections {
        let custom_sections: Vec<String> = sections_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !custom_sections.is_empty() {
            builder = builder.with_sections(custom_sections);
        }
    } else {
        // Use default sections based on type
        let default_sections = match template_type.to_lowercase().as_str() {
            "assignment" | "a" => config.note_preferences.assignment_sections.clone(),
            _ => config.note_preferences.lecture_sections.clone(),
        };
        builder = builder.with_sections(default_sections);
    }

    // Generate template content using the builder
    let content = builder.build()?;

    // Generate filename
    let filename = generate_custom_template_filename(course_id, template_type, title);

    // Create output directory
    let output_dir = Path::new(&config.paths.notes_dir)
        .join(course_id)
        .join("templates");

    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    // Write template file
    let file_path = output_dir.join(&filename);
    fs::write(&file_path, &content)?;

    OutputManager::print_status(
        Status::Success,
        &format!(
            "Custom template created: {}",
            file_path.display().to_string().bright_white()
        ),
    );

    // Auto-open if configured
    if config.note_preferences.auto_open_file {
        OutputManager::print_status(Status::Info, "Opening in editor...");

        if let Some(editor) = &config.preferred_editor {
            if std::process::Command::new(editor)
                .arg(&file_path)
                .spawn()
                .is_err()
            {
                // Fallback to system default
                let _ = opener::open(&file_path);
            }
        } else {
            let _ = opener::open(&file_path);
        }
    }

    // Show what was created
    println!();
    println!("Template Details:");
    println!("  Type: {}", template_type.bright_blue());
    println!(
        "  Course: {} - {}",
        course_id.yellow(),
        config.get_course_name(course_id).dimmed()
    );
    println!("  Title: {}", title.green());

    // Show sections if any
    let sections_used = if let Some(s) = sections {
        s.split(',')
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        match template_type.to_lowercase().as_str() {
            "assignment" => config.note_preferences.assignment_sections.join(", "),
            _ => config.note_preferences.lecture_sections.join(", "),
        }
    };

    if !sections_used.is_empty() {
        println!("  Sections: {}", sections_used.dimmed());
    }

    println!();
    OutputManager::print_command_examples(&[
        (
            &format!("noter compile {}", file_path.display()),
            "Compile to PDF",
        ),
        (
            &format!("noter watch {}", file_path.display()),
            "Watch and auto-compile",
        ),
        (&format!("noter recent {}", course_id), "List recent files"),
    ]);

    Ok(())
}

/// Generate filename for custom templates
fn generate_custom_template_filename(course_id: &str, template_type: &str, title: &str) -> String {
    use chrono::Local;

    let date = Local::now().format("%Y-%m-%d").to_string();
    let template_part = template_type.replace(' ', "-").to_lowercase();

    if title.is_empty() {
        format!("{date}-{course_id}-{template_part}.typ")
    } else {
        let title_part = title.replace(' ', "-").to_lowercase();
        format!("{date}-{course_id}-{template_part}-{title_part}.typ")
    }
}
