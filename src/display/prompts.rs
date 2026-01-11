//! Interactive prompts and user input handling
//!
//! Provides utilities for getting user input, confirmation prompts,
//! and selection menus.

use anyhow::Result;
use colored::*;
use std::io::{self, Write};

pub struct PromptManager;

#[allow(dead_code)]
impl PromptManager {
    /// Ask for confirmation (y/n)
    pub fn confirm(message: &str, default: Option<bool>) -> Result<bool> {
        let default_text = match default {
            Some(true) => " [Y/n]",
            Some(false) => " [y/N]",
            None => " [y/n]",
        };

        loop {
            print!("{} {}{}: ", "❓".yellow(), message, default_text.dimmed());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            match input.as_str() {
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                "" => {
                    if let Some(default_value) = default {
                        return Ok(default_value);
                    }
                    println!("Please enter 'y' or 'n'");
                }
                _ => println!("Please enter 'y' or 'n'"),
            }
        }
    }

    /// Get text input from user
    pub fn input(message: &str, default: Option<&str>) -> Result<String> {
        let default_text = if let Some(default_val) = default {
            format!(" [{}]", default_val.dimmed())
        } else {
            String::new()
        };

        print!("{} {}{}: ", "❓".blue(), message, default_text);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            if let Some(default_val) = default {
                Ok(default_val.to_string())
            } else {
                Ok(String::new())
            }
        } else {
            Ok(input.to_string())
        }
    }

    /// Get required text input (cannot be empty)
    pub fn required_input(message: &str) -> Result<String> {
        loop {
            let input = Self::input(message, None)?;
            if !input.trim().is_empty() {
                return Ok(input);
            }
            println!("{} This field is required", "⚠️".yellow());
        }
    }

    /// Select from a list of options
    pub fn select(message: &str, options: &[String]) -> Result<usize> {
        if options.is_empty() {
            return Err(anyhow::anyhow!("No options provided"));
        }

        println!("{} {}:", "❓".blue(), message);
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().bright_white(), option);
        }

        loop {
            print!("Select option (1-{}): ", options.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(num) if num >= 1 && num <= options.len() => {
                    return Ok(num - 1); // Convert to 0-based index
                }
                _ => {
                    println!(
                        "{} Please enter a number between 1 and {}",
                        "⚠️".yellow(),
                        options.len()
                    );
                }
            }
        }
    }

    /// Multi-select from a list of options
    pub fn multi_select(message: &str, options: &[String]) -> Result<Vec<usize>> {
        if options.is_empty() {
            return Err(anyhow::anyhow!("No options provided"));
        }

        println!(
            "{} {} (enter numbers separated by commas):",
            "❓".blue(),
            message
        );
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().bright_white(), option);
        }

        loop {
            print!("Select options (e.g., 1,3,5): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let selections: Result<Vec<usize>, _> = input
                .trim()
                .split(',')
                .map(|s| s.trim().parse::<usize>())
                .collect();

            match selections {
                Ok(nums) => {
                    let valid_nums: Vec<usize> = nums
                        .into_iter()
                        .filter(|&num| num >= 1 && num <= options.len())
                        .map(|num| num - 1) // Convert to 0-based
                        .collect();

                    if !valid_nums.is_empty() {
                        return Ok(valid_nums);
                    }
                    println!("{} Please enter valid option numbers", "⚠️".yellow());
                }
                Err(_) => {
                    println!("{} Please enter numbers separated by commas", "⚠️".yellow());
                }
            }
        }
    }

    /// Display a warning and ask for confirmation
    pub fn warn_and_confirm(warning: &str, action: &str) -> Result<bool> {
        println!("{} {}", "⚠️".bright_yellow(), warning.yellow());
        Self::confirm(&format!("Do you want to {}", action), Some(false))
    }

    /// Show a spinner with message (simple text-based)
    pub fn with_spinner<F, T>(message: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        print!("{} {}...", "⏳".blue(), message);
        io::stdout().flush()?;

        let result = f()?;

        // Clear the line and show success
        print!("\r{} {} ✓\n", "✅".green(), message);
        io::stdout().flush()?;

        Ok(result)
    }

    /// Interactive course selection from configured courses
    pub fn select_course(courses: &[(String, String)]) -> Result<String> {
        if courses.is_empty() {
            return Err(anyhow::anyhow!("No courses configured. Add courses first."));
        }

        let options: Vec<String> = courses
            .iter()
            .map(|(id, name)| format!("{} - {}", id, name))
            .collect();

        let selection = Self::select("Select a course", &options)?;
        Ok(courses[selection].0.clone())
    }

    /// Interactive editor selection
    pub fn select_editor(editors: &[String]) -> Result<String> {
        if editors.is_empty() {
            return Err(anyhow::anyhow!("No editors available"));
        }

        let selection = Self::select("Choose your preferred editor", editors)?;
        Ok(editors[selection].clone())
    }

    /// Setup wizard prompt
    pub fn setup_wizard() -> Result<UserPreferences> {
        println!("{} DTU Notes Setup Wizard", "🎓".blue());
        println!();

        let author = Self::required_input("Enter your name (for templates)")?;

        let editor = Self::input(
            "Enter your preferred editor command (e.g., 'code', 'nvim')",
            Some("code"),
        )?;

        let auto_open = Self::confirm("Automatically open files after creation", Some(true))?;

        let include_date = Self::confirm("Include date in note titles", Some(true))?;

        Ok(UserPreferences {
            author,
            editor: if editor.is_empty() {
                None
            } else {
                Some(editor)
            },
            auto_open,
            include_date,
        })
    }
}

