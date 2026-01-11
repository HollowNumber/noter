use anyhow::Result;
use colored::*;
use serde_json::Value;

use crate::config::{Config, TemplateRepository, get_config, update_author, update_editor};
use crate::display::output::{OutputManager, Status};

pub fn show_config() -> Result<()> {
    let config = get_config()?;

    println!("{} Current Configuration:", "⚙️".blue());
    println!();

    // Serialize to JSON Value for smart traversal
    let json_value = serde_json::to_value(&config)?;

    // Display the config recursively with smart formatting
    display_value(&json_value, 0, "");

    Ok(())
}

/// Recursively display JSON values with smart formatting and colors
fn display_value(value: &Value, indent: usize, key: &str) {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Object(map) => {
            if !key.is_empty() {
                println!("{}{}", indent_str, key.bright_cyan().bold());
            }
            for (k, v) in map {
                display_value(v, indent + 1, k);
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                println!("{}{}: {}", indent_str, key.green(), "[]".bright_black());
            } else {
                println!("{}{}", indent_str, key.green().bold());
                for (i, item) in arr.iter().enumerate() {
                    if let Value::Object(_) = item {
                        println!("{}  {}:", indent_str, format!("[{i}]").yellow());
                        display_value(item, indent + 1, "");
                    } else {
                        print!("{indent_str}  - ");
                        display_value(item, 0, "");
                    }
                }
            }
        }
        Value::String(s) => {
            println!("{}{}: {}", indent_str, key.green(), s.yellow());
        }
        Value::Number(n) => {
            println!("{}{}: {}", indent_str, key.green(), n.to_string().cyan());
        }
        Value::Bool(b) => {
            let colored_bool = if *b {
                "true".bright_green()
            } else {
                "false".bright_red()
            };
            println!("{}{}: {}", indent_str, key.green(), colored_bool);
        }
        Value::Null => {
            println!("{}{}: {}", indent_str, key.green(), "null".bright_black());
        }
    }
}

pub fn get_config_value(key: &str) -> Result<()> {
    let config = get_config()?;
    let json_value = serde_json::to_value(&config)?;

    // Navigate to the value using dot notation
    let value = navigate_json_path(&json_value, key)
        .ok_or_else(|| anyhow::anyhow!("Configuration key '{}' not found", key))?;

    println!("{}: {}", key.green(), format_value_inline(value));
    Ok(())
}

pub fn set_config_value(key: &str, value: &str) -> Result<()> {
    let mut config = get_config()?;
    let mut json_value = serde_json::to_value(&config)?;

    // Update the value using dot notation
    update_json_path(&mut json_value, key, value)?;

    // Deserialize back to Config and save
    config = serde_json::from_value(json_value)?;
    config.save()?;

    println!(
        "{} Configuration updated: {} = {}",
        "✅".green(),
        key.cyan(),
        value.yellow()
    );
    Ok(())
}

pub fn edit_config() -> Result<()> {
    let config_path = Config::config_file_path()?;
    let config = get_config()?;

    // Get editor
    let editor = config
        .preferred_editor
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    println!(
        "{} Opening config file in {}...",
        "📝".blue(),
        editor.yellow()
    );
    println!("{}", config_path.display().to_string().bright_black());

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    if status.success() {
        // Validate the config after editing
        match get_config() {
            Ok(_) => {
                println!("{} Configuration file is valid", "✅".green());
            }
            Err(e) => {
                println!(
                    "{} Configuration file has errors: {}",
                    "⚠️".yellow(),
                    e.to_string().red()
                );
                println!("Please fix the errors and try again.");
            }
        }
    } else {
        println!("{} Editor exited with error", "❌".red());
    }

    Ok(())
}

pub fn list_config_keys() -> Result<()> {
    let config = get_config()?;
    let json_value = serde_json::to_value(&config)?;

    println!("{} Available Configuration Keys:", "🔑".blue());
    println!();

    let keys = collect_json_keys(&json_value, "");
    for key in keys {
        println!("  {}", key.green());
    }

    println!();
    println!("Usage:");
    println!("  Get value: {}", "noter config get <key>".bright_white());
    println!(
        "  Set value: {}",
        "noter config set <key> <value>".bright_white()
    );
    println!();
    println!("Example:");
    println!("  {}", "noter config get author".bright_white());
    println!(
        "  {}",
        "noter config set author \"John Doe\"".bright_white()
    );
    println!(
        "  {}",
        "noter config set templates.auto_update true".bright_white()
    );

    Ok(())
}

