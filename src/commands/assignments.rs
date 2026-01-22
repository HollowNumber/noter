//! Assignment command implementation
//!
//! Uses the template system directly for assignment creation and management.

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::config::get_config;
use crate::core::files::FileOperations;
use crate::core::template::{builder::TemplateBuilder, engine::TemplateReference};
use crate::core::validation::Validator;
use crate::display::output::{OutputManager, Status};

/// Create a new assignment using the template system
pub fn create_assignment(course_id: &str, title: &str) -> Result<()> {
    let config = get_config()?;

    // Validate course ID
    Validator::validate_course_id(course_id)?;

    let course_name = config.get_course_name(course_id);
    if course_name.is_empty() {
        OutputManager::print_status(
            Status::Error,
            &format!(
                "Course {} not found in configuration. Add it first with 'noter courses add'",
                course_id
            ),
        );
        return Ok(());
    }

    OutputManager::print_status(
        Status::Loading,
        &format!("Creating assignment for course {}", course_id.yellow()),
    );

    // Create assignment directory if it doesn't exist
    let assignments_dir = Path::new(&config.paths.notes_dir)
        .join(course_id)
        .join("assignments");

    if let Err(e) = fs::create_dir_all(&assignments_dir) {
        OutputManager::print_status(
            Status::Error,
            &format!("Failed to create assignments directory: {}", e),
        );
        return Ok(());
    }

    // Generate filename
    let sanitized_title = Validator::sanitize_filename(title);
    let filename = format!("{}.typ", sanitized_title);
    let file_path = assignments_dir.join(&filename);

    // Check if file already exists
    if file_path.exists() {
        if config.note_preferences.create_backups {
            let backup_path = file_path.with_extension("typ.bak");
            if let Err(e) = fs::copy(&file_path, backup_path) {
                OutputManager::print_status(
                    Status::Warning,
                    &format!("Could not create backup: {}", e),
                );
            }
        } else {
            OutputManager::print_status(
                Status::Error,
                &format!("Assignment file already exists: {}", file_path.display()),
            );
            return Ok(());
        }
    }

    // Generate content using the template system
    match TemplateBuilder::new(course_id, &config)?
        .with_reference(TemplateReference::assignment())
        .with_title(title)
        .with_sections(config.note_preferences.assignment_sections.clone())
        .build()
    {
        Ok(content) => {
            // Write file
            if let Err(e) = fs::write(&file_path, content) {
                OutputManager::print_status(
                    Status::Error,
                    &format!("Failed to write assignment file: {}", e),
                );
                return Ok(());
            }

            OutputManager::print_status(
                Status::Success,
                &format!(
                    "Assignment created: {}",
                    file_path.to_string_lossy().bright_white()
                ),
            );

            // Auto-open if configured
            if config.note_preferences.auto_open_file {
                OutputManager::print_status(Status::Info, "Opening in editor...");
                if let Err(e) = FileOperations::open_file(&file_path, &config) {
                    OutputManager::print_status(
                        Status::Warning,
                        &format!("Could not open file automatically: {}", e),
                    );
                }
            }

            // Show helpful next steps
            println!();
            OutputManager::print_command_examples(&[
                (
                    &format!("noter compile {}", file_path.to_string_lossy()),
                    "Compile to PDF",
                ),
                (
                    &format!("noter watch {}", file_path.to_string_lossy()),
                    "Auto-compile on changes",
                ),
                (&format!("noter recent {}", course_id), "List recent files"),
            ]);
        }
        Err(e) => {
            OutputManager::print_status(
                Status::Error,
                &format!("Failed to generate assignment template: {}", e),
            );

            if e.to_string().contains("template") || e.to_string().contains("Template") {
                println!();
                println!("This might be because:");
                println!("  • Templates haven't been installed yet");
                println!("  • Template configuration is missing or invalid");
                println!();
                println!("Try: {}", "noter template update".bright_white());
            }
        }
    }

    Ok(())
}

