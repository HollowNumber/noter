//! Typst compilation commands
//!
//! Thin command layer that delegates to core typst compiler.

use anyhow::Result;
use colored::Colorize;

use crate::config::get_config;
use crate::core::typst::{CompilationStatus, TypstCompiler};
use crate::display::output::{OutputManager, Status};

pub fn compile_file(filepath: &str) -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(
        Status::Loading,
        &format!("Compiling {}", filepath.bright_white()),
    );

    match TypstCompiler::compile_file(filepath, &config) {
        Ok(output_path) => {
            OutputManager::print_status(
                Status::Success,
                &format!("Compiled successfully: {}", output_path.bright_green()),
            );

            // Show file size if available
            if let Ok(metadata) = std::fs::metadata(&output_path) {
                let size_kb = metadata.len() / 1024;
                println!("File size: {} KB", size_kb.to_string().dimmed());
            }

            // Auto-open the compiled PDF if configured to do so
            if config.note_preferences.auto_open_file {
                OutputManager::print_status(Status::Info, "Opening compiled PDF...");
                if let Err(e) = opener::open(&output_path) {
                    OutputManager::print_status(
                        Status::Warning,
                        &format!("Could not open PDF automatically: {}", e),
                    );
                }
            } else {
                println!("PDF created at: {}", output_path);
            }

            // Show helpful next steps
            OutputManager::print_command_examples(&[
                (&format!("noter watch {}", filepath), "Watch for changes"),
                (&format!("opener {}", output_path), "Open PDF manually"),
            ]);
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Compilation failed: {}", e));

            if e.to_string().contains("not found") {
                println!(
                    "Make sure Typst is installed: {}",
                    "https://github.com/typst/typst#installation".bright_blue()
                );
            }
        }
    }

    Ok(())
}

pub fn watch_file(filepath: &str) -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(
        Status::Info,
        &format!("Watching {} for changes...", filepath.bright_white()),
    );

    println!("Press {} to stop", "Ctrl+C".yellow());

    match TypstCompiler::watch_file(filepath, &config) {
        Ok(_) => {
            OutputManager::print_status(Status::Info, "Watch stopped");
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Watch failed: {}", e));
        }
    }

    Ok(())
}

pub fn clean_files() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, "Cleaning compiled files...");

    match TypstCompiler::clean_files(&config) {
        Ok(cleaned_count) => {
            if cleaned_count > 0 {
                OutputManager::print_status(
                    Status::Success,
                    &format!("Cleaned {} PDF files", cleaned_count),
                );
            } else {
                OutputManager::print_status(Status::Info, "No PDF files found to clean");
            }
        }
        Err(_e) => {
            // Silently ignore errors when cleaning PDFs
        }
    }

    Ok(())
}

/// Check compilation status before compiling
pub fn check_compilation_status(filepath: &str) -> Result<()> {
    let config = get_config()?;

    match TypstCompiler::get_compilation_status(filepath, &config) {
        Ok(status) => {
            let (icon, status_text, should_compile) = match status {
                CompilationStatus::UpToDate => ("🟢", "Up to date", false),
                CompilationStatus::OutOfDate => ("🟡", "Out of date", true),
                CompilationStatus::NotCompiled => ("🔴", "Not compiled", true),
                CompilationStatus::SourceNotFound => ("❌", "Source not found", false),
            };

            println!("{} {} - {}", icon, filepath.bright_white(), status_text);

            if should_compile {
                println!("📋 Compilation recommended");
            }
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Status check failed: {}", e));
        }
    }

    Ok(())
}