pub fn interactive_config() -> Result<()> {
    use std::io::{self, Write};

    println!("{} Interactive Configuration Wizard", "🧙".blue().bold());
    println!();
    println!("This wizard will help you configure common settings.");
    println!("Press Enter to keep the current value, or type a new value.");
    println!();

    let mut config = get_config()?;

    // Author
    print!("Author name [{}]: ", config.author.green());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        config.author = input.trim().to_string();
    }

    // Preferred Editor
    print!(
        "Preferred editor [{}]: ",
        config
            .preferred_editor
            .as_deref()
            .unwrap_or("None")
            .yellow()
    );
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        config.preferred_editor = Some(input.trim().to_string());
    }

    // Notes directory
    print!("Notes directory [{}]: ", config.paths.notes_dir.cyan());
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        config.paths.notes_dir = input.trim().to_string();
    }

    // Auto open file
    print!(
        "Auto-open files after creation? (y/n) [{}]: ",
        if config.note_preferences.auto_open_file {
            "y".green()
        } else {
            "n".red()
        }
    );
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if !trimmed.is_empty() {
        config.note_preferences.auto_open_file = matches!(trimmed.as_str(), "y" | "yes" | "true");
    }

    // Template auto-update
    print!(
        "Auto-update templates? (y/n) [{}]: ",
        if config.templates.auto_update {
            "y".green()
        } else {
            "n".red()
        }
    );
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if !trimmed.is_empty() {
        config.templates.auto_update = matches!(trimmed.as_str(), "y" | "yes" | "true");
    }

    // Obsidian integration
    print!(
        "Enable Obsidian integration? (y/n) [{}]: ",
        if config.obsidian_integration.enabled {
            "y".green()
        } else {
            "n".red()
        }
    );
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if !trimmed.is_empty() {
        config.obsidian_integration.enabled = matches!(trimmed.as_str(), "y" | "yes" | "true");
    }

    // Save configuration
    config.save()?;

    println!();
    println!("{} Configuration saved successfully!", "✅".green());
    println!();
    println!("You can further customize your configuration with:");
    println!(
        "  {} - View all settings",
        "noter config show".bright_white()
    );
    println!(
        "  {} - List all available keys",
        "noter config list-keys".bright_white()
    );
    println!(
        "  {} - Get a specific value",
        "noter config get <key>".bright_white()
    );
    println!(
        "  {} - Set a specific value",
        "noter config set <key> <value>".bright_white()
    );
    println!(
        "  {} - Edit config file directly",
        "noter config edit".bright_white()
    );

    Ok(())
}

/// Navigate JSON value using dot notation path
fn navigate_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Update JSON value using dot notation path
fn update_json_path(value: &mut Value, path: &str, new_value: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Invalid key path"));
    }

    let mut current = value;

    // Navigate to the parent of the target
    for part in &parts[..parts.len() - 1] {
        match current {
            Value::Object(map) => {
                current = map
                    .get_mut(*part)
                    .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", part))?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Cannot navigate through non-object at '{}'",
                    part
                ));
            }
        }
    }

    // Update the target value
    let last_key = parts[parts.len() - 1];
    match current {
        Value::Object(map) => {
            let old_value = map
                .get(last_key)
                .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", last_key))?;

            // Parse new_value based on the type of old_value
            let parsed_value = match old_value {
                Value::String(_) => Value::String(new_value.to_string()),
                Value::Number(_) => {
                    if let Ok(n) = new_value.parse::<i64>() {
                        serde_json::json!(n)
                    } else if let Ok(n) = new_value.parse::<f64>() {
                        serde_json::json!(n)
                    } else {
                        return Err(anyhow::anyhow!("Invalid number format: {}", new_value));
                    }
                }
                Value::Bool(_) => {
                    let b = match new_value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "y" => true,
                        "false" | "0" | "no" | "n" => false,
                        _ => return Err(anyhow::anyhow!("Invalid boolean format: {}", new_value)),
                    };
                    Value::Bool(b)
                }
                Value::Array(_) => {
                    // Try to parse as JSON array
                    serde_json::from_str(new_value)
                        .map_err(|_| anyhow::anyhow!("Invalid array format: {}", new_value))?
                }
                Value::Object(_) => {
                    // Try to parse as JSON object
                    serde_json::from_str(new_value)
                        .map_err(|_| anyhow::anyhow!("Invalid object format: {}", new_value))?
                }
                Value::Null => Value::String(new_value.to_string()),
            };

            map.insert(last_key.to_string(), parsed_value);
            Ok(())
        }
        _ => Err(anyhow::anyhow!(
            "Cannot set value on non-object at '{}'",
            last_key
        )),
    }
}