/// List recent assignments for a course
pub fn list_recent_assignments(course_id: &str, limit: usize) -> Result<()> {
    let config = get_config()?;

    // Validate course ID
    Validator::validate_course_id(course_id)?;

    OutputManager::print_status(
        Status::Loading,
        &format!("Finding recent assignments for {}", course_id.yellow()),
    );

    let assignments_dir = Path::new(&config.paths.notes_dir)
        .join(course_id)
        .join("assignments");

    if !assignments_dir.exists() {
        println!(
            "{} No assignments found for course {}",
            "📝".dimmed(),
            course_id.yellow()
        );
        println!(
            "Create one: {}",
            format!("noter assignment {} \"Assignment Title\"", course_id).bright_white()
        );
        return Ok(());
    }

    // Collect assignment files with modification times
    let mut files = Vec::new();
    match fs::read_dir(&assignments_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "typ") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            files.push((path.to_string_lossy().to_string(), modified));
                        }
                    }
                }
            }
        }
        Err(e) => {
            OutputManager::print_status(
                Status::Error,
                &format!("Failed to read assignments directory: {}", e),
            );
            return Ok(());
        }
    }

    // Sort by modification time (newest first)
    files.sort_by(|a, b| b.1.cmp(&a.1));

    let assignments: Vec<String> = files
        .into_iter()
        .take(limit)
        .map(|(path, _)| path)
        .collect();

    if assignments.is_empty() {
        println!(
            "{} No assignments found for course {}",
            "📝".dimmed(),
            course_id.yellow()
        );
        println!(
            "Create one: {}",
            format!("noter assignment {} \"Assignment Title\"", course_id).bright_white()
        );
    } else {
        println!();
        println!(
            "{} Recent assignments for {}:",
            "📝".blue(),
            course_id.yellow()
        );
        println!();

        for (i, assignment_path) in assignments.iter().enumerate() {
            let file_name = std::path::Path::new(assignment_path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();

            println!(
                "  {}. {}",
                (i + 1).to_string().bright_white(),
                file_name.green()
            );
            println!("     {}", assignment_path.dimmed());
        }

        println!();
        OutputManager::print_command_examples(&[
            (&format!("noter open {}", course_id), "Open most recent"),
            (
                &format!("noter assignment {} \"New Assignment\"", course_id),
                "Create new assignment",
            ),
        ]);
    }

    Ok(())
}

/// Show assignment statistics for a course
pub fn show_assignment_stats(course_id: &str) -> Result<()> {
    let config = get_config()?;

    // Validate course ID
    Validator::validate_course_id(course_id)?;

    OutputManager::print_status(
        Status::Loading,
        &format!("Calculating assignment stats for {}", course_id.yellow()),
    );

    let assignments_dir = Path::new(&config.paths.notes_dir)
        .join(course_id)
        .join("assignments");

    let (count, last_modified) = if !assignments_dir.exists() {
        (0, None)
    } else {
        get_assignment_stats_for_directory(&assignments_dir)?
    };

    println!();
    println!(
        "{} Assignment Statistics for {}",
        "📊".blue(),
        course_id.yellow()
    );
    println!();

    println!("Total assignments: {}", count.to_string().bright_green());

    if let Some(last_modified) = last_modified {
        let datetime: chrono::DateTime<chrono::Local> = last_modified.into();
        println!(
            "Last modified: {}",
            datetime.format("%Y-%m-%d %H:%M").to_string().bright_white()
        );

        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(last_modified) {
            let days = duration.as_secs() / (24 * 60 * 60);
            let health = match days {
                0..=3 => format!("{} Excellent - recent activity", "🟢".green()),
                4..=7 => format!("{} Good - somewhat recent", "🟡".yellow()),
                8..=14 => format!("{} Warning - getting old", "🟠".yellow()),
                _ => format!("{} Critical - very old", "🔴".red()),
            };
            println!("Activity health: {}", health);
        }
    } else {
        println!("Last modified: {}", "Never".dimmed());
        println!("Activity health: {} Critical - no assignments", "🔴".red());
    }

    println!();
    OutputManager::print_command_examples(&[
        (
            &format!("noter assignments recent {}", course_id),
            "List recent assignments",
        ),
        (
            &format!("noter assignment {} \"New Assignment\"", course_id),
            "Create new assignment",
        ),
    ]);

    Ok(())
}