/// User preferences collected from setup wizard
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub author: String,
    pub editor: Option<String>,
    pub auto_open: bool,
    pub include_date: bool,
}

/// Specialized prompts for DTU Noter
#[allow(dead_code)]
pub struct NoterPrompts;

#[allow(dead_code)]
impl NoterPrompts {
    /// Prompt for course information
    pub fn course_info() -> Result<(String, String)> {
        println!("{} Add New Course", "🎓".blue());

        let course_id = loop {
            let input = PromptManager::required_input("Enter course code (5 digits, e.g. 02101)")?;
            if input.len() == 5 && input.chars().all(|c| c.is_ascii_digit()) {
                break input;
            }
            println!("{} Course code must be exactly 5 digits", "⚠️".yellow());
        };

        let course_name = PromptManager::required_input("Enter course name")?;

        Ok((course_id, course_name))
    }

    /// Prompt for assignment details
    pub fn assignment_info(course_id: &str) -> Result<String> {
        let title =
            PromptManager::required_input(&format!("Enter assignment title for {}", course_id))?;
        Ok(title)
    }

    /// Prompt for template sections
    pub fn template_sections(section_type: &str, defaults: &[String]) -> Result<Vec<String>> {
        println!("{} Configure {} sections:", "📝".blue(), section_type);

        if Self::confirm_use_defaults(defaults) {
            Ok(defaults.to_vec())
        } else {
            Self::collect_custom_sections()
        }
    }

    /// Confirm whether to use default editor or configure a custom one
    pub fn configure_editor() -> Result<Option<String>> {
        let use_default =
            PromptManager::confirm("Use default editor detection (recommended)", Some(true))?;

        if use_default {
            Ok(None)
        } else {
            let editor = PromptManager::input("Enter your preferred editor command", Some("code"))?;
            Ok(Some(editor))
        }
    }

    /// Prompt for directory paths during setup
    pub fn configure_paths() -> Result<PathConfig> {
        println!("{} Configure Directory Paths", "📁".blue());

        let notes_dir = PromptManager::input("Notes directory", Some("notes"))?;

        let obsidian_dir =
            PromptManager::input("Obsidian vault directory", Some("obsidian-vault"))?;

        let templates_dir = PromptManager::input("Templates directory", Some("templates"))?;

        Ok(PathConfig {
            notes_dir,
            obsidian_dir,
            templates_dir,
            typst_packages_dir: crate::config::PathConfig::default().typst_packages_dir,
        })
    }

    fn confirm_use_defaults(defaults: &[String]) -> bool {
        println!("Default sections:");
        for (i, section) in defaults.iter().enumerate() {
            println!("  {}. {}", i + 1, section);
        }

        PromptManager::confirm("Use these default sections", Some(true)).unwrap_or(true)
    }

    fn collect_custom_sections() -> Result<Vec<String>> {
        let mut sections = Vec::new();

        println!("Enter section names (press Enter with empty input to finish):");
        loop {
            let section =
                PromptManager::input(&format!("Section {} name", sections.len() + 1), None)?;

            if section.trim().is_empty() {
                break;
            }
            sections.push(section);
        }

        if sections.is_empty() {
            sections.push("Notes".to_string());
        }

        Ok(sections)
    }
}

// Re-export the PathConfig from config module for convenience
use crate::config::PathConfig;