/// Collect all JSON keys recursively
fn collect_json_keys(value: &Value, prefix: &str) -> Vec<String> {
    let mut keys = Vec::new();

    if let Value::Object(map) = value {
        for (key, val) in map {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match val {
                Value::Object(_) => {
                    keys.extend(collect_json_keys(val, &full_key));
                }
                Value::Array(_) => {
                    keys.extend(collect_json_keys(val, &full_key));
                }
                _ => {
                    keys.push(full_key);
                }
            }
        }
    }

    keys
}

/// Format a JSON value for inline display
fn format_value_inline(value: &Value) -> String {
    match value {
        Value::String(s) => s.yellow().to_string(),
        Value::Number(n) => n.to_string().cyan().to_string(),
        Value::Bool(b) => {
            if *b {
                "true".bright_green().to_string()
            } else {
                "false".bright_red().to_string()
            }
        }
        Value::Null => "null".bright_black().to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".bright_black().to_string()
            } else {
                format!("[{} items]", arr.len()).bright_black().to_string()
            }
        }
        Value::Object(map) => {
            if map.is_empty() {
                "{}".bright_black().to_string()
            } else {
                format!("{{...}} ({} fields)", map.len())
                    .bright_black()
                    .to_string()
            }
        }
    }
}

pub fn set_author(name: &str) -> Result<()> {
    update_author(name.to_string())?;
    println!("{} Author updated to: {}", "✅".green(), name.green());
    Ok(())
}

pub fn set_editor(editor: &str) -> Result<()> {
    update_editor(Some(editor.to_string()))?;
    println!(
        "{} Preferred editor set to: {}",
        "✅".green(),
        editor.yellow()
    );
    Ok(())
}

pub fn add_template_repository(
    name: &str,
    repository: &str,
    version: Option<&str>,
    template_path: Option<&str>,
) -> Result<()> {
    let mut config = get_config()?;

    // Check if repository already exists
    if config
        .templates
        .custom_repositories
        .iter()
        .any(|r| r.name == name)
    {
        return Err(anyhow::anyhow!(
            "Template repository '{}' already exists",
            name
        ));
    }

    let template_repo = TemplateRepository {
        name: name.to_string(),
        repository: repository.to_string(),
        version: version.map(|v| v.to_string()),
        branch: None,
        template_path: template_path.map(|p| p.to_string()),
        enabled: true,
    };

    config.templates.custom_repositories.push(template_repo);
    config.save()?;

    println!(
        "{} Added template repository: {} ({})",
        "✅".green(),
        name.green(),
        repository.yellow()
    );
    Ok(())
}

pub fn remove_template_repository(name: &str) -> Result<()> {
    let mut config = get_config()?;

    let initial_len = config.templates.custom_repositories.len();
    config
        .templates
        .custom_repositories
        .retain(|r| r.name != name);

    if config.templates.custom_repositories.len() == initial_len {
        return Err(anyhow::anyhow!("Template repository '{}' not found", name));
    }

    config.save()?;
    println!("{} Removed template repository: {}", "🗑️".red(), name);
    Ok(())
}