/// List all assignments across courses with activity summary
pub fn list_all_assignments() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_status(Status::Loading, "Scanning all assignments...");

    let mut total_assignments = 0;
    let mut course_assignments = Vec::new();

    for (course_id, course_name) in config.list_courses() {
        let assignments_dir = Path::new(&config.paths.notes_dir)
            .join(&course_id)
            .join("assignments");

        if let Ok((count, last_modified)) = get_assignment_stats_for_directory(&assignments_dir) {
            total_assignments += count;
            if count > 0 {
                course_assignments.push((course_id, course_name, count, last_modified));
            }
        }
    }

    println!();
    println!("{} Assignment Summary", "📋".blue());
    println!();

    if total_assignments == 0 {
        OutputManager::print_status(Status::Info, "No assignments found.");
        println!(
            "Create your first assignment with: {}",
            "noter assignment 02101 \"Problem Set 1\"".bright_white()
        );
        return Ok(());
    }

    println!(
        "Total assignments: {}",
        total_assignments.to_string().bright_green()
    );
    println!();

    // Sort by most recent activity
    course_assignments.sort_by(|a, b| match (a.3, b.3) {
        (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.0.cmp(&b.0),
    });

    println!("{} Assignments by Course:", "📚".green());
    for (course_id, course_name, count, last_modified) in course_assignments {
        let activity_indicator = if let Some(last_modified) = last_modified {
            let now = std::time::SystemTime::now();
            if let Ok(duration) = now.duration_since(last_modified) {
                let days = duration.as_secs() / (24 * 60 * 60);
                match days {
                    0..=3 => "🟢".to_string(),
                    4..=7 => "🟡".to_string(),
                    8..=14 => "🟠".to_string(),
                    _ => "🔴".to_string(),
                }
            } else {
                "❓".to_string()
            }
        } else {
            "⚫".to_string()
        };

        println!(
            "  {} {} - {} ({})",
            activity_indicator,
            course_id.bright_blue(),
            course_name,
            format!("{} assignments", count).dimmed()
        );
    }

    println!();
    OutputManager::print_command_examples(&[
        (
            "noter assignments recent 02101",
            "Recent assignments for course",
        ),
        ("noter assignments stats 02101", "Detailed stats for course"),
        ("noter assignments health", "Assignment health analysis"),
    ]);

    Ok(())
}

