//! Info and status commands
//!
//! Thin command layer that delegates to core status manager.

use anyhow::Result;
use colored::Colorize;

use crate::config::get_config;
use crate::core::status::StatusManager;
use crate::display::output::{OutputManager, Status};

#[allow(dead_code)]
pub fn show_enhanced_status() -> Result<()> {
    let config = get_config()?;

    OutputManager::print_section("DTU Notes Status Dashboard", Some("📊"));

    // Get comprehensive status information
    let system_status = StatusManager::get_system_status(&config)?;
    let activity_summary = StatusManager::get_activity_summary(&config)?;
    let course_health = StatusManager::get_course_health(&config)?;
    let semester_info = StatusManager::get_semester_info(&config);

    // Display system status
    show_system_status_section(&system_status);

    // Display configuration warnings
    if !system_status.configuration_warnings.is_empty() {
        println!();
        println!("{} Configuration Warnings:", "⚠️".yellow());
        for warning in &system_status.configuration_warnings {
            println!("  • {}", warning.yellow());
        }
    }

    // Display activity summary
    show_activity_summary_section(&activity_summary);

    // Display course health
    if !course_health.is_empty() {
        show_course_health_section(&course_health);
    }

    // Show semester info
    println!();
    println!(
        "📅 Current semester: {}",
        semester_info.current_semester.bright_green()
    );

    // Quick suggestions
    println!();
    show_quick_suggestions(&activity_summary)?;

    Ok(())
}

pub fn show_semester() -> Result<()> {
    let config = get_config()?;
    let semester_info = StatusManager::get_semester_info(&config);

    OutputManager::print_section("Semester Information", Some("📅"));

    println!(
        "Current semester: {}",
        semester_info.current_semester.bright_green()
    );
    println!(
        "University: {}",
        "Technical University of Denmark (DTU)".bright_cyan()
    );
    println!("Format: {:?}", semester_info.format);

    println!();
    println!("{} Quick Info:", "ℹ️".blue());
    println!(
        "  Notes directory: {}",
        config.paths.notes_dir.bright_white()
    );
    println!(
        "  Template version: {}",
        config.template_version.bright_white()
    );
    println!("  Author: {}", config.author.bright_white());

    Ok(())
}

#[allow(dead_code)]
pub fn show_status() -> Result<()> {
    let config = get_config()?;
    let system_status = StatusManager::get_system_status(&config)?;

    OutputManager::print_section("DTU Notes Status", Some("📊"));

    // Show system status
    show_system_status_section(&system_status);

    // Show configuration warnings
    if !system_status.configuration_warnings.is_empty() {
        println!();
        println!("{} Configuration Warnings:", "⚠️".yellow());
        for warning in &system_status.configuration_warnings {
            println!("  • {}", warning.yellow());
        }
    }

    // Show course count
    println!();
    if std::path::Path::new(&config.paths.notes_dir).exists() {
        let course_count = count_course_directories(&config.paths.notes_dir)?;
        println!(
            "Courses initialized: {}",
            course_count.to_string().bright_green()
        );
    } else {
        println!("Courses initialized: {}", "0 (run setup first)".yellow());
    }

    // Show next steps
    println!();
    if !std::path::Path::new(&config.paths.notes_dir).exists() {
        println!(
            "{} Run {} to initialize your note-taking environment",
            "💡".yellow(),
            "noter setup".bright_white()
        );
    } else {
        println!(
            "{} Ready to take notes! Try {} to get started",
            "🎉".green(),
            "noter note 02101".bright_white()
        );
    }

    Ok(())
}

#[allow(dead_code)]
pub fn list_courses() -> Result<()> {
    let config = get_config()?;
    let courses = config.list_courses();

    if courses.is_empty() {
        OutputManager::print_status(Status::Info, "No courses configured.");
        println!(
            "Add courses with: {}",
            "noter courses add 02101 \"Introduction to Programming\"".bright_white()
        );
        return Ok(());
    }

    OutputManager::print_section("Your DTU Courses", Some("🎓"));

    for (course_id, course_name) in courses {
        println!("  {} - {}", course_id.yellow(), course_name);
    }

    println!();
    OutputManager::print_command_examples(&[
        ("noter note 02101", "Create a lecture note"),
        (
            "noter assignment 02101 \"Problem Set 1\"",
            "Create assignment",
        ),
        (
            "noter courses add 02103 \"Programming\"",
            "Add a new course",
        ),
        ("noter recent 02101", "List recent notes for course"),
    ]);

    Ok(())
}

