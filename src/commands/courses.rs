//! Course management commands
//!
//! Thin command layer that delegates to core business logic.

use crate::config::get_config;
use crate::core::courses::{CourseManager, get_common_courses};
use crate::core::validation::Validator;
use crate::display::formatters::Formatters;
use crate::display::output::{OutputManager, Status};
use anyhow::Result;
use colored::Colorize;

pub fn list_courses() -> Result<()> {
    let config = get_config()?;
    let courses = config.list_courses();

    let formatted_output = Formatters::format_course_list(&courses);
    println!("{}", formatted_output);

    if !courses.is_empty() {
        print_usage_examples();
    } else {
        println!(
            "Add courses with: {}",
            "noter courses add 02101 \"Introduction to Programming\"".bright_white()
        );
    }

    Ok(())
}

pub fn add_course(course_id: &str, course_name: &str) -> Result<()> {
    Validator::validate_course_id(course_id)?;

    let mut config = get_config()?;
    let mut manager = CourseManager::new(&mut config);

    match manager.add_course(course_id, course_name) {
        Ok(()) => {
            OutputManager::print_status(
                Status::Success,
                &format!(
                    "Added course: {} - {}",
                    course_id.yellow(),
                    course_name.green()
                ),
            );
            println!(
                "You can now create notes with: {}",
                format!("noter note {}", course_id).bright_white()
            );
        }
        Err(e) => {
            OutputManager::print_status(Status::Warning, &e.to_string());
            println!("Use a different course ID or remove the existing one first.");
        }
    }

    Ok(())
}

pub fn remove_course(course_id: &str) -> Result<()> {
    Validator::validate_course_id(course_id)?;

    let mut config = get_config()?;
    let mut manager = CourseManager::new(&mut config);

    match manager.remove_course(course_id) {
        Ok(course_name) => {
            OutputManager::print_status(
                Status::Success,
                &format!(
                    "Removed course: {} - {}",
                    course_id.yellow(),
                    course_name.dimmed()
                ),
            );
        }
        Err(_) => {
            OutputManager::print_status(
                Status::Error,
                &format!(
                    "Course {} not found in your configuration.",
                    course_id.yellow()
                ),
            );
            println!(
                "Use {} to see available courses.",
                "noter courses list".bright_white()
            );
        }
    }

    Ok(())
}

pub fn browse_common_courses() -> Result<()> {
    let config = get_config()?;
    let user_courses: std::collections::HashSet<String> = config.courses.keys().cloned().collect();
    let dtu_courses = crate::data::get_common_dtu_courses();

    OutputManager::print_section("DTU Course Database", Some("🎓"));

    // Show user's current courses first
    if !user_courses.is_empty() {
        println!("{} Your configured courses:", "✅".green());
        let mut user_course_list: Vec<_> = config.courses.iter().collect();
        user_course_list.sort_by_key(|&(id, _)| id);

        for (course_id, course_name) in user_course_list {
            println!("  {} - {}", course_id.bright_green(), course_name.dimmed());
        }
        println!();
    }

    // Show available DTU courses by categories
    println!("{} Available DTU courses:", "📚".blue());
    let categories = get_common_courses();
    for (category, courses) in categories {
        println!("{}:", category.bright_cyan());
        for (course_id, course_name) in *courses {
            if user_courses.contains(*course_id) {
                // Already configured - show dimmed
                println!(
                    "  {} - {} {}",
                    course_id.dimmed(),
                    course_name.dimmed(),
                    "✓".green()
                );
            } else {
                // Available to add
                println!("  {} - {}", course_id.yellow(), course_name);
            }
        }
        println!();
    }

    // Suggest courses from DTU database not in categories
    let category_courses: std::collections::HashSet<&str> = categories
        .iter()
        .flat_map(|(_, courses)| courses.iter().map(|(id, _)| *id))
        .collect();

    let additional_courses: Vec<_> = dtu_courses
        .iter()
        .filter(|(id, _)| !category_courses.contains(*id) && !user_courses.contains(**id))
        .collect();

    if !additional_courses.is_empty() {
        println!("{} More DTU courses:", "💡".blue());
        let mut sorted_additional: Vec<_> = additional_courses.into_iter().collect();
        sorted_additional.sort_by_key(|(id, _)| *id);

        for (course_id, course_name) in sorted_additional.into_iter().take(10) {
            println!("  {} - {}", course_id.yellow(), course_name);
        }
        if dtu_courses.len() - category_courses.len() > 10 {
            println!(
                "  {} ... and {} more courses",
                "".dimmed(),
                (dtu_courses.len() - category_courses.len() - 10)
                    .to_string()
                    .dimmed()
            );
        }
        println!();
    }

    print_quick_add_examples();
    Ok(())
}

fn print_usage_examples() {
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
        ("noter recent 02101", "List recent notes"),
    ]);
}

fn print_quick_add_examples() {
    OutputManager::print_command_examples(&[
        (
            "noter courses add 02101 \"Introduction to Programming\"",
            "",
        ),
        (
            "noter courses add 01005 \"Advanced Engineering Mathematics 1\"",
            "",
        ),
        ("noter courses add 25200 \"Classical Physics 1\"", ""),
    ]);

    println!();
    println!(
        "Use {} to see your configured courses.",
        "noter courses list".bright_white()
    );
}