/// Show assignment health and activity analysis
pub fn show_assignment_health(course_id: Option<&str>) -> Result<()> {
    let config = get_config()?;

    let message = if let Some(course_id) = course_id {
        format!("Analyzing assignment health for {}", course_id.yellow())
    } else {
        "Analyzing assignment health for all courses".to_string()
    };

    OutputManager::print_status(Status::Loading, &message);

    let mut health_data = Vec::new();

    let courses_to_check = if let Some(specific_course) = course_id {
        vec![(
            specific_course.to_string(),
            config
                .courses
                .get(specific_course)
                .cloned()
                .unwrap_or_else(|| "Unknown Course".to_string()),
        )]
    } else {
        config.list_courses()
    };

    for (course_id, course_name) in courses_to_check {
        let assignments_dir = Path::new(&config.paths.notes_dir)
            .join(&course_id)
            .join("assignments");

        if let Ok((count, last_modified)) = get_assignment_stats_for_directory(&assignments_dir) {
            let health_status = calculate_assignment_health_status(count, last_modified);
            let days_since_activity = if let Some(last_modified) = last_modified {
                let now = std::time::SystemTime::now();
                now.duration_since(last_modified)
                    .map(|d| d.as_secs() / (24 * 60 * 60))
                    .unwrap_or(999)
            } else {
                999 // No activity
            };

            health_data.push((
                course_id,
                course_name,
                count,
                days_since_activity,
                health_status,
            ));
        }
    }

    if health_data.is_empty() {
        OutputManager::print_status(Status::Info, "No assignment data found.");
        return Ok(());
    }

    println!();
    println!("{} Assignment Health Analysis", "🏥".blue());
    println!();

    // Sort by health status and activity
    health_data.sort_by(|a, b| {
        // Sort by health (0 = best, 3 = worst), then by days since activity
        let health_cmp = health_status_to_priority(a.4).cmp(&health_status_to_priority(b.4));
        if health_cmp == std::cmp::Ordering::Equal {
            a.3.cmp(&b.3) // Less days is better
        } else {
            health_cmp
        }
    });

    // Clone health_data for recommendations before consuming it
    let critical_courses: Vec<_> = health_data
        .iter()
        .filter(|(_, _, _, _, health)| *health >= 3)
        .map(|(course_id, _, count, _, health)| (course_id.clone(), *count, *health))
        .collect();

    for (course_id, course_name, count, days_since, health_status) in health_data {
        let (icon, status_text, color_fn): (_, _, fn(&str) -> colored::ColoredString) =
            match health_status {
                0 => ("🟢", "Excellent", |s: &str| s.bright_green()),
                1 => ("🟡", "Good", |s: &str| s.bright_yellow()),
                2 => ("🟠", "Warning", |s: &str| s.yellow()),
                _ => ("🔴", "Critical", |s: &str| s.bright_red()),
            };

        let activity_text = if days_since >= 999 {
            "no activity".dimmed()
        } else if days_since == 0 {
            "active today".bright_green()
        } else if days_since == 1 {
            "active yesterday".green()
        } else {
            format!("active {} days ago", days_since).dimmed()
        };

        println!(
            "  {} {} {} - {} ({} assignments, {})",
            icon,
            color_fn(status_text),
            course_id.bright_blue(),
            course_name,
            count,
            activity_text
        );
    }

    println!();

    // Provide recommendations
    if !critical_courses.is_empty() {
        println!("{} Recommendations:", "💡".yellow());
        for (course_id, count, _) in &critical_courses {
            if *count == 0 {
                println!(
                    "  • Create first assignment for {}: {}",
                    course_id.bright_blue(),
                    format!("noter assignment {} \"Assignment 1\"", course_id).bright_white()
                );
            } else {
                println!(
                    "  • Resume work on {}: {}",
                    course_id.bright_blue(),
                    format!("noter assignments recent {}", course_id).bright_white()
                );
            }
        }
        println!();
    }

    OutputManager::print_command_examples(&[
        ("noter assignments list", "Overview of all assignments"),
        (
            "noter assignments recent 02101",
            "Recent assignments for course",
        ),
        (
            "noter assignment 02101 \"New Assignment\"",
            "Create new assignment",
        ),
    ]);

    Ok(())
}

// Helper functions

/// Get assignment statistics for a directory
fn get_assignment_stats_for_directory(
    assignments_dir: &Path,
) -> Result<(usize, Option<std::time::SystemTime>)> {
    if !assignments_dir.exists() {
        return Ok((0, None));
    }

    let mut count = 0;
    let mut most_recent = None;

    for entry in fs::read_dir(assignments_dir)? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|ext| ext == "typ") {
            count += 1;

            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    match most_recent {
                        None => most_recent = Some(modified),
                        Some(prev_time) => {
                            if modified > prev_time {
                                most_recent = Some(modified);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((count, most_recent))
}

fn calculate_assignment_health_status(
    count: usize,
    last_modified: Option<std::time::SystemTime>,
) -> usize {
    if count == 0 {
        return 3; // Critical - no assignments
    }

    last_modified.map_or(3, |last_modified| {
        let now = std::time::SystemTime::now();
        now.duration_since(last_modified).map_or(3, |duration| {
            let days = duration.as_secs() / (24 * 60 * 60);
            match days {
                0..=3 => 0,  // Excellent
                4..=7 => 1,  // Good
                8..=14 => 2, // Warning
                _ => 3,      // Critical
            }
        })
    })
}

const fn health_status_to_priority(health: usize) -> usize {
    health // 0 = best, 3 = worst
}