pub fn enable_template_repository(name: &str, enabled: bool) -> Result<()> {
    let mut config = get_config()?;

    let repo = config
        .templates
        .custom_repositories
        .iter_mut()
        .find(|r| r.name == name)
        .ok_or_else(|| anyhow::anyhow!("Template repository '{}' not found", name))?;

    repo.enabled = enabled;
    config.save()?;

    let status = if enabled { "enabled" } else { "disabled" };
    let emoji = if enabled { "✅" } else { "❌" };
    println!("{} Template repository '{}' {}", emoji, name, status);
    Ok(())
}

pub fn list_template_repositories() -> Result<()> {
    let config = get_config()?;

    if config.templates.custom_repositories.is_empty() {
        println!("{} No custom template repositories configured", "📝".blue());
        println!(
            "Add one with: {}",
            "noter config add-template-repo <name> <owner/repo>".bright_white()
        );
    } else {
        println!("{} Template Repositories:", "📦".blue());
        for repo in &config.templates.custom_repositories {
            let status = if repo.enabled { "✅" } else { "❌" };
            println!(
                "  {} {} ({})",
                status,
                repo.name.green(),
                repo.repository.yellow()
            );
            if let Some(version) = &repo.version {
                println!("    Version: {}", version);
            }
            if let Some(path) = &repo.template_path {
                println!("    Template Path: {}", path);
            }
        }
    }

    if config.templates.use_official_fallback {
        println!("  {} official (fallback)", "🏛️".blue());
    }

    Ok(())
}

pub fn set_template_auto_update(enabled: bool) -> Result<()> {
    let mut config = get_config()?;
    config.templates.auto_update = enabled;
    config.save()?;

    let status = if enabled { "enabled" } else { "disabled" };
    println!("{} Template auto-update {}", "🔄".blue(), status);
    Ok(())
}

pub fn reset_config() -> Result<()> {
    let default_config = Config::default();
    default_config.save()?;
    println!("{} Configuration reset to defaults", "🔄".blue());
    Ok(())
}

pub fn show_config_path() -> Result<()> {
    let path = Config::config_file_path()?;
    println!("{} Config file location:", "📁".blue());
    println!("{}", path.display());
    Ok(())
}

pub fn check_config() -> Result<()> {
    let config = get_config()?;
    let warnings = config.validate()?;

    if warnings.is_empty() {
        println!("{} Configuration is valid!", "✅".green());
    } else {
        println!("{} Configuration warnings:", "⚠️".yellow());
        for warning in warnings {
            println!("  • {}", warning);
        }
    }

    Ok(())
}

pub fn migrate_config() -> Result<()> {
    println!("{} Checking config migration status...", "🔄".blue());

    let config_path = Config::config_file_path()?;

    if !config_path.exists() {
        println!(
            "{} No config file found. Nothing to migrate.",
            "ℹ️".yellow()
        );
        return Ok(());
    }

    // Load config - this will automatically trigger migration if needed
    let config = Config::load()?;

    println!("{} Config is up to date!", "✅".green());
    println!("  Version: {}", config.metadata.config_version);

    if !config.metadata.migration_notes.is_empty() {
        println!("\n{} Migration notes:", "📝".blue());
        println!("  {}", config.metadata.migration_notes);
    }

    Ok(())
}

pub fn cleanse_config(skip_confirmation: bool) -> Result<()> {
    if !skip_confirmation {
        let config = get_config()?;
        let config_path = Config::config_file_path()?;

        OutputManager::print_status(
            Status::Warning,
            "This will completely reset your noter configuration to defaults.",
        );

        println!("Current configuration:");
        println!("  📁 Config file: {}", config_path.display());
        println!("  👤 Author: {}", config.author);
        println!(
            "  📝 Editor: {}",
            config.preferred_editor.as_deref().unwrap_or("None")
        );
        println!("  📂 Notes dir: {}", config.paths.notes_dir);

        use std::io::{self, Write};
        print!("\nAre you sure? Type 'yes' to confirm: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    Config::cleanse()?;

    OutputManager::print_status(
        Status::Success,
        "Configuration cleansed! Fresh defaults have been applied.",
    );

    println!("Next steps:");
    println!("  1. Run: noter config set-author \"Your Name\"");
    println!("  2. Run: noter setup (if needed)");
    println!("  3. Run: noter config show");

    Ok(())
}
