//! Setup command implementation
//!
//! Thin command layer that delegates to core setup manager.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use colored::Colorize;

use crate::config::get_config;
use crate::core::setup::{SetupConfig, SetupManager};
use crate::display::output::{OutputManager, Status};
use crate::display::prompts::PromptManager;

/// Prompt for setup configuration options
fn prompt_setup_options() -> Result<SetupConfig> {
    println!();
    println!("{} Setup Configuration", "⚙️".blue());
    println!();

    let create_sample_courses =
        PromptManager::confirm("Create sample courses (02101, 02132)", Some(true))?;

    let install_templates =
        PromptManager::confirm("Install DTU templates from GitHub", Some(true))?;

    let create_readme = PromptManager::confirm("Create README file", Some(true))?;

    let create_gitignore = PromptManager::confirm("Create .gitignore file", Some(true))?;

    let force_overwrite = PromptManager::confirm("Force overwrite existing files", Some(false))?;

    Ok(SetupConfig {
        create_sample_courses,
        install_templates,
        create_readme,
        create_gitignore,
        force_overwrite,
    })
}

pub fn setup_repository() -> Result<()> {
    // Run the setup wizard to get user preferences and setup options
    let user_prefs = PromptManager::setup_wizard()?;
    let setup_config = prompt_setup_options()?;

    let mut config = get_config()?;

    // Apply user preferences to config
    if !user_prefs.author.is_empty() {
        config.author = user_prefs.author;
    }
    config.preferred_editor = user_prefs.editor;
    config.note_preferences.auto_open_file = user_prefs.auto_open;
    config.note_preferences.include_date_in_title = user_prefs.include_date;

    // Save the updated config
    config.save()?;

    OutputManager::print_status(Status::Loading, "Setting up DTU notes repository...");

    match SetupManager::setup_repository(&config, &setup_config) {
        Ok(result) => {
            OutputManager::print_status(Status::Success, "Setup completed successfully! 🎉");

            println!();
            println!("{} Directories created:", "📁".blue());
            for dir in &result.directories_created {
                let dir_str = dir.display().to_string();
                println!("  • {}", dir_str.dimmed());
            }

            if !result.templates_installed.is_empty() {
                println!();
                println!("{} Templates installed:", "📦".blue());
                for template in &result.templates_installed {
                    println!("  • {}", template.green());
                }
            }

            if !result.sample_courses.is_empty() {
                println!();
                println!("{} Sample courses created:", "📚".blue());
                let sample_courses_data = SetupManager::get_sample_courses();
                for course_id in &result.sample_courses {
                    if let Some((_, course_name)) =
                        sample_courses_data.iter().find(|(id, _)| id == course_id)
                    {
                        println!("  {} - {}", course_id.yellow(), course_name);
                    } else {
                        println!("  {}", course_id.yellow());
                    }
                }
            }

            if !result.warnings.is_empty() {
                println!();
                println!("{} Warnings:", "⚠️".yellow());
                for warning in &result.warnings {
                    println!("  • {}", warning.yellow());
                }
            }

            // Show next steps
            println!();
            OutputManager::print_command_examples(&[
                (
                    "noter config set-author \"Your Full Name\"",
                    "Set your name",
                ),
                ("noter note 02101", "Create first note"),
                ("noter config show", "Check your setup"),
            ]);
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Setup failed: {}", e));
        }
    }

    Ok(())
}

pub fn clean_setup() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(
        Status::Warning,
        "This will remove all directories and files created by setup.",
    );

    println!("The following will be deleted:");
    println!("  • {}", config.paths.notes_dir);
    println!("  • {}", config.paths.obsidian_dir);
    println!("  • {}", config.paths.templates_dir);
    println!("  • README.md");
    println!("  • .gitignore");

    use std::io::{self, Write};
    print!("\nAre you sure? Type 'yes' to confirm: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    match SetupManager::clean_setup(&config) {
        Ok(cleaned_items) => {
            OutputManager::print_status(Status::Success, "Setup cleanup completed!");

            for item in cleaned_items {
                let item_str = item.display().to_string();
                println!("{} Removed: {}", "🗑️".red(), item_str);
            }

            println!("\nRun {} to set up again.", "noter setup".bright_white());
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Cleanup failed: {}", e));
        }
    }

    Ok(())
}