// Private helper functions for displaying status sections

fn show_system_status_section(system_status: &crate::core::status::SystemStatus) {
    println!("🏗️ System Status:");
    for (name, exists) in &system_status.directories {
        let status = if *exists { "✅".green() } else { "❌".red() };
        println!("  {}: {}", name, status);
    }

    println!();
    println!("📦 Templates:");
    for (template_path, exists) in &system_status.templates {
        let status = if *exists { "✅".green() } else { "❌".red() };
        let filename = std::path::Path::new(template_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(template_path);
        println!("  {}: {}", status, filename.dimmed());
    }
}

#[allow(dead_code)]
fn show_activity_summary_section(activity_summary: &crate::core::status::ActivitySummary) {
    println!();
    println!("📈 Recent Activity:");

    if activity_summary.total_notes == 0 && activity_summary.total_assignments == 0 {
        println!("  No activity (run setup first)");
        return;
    }

    println!(
        "  Total files: {} notes, {} assignments",
        activity_summary.total_notes.to_string().green(),
        activity_summary.total_assignments.to_string().blue()
    );

    if let Some(ref recent) = activity_summary.most_recent_activity {
        let datetime: chrono::DateTime<chrono::Local> = recent.timestamp.into();
        println!(
            "  Last activity: {} ({} - {})",
            datetime.format("%Y-%m-%d %H:%M").to_string().bright_white(),
            recent.course_id.yellow(),
            recent.course_name.dimmed()
        );
        println!("  File: {}", recent.file_name.dimmed());
    }

    if let Some((course_id, count)) = &activity_summary.most_active_course {
        println!(
            "  Most active: {} ({} files)",
            course_id.yellow(),
            count.to_string().green()
        );
    }
}

#[allow(dead_code)]
fn show_course_health_section(course_health: &[crate::core::status::CourseHealthInfo]) {
    println!();
    println!("🎓 Course Health:");

    for health_info in course_health {
        let health_indicator = match health_info.health_status {
            crate::core::status::HealthStatus::Excellent => "✅",
            crate::core::status::HealthStatus::Good => "⚠️",
            crate::core::status::HealthStatus::Warning => "🔴",
            crate::core::status::HealthStatus::Critical => "❌",
        };

        let last_activity = match health_info.days_since_last_activity {
            0 => "today".bright_green(),
            1 => "1 day ago".green(),
            2..=7 => format!("{} days ago", health_info.days_since_last_activity).yellow(),
            8..=14 => format!("{} days ago", health_info.days_since_last_activity).red(),
            999 => "never".red(),
            _ => format!("{} days ago", health_info.days_since_last_activity).red(),
        };

        println!(
            "  {} {} - {} ({} notes, {} assignments, last: {})",
            health_indicator,
            health_info.course_id.yellow(),
            health_info.course_name.dimmed(),
            health_info.notes_count,
            health_info.assignments_count,
            last_activity
        );
    }
}

#[allow(dead_code)]
fn show_quick_suggestions(activity_summary: &crate::core::status::ActivitySummary) -> Result<()> {
    println!("💡 Quick Suggestions:");

    if let Some((course_id, _)) = &activity_summary.most_active_course {
        OutputManager::print_command_examples(&[
            (
                &format!("noter note {}", course_id),
                "Continue with most active course",
            ),
            (&format!("noter open {}", course_id), "Open recent note"),
            ("noter recent", "See all recent activity"),
        ]);
    } else {
        OutputManager::print_command_examples(&[
            (
                "noter courses add 02101 \"Course Name\"",
                "Add your first course",
            ),
            ("noter note 02101", "Create your first note"),
            ("noter setup", "Run setup if needed"),
        ]);
    }

    Ok(())
}

// Simple helper function for basic course counting (kept for backward compatibility)
#[allow(dead_code)]
fn count_course_directories(notes_dir: &str) -> Result<usize> {
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(notes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                // Check if it looks like a course code (5 digits)
                if let Some(name) = entry.file_name().to_str() {
                    if name.len() == 5 && name.chars().all(|c| c.is_ascii_digit()) {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}