/// Check status of a specific file
pub fn check_file_status(filepath: &str, detailed: bool) -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(
        Status::Loading,
        &format!("Checking status of {}", filepath.bright_white()),
    );

    match TypstCompiler::get_compilation_status(filepath, &config) {
        Ok(status) => {
            println!();
            println!("📊 Compilation Status: {}", filepath.bright_white());
            println!();

            let (icon, status_text, color_fn): (_, _, fn(&str) -> colored::ColoredString) =
                match status {
                    CompilationStatus::UpToDate => {
                        ("🟢", "Up to date", |s: &str| s.bright_green())
                    }
                    CompilationStatus::OutOfDate => {
                        ("🟡", "Out of date - needs recompilation", |s: &str| {
                            s.bright_yellow()
                        })
                    }
                    CompilationStatus::NotCompiled => {
                        ("🔴", "Not compiled - PDF missing", |s: &str| {
                            s.bright_red()
                        })
                    }
                    CompilationStatus::SourceNotFound => {
                        ("❌", "Source file not found", |s: &str| s.bright_red())
                    }
                };

            println!("Status: {} {}", icon, color_fn(status_text));

            if detailed {
                // Show file information
                use std::path::Path;
                let input_path = Path::new(filepath);
                let mut output_path = input_path.with_extension("pdf");

                #[allow(clippy::nonminimal_bool)]
                if !input_path.extension().is_none_or(|ext| ext != "typ") {
                    let mut typ_path = input_path.to_path_buf();
                    typ_path.set_extension("typ");
                    output_path = typ_path.with_extension("pdf");
                }

                println!();
                println!("📁 File Details:");
                println!("  Source: {}", input_path.display());
                println!("  Output: {}", output_path.display());

                if input_path.exists() {
                    if let Ok(metadata) = std::fs::metadata(input_path) {
                        if let Ok(modified) = metadata.modified() {
                            let datetime: chrono::DateTime<chrono::Local> = modified.into();
                            println!("  Modified: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
                        }
                    }
                }

                if output_path.exists() {
                    if let Ok(metadata) = std::fs::metadata(&output_path) {
                        if let Ok(modified) = metadata.modified() {
                            let datetime: chrono::DateTime<chrono::Local> = modified.into();
                            println!("  PDF created: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
                        }

                        let size = metadata.len();
                        println!("  PDF size: {:.1} KB", size as f64 / 1024.0);
                    }
                } else {
                    println!("  PDF: Not generated");
                }
            }

            // Provide recommendations
            println!();
            match status {
                CompilationStatus::OutOfDate | CompilationStatus::NotCompiled => {
                    println!("💡 Recommended actions:");
                    println!(
                        "  • {}",
                        format!("noter compile {}", filepath).bright_white()
                    );
                    println!("  • {}", format!("noter watch {}", filepath).bright_white());
                }
                CompilationStatus::UpToDate => {
                    println!("✅ No action needed - file is up to date");
                }
                CompilationStatus::SourceNotFound => {
                    println!("❌ Cannot compile - source file not found");
                }
            }
        }
        Err(e) => {
            OutputManager::print_status(Status::Error, &format!("Status check failed: {}", e));
        }
    }

    Ok(())
}

/// Check status of all Typst files in the workspace
pub fn check_all_files(detailed: bool) -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, "Scanning for Typst files...");

    use crate::core::files::FileOperations;
    use std::path::Path;

    let mut all_files = Vec::new();
    let notes_dir = Path::new(&config.paths.notes_dir);

    if notes_dir.exists() {
        if let Ok(files) =
            FileOperations::list_files_with_extensions(notes_dir.to_str().unwrap(), &["typ"])
        {
            all_files.extend(files);
        }
    }

    if all_files.is_empty() {
        OutputManager::print_status(Status::Info, "No Typst files found in workspace");
        return Ok(());
    }

    println!();
    println!("📊 Compilation Status Summary ({} files)", all_files.len());
    println!();

    let mut status_counts = std::collections::HashMap::new();
    let mut files_by_status = std::collections::HashMap::new();

    for file_path in &all_files {
        if let Ok(status) =
            TypstCompiler::get_compilation_status(file_path.to_str().unwrap(), &config)
        {
            *status_counts.entry(status.clone()).or_insert(0) += 1;
            files_by_status
                .entry(status)
                .or_insert_with(Vec::new)
                .push(file_path.clone());
        }
    }

    // Show summary
    for (status, status_text) in [
        (CompilationStatus::UpToDate, "🟢 Up to date"),
        (CompilationStatus::OutOfDate, "🟡 Out of date"),
        (CompilationStatus::NotCompiled, "🔴 Not compiled"),
        (CompilationStatus::SourceNotFound, "❌ Source missing"),
    ] {
        if let Some(count) = status_counts.get(&status) {
            println!("{}: {} files", status_text, count);
        }
    }

    if detailed {
        println!();

        // Show files that need attention first
        for (status, status_name) in [
            (
                CompilationStatus::OutOfDate,
                "🟡 Files needing recompilation",
            ),
            (CompilationStatus::NotCompiled, "🔴 Uncompiled files"),
        ] {
            if let Some(files) = files_by_status.get(&status) {
                if !files.is_empty() {
                    println!();
                    println!("{}:", status_name);
                    for file in files {
                        let relative_path = file
                            .strip_prefix(&config.paths.notes_dir)
                            .unwrap_or(file)
                            .display()
                            .to_string();
                        println!("  • {}", relative_path);
                    }
                }
            }
        }
    }

    // Show recommended actions
    println!();
    let needs_compilation = status_counts
        .get(&CompilationStatus::OutOfDate)
        .unwrap_or(&0)
        + status_counts
            .get(&CompilationStatus::NotCompiled)
            .unwrap_or(&0);

    if needs_compilation > 0 {
        println!("💡 Recommended actions:");
        println!(
            "  • Compile all out-of-date files: {}",
            "find with 'noter check --detailed' and compile individually".bright_white()
        );
        if needs_compilation <= 3 {
            // Show specific commands for small numbers
            for (status, files) in &files_by_status {
                if matches!(
                    status,
                    CompilationStatus::OutOfDate | CompilationStatus::NotCompiled
                ) {
                    for file in files.iter().take(3) {
                        println!(
                            "  • {}",
                            format!("noter compile {}", file.to_string_lossy()).bright_white()
                        );
                    }
                }
            }
        }
    } else {
        println!("✅ All files are up to date!");
    }

    Ok(())
}