/// Setup with custom configuration options
#[allow(dead_code)]
pub fn setup_repository_with_options(
    create_samples: bool,
    install_templates: bool,
    force_overwrite: bool,
) -> Result<()> {
    let config = get_config()?;
    let setup_config = SetupConfig {
        create_sample_courses: create_samples,
        install_templates,
        create_readme: true,
        create_gitignore: true,
        force_overwrite,
    };

    OutputManager::print_status(
        Status::Loading,
        "Setting up DTU notes repository with custom options...",
    );

    match SetupManager::setup_repository(&config, &setup_config) {
        Ok(result) => {
            OutputManager::print_status(Status::Success, "Custom setup completed successfully! 🎉");

            // Display results (same as above)
            if !result.directories_created.is_empty() {
                println!();
                println!("{} Directories created:", "📁".blue());
                for dir in &result.directories_created {
                    let dir_str = dir.display().to_string();
                    println!("  • {}", dir_str.dimmed());
                }
            }

            if !result.templates_installed.is_empty() {
                println!();
                println!("{} Templates installed:", "📦".blue());
                for template in &result.templates_installed {
                    println!("  • {}", template.green());
                }
            }

            if !result.warnings.is_empty() {
                println!();
                println!("{} Warnings:", "⚠️".yellow());
                for warning in &result.warnings {
                    println!("  • {}", warning.yellow());
                }
            }
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Setup failed: {}", e));
        }
    }

    Ok(())
}

/// Show setup status
pub fn show_setup_status() -> Result<()> {
    let config = get_config()?;

    match SetupManager::check_setup_status(&config) {
        Ok(status) => {
            OutputManager::print_section("Setup Status", Some("🔧"));

            println!(
                "Completion: {}%",
                status.completion_percentage().to_string().bright_green()
            );
            println!();

            let check_mark = |exists: bool| if exists { "✅" } else { "❌" };

            println!("📁 Directories:");
            println!(
                "  {} Notes directory: {}",
                check_mark(status.notes_dir_exists),
                config.paths.notes_dir.dimmed()
            );
            println!(
                "  {} Obsidian directory: {}",
                check_mark(status.obsidian_dir_exists),
                config.paths.obsidian_dir.dimmed()
            );
            println!(
                "  {} Templates directory: {}",
                check_mark(status.templates_dir_exists),
                config.paths.templates_dir.dimmed()
            );

            println!();
            println!("📦 Templates:");
            println!(
                "  {} DTU templates installed: {}",
                check_mark(status.templates_installed),
                if status.templates_installed {
                    "Yes"
                } else {
                    "Run setup to install"
                }
            );

            println!();
            println!("🎓 Courses:");
            println!(
                "  Sample courses created: {}",
                status.sample_courses_count.to_string().bright_white()
            );

            println!();
            println!("⚙️ Configuration:");
            println!(
                "  {} Author configured: {}",
                check_mark(status.author_configured),
                if status.author_configured {
                    &config.author
                } else {
                    "Run 'noter config set-author'"
                }
            );

            if !status.is_complete() {
                println!();
                println!(
                    "{} Run {} to complete setup",
                    "💡".blue(),
                    "noter setup".bright_white()
                );
            } else {
                println!();
                println!("{} Setup is complete! Ready to take notes.", "🎉".green());
            }
        }
        Err(e) => {
            OutputManager::print_status(
                Status::Error,
                &format!("Failed to check setup status: {}", e),
            );
        }
    }

    Ok(())
}

use clap_complete::env::{Bash, EnvCompleter, Fish, Zsh};

const BIN: &str = "noter";
const ENV_VAR: &str = "COMPLETE";

/// Install the dynamic completion registration script for the user's shell.
///
/// The script only calls back into `noter` on tab, so it never needs
/// regenerating when commands or completers change.
pub(crate) fn completions(shell: Option<&str>) -> Result<()> {
    let shell = match shell {
        Some(s) => s.to_owned(),
        None => detect_shell()?,
    };

    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    let data_dir = dirs::data_dir().context("Could not determine data directory")?;

    let (completer, path, needs_source): (&dyn EnvCompleter, PathBuf, bool) = match shell.as_str() {
        "fish" => (&Fish, config_dir.join("fish/completions/noter.fish"), false),
        "bash" => (
            &Bash,
            data_dir.join("bash-completion/completions/noter"),
            false,
        ),
        "zsh" => (&Zsh, data_dir.join("noter/completions.zsh"), true),
        other => bail!("Unsupported shell '{other}' (supported: fish, bash, zsh)"),
    };

    let mut script = Vec::new();
    completer
        .write_registration(ENV_VAR, BIN, BIN, BIN, &mut script)
        .context("Failed to generate completion script")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(&path, script).with_context(|| format!("Failed to write {}", path.display()))?;

    println!("Installed {shell} completions to {}", path.display());
    if needs_source {
        println!(
            "Add this line to your ~/.zshrc:\n  source {}",
            path.display()
        );
    }

    Ok(())
}

fn detect_shell() -> Result<String> {
    let shell = std::env::var("SHELL")
        .context("Could not detect shell from $SHELL; pass the shell explicitly")?;

    Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("Could not parse shell from $SHELL: {shell}"))
}
