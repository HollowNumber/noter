//! Command execution and routing
//!
//! This module serves as the main entry point for all CLI commands,
//! routing them to appropriate command handlers while maintaining
//! clean separation between command parsing and business logic.

use anyhow::{Context, Result};

pub mod assignments;
pub mod config;
pub mod courses;
pub mod info;
pub mod notes;
pub mod search;
pub mod setup;
pub mod templates;
pub mod typst;

use crate::{AssignmentAction, Commands, ConfigAction, CourseAction, SetupAction, TemplateAction};

#[cfg(feature = "dev-tools")]
use crate::DevAction;

/// Execute a command with proper error context
pub fn execute_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Note {
            course_id,
            title,
            variant,
            sections,
            no_open,
        } => notes::create_note(course_id, title, variant, sections, no_open)
            .with_context(|| format!("Failed to create note for course {}", course_id)),
        Commands::Assignment { course_id, title } => {
            assignments::create_assignment(course_id, title).with_context(|| {
                format!(
                    "Failed to create assignment '{}' for course {}",
                    title, course_id
                )
            })
        }
        Commands::Compile {
            filepath,
            check_status,
        } => {
            if *check_status {
                typst::check_compilation_status(filepath)
                    .with_context(|| format!("Failed to check compilation status: {}", filepath))?;
            }
            typst::compile_file(filepath)
                .with_context(|| format!("Failed to compile file: {}", filepath))
        }
        Commands::Check { filepath, detailed } => {
            if let Some(filepath) = filepath {
                typst::check_file_status(filepath, *detailed)
                    .with_context(|| format!("Failed to check file status: {}", filepath))
            } else {
                typst::check_all_files(*detailed).with_context(|| "Failed to check all files")
            }
        }
        Commands::Watch { filepath } => typst::watch_file(filepath)
            .with_context(|| format!("Failed to watch file: {}", filepath)),
        Commands::Recent { course_id } => notes::list_recent(course_id)
            .with_context(|| format!("Failed to list recent notes for course {}", course_id)),
        Commands::Setup { action } => {
            if let Some(action) = action {
                execute_setup_action(action).with_context(|| "Failed to execute setup command")
            } else {
                setup::setup_repository().with_context(|| "Failed to setup repository")
            }
        }
        Commands::Index { course_id } => notes::create_index(course_id)
            .with_context(|| format!("Failed to create index for course {}", course_id)),
        Commands::Search { query } => {
            search::search_notes(query).with_context(|| format!("Failed to search for: {}", query))
        }
        Commands::RebuildIndex { force } => {
            search::rebuild_index(*force).with_context(|| "Failed to rebuild search index")
        }
        Commands::Assignments { action } => execute_assignment_action(action)
            .with_context(|| "Failed to execute assignment command"),
        Commands::Courses { action } => {
            execute_course_action(action).with_context(|| "Failed to execute course command")
        }
        Commands::Clean => typst::clean_files().with_context(|| "Failed to clean compiled files"),
        Commands::Status => {
            info::show_enhanced_status().with_context(|| "Failed to show status information")
        }
        Commands::Open { course_id } => notes::open_recent(course_id)
            .with_context(|| format!("Failed to open recent note for course {}", course_id)),
        Commands::Semester => {
            info::show_semester().with_context(|| "Failed to show semester information")
        }
        Commands::Config { action } => {
            execute_config_action(action).with_context(|| "Failed to execute config command")
        }
        Commands::Template { action } => {
            execute_template_action(action).with_context(|| "Failed to execute template command")
        }
        #[cfg(feature = "dev-tools")]
        Commands::Dev { action } => {
            execute_dev_action(action).with_context(|| "Failed to execute dev command")
        }
    }
}

fn execute_setup_action(action: &SetupAction) -> Result<()> {
    match action {
        SetupAction::Status => setup::show_setup_status(),
        SetupAction::Clean => setup::clean_setup(),
    }
}

fn execute_assignment_action(action: &AssignmentAction) -> Result<()> {
    match action {
        AssignmentAction::Recent { course_id, limit } => {
            assignments::list_recent_assignments(course_id, *limit)
        }
        AssignmentAction::Stats { course_id } => assignments::show_assignment_stats(course_id),
        AssignmentAction::List => assignments::list_all_assignments(),
        AssignmentAction::Health { course_id } => {
            assignments::show_assignment_health(course_id.as_deref())
        }
    }
}

fn execute_template_action(action: &TemplateAction) -> Result<()> {
    match action {
        TemplateAction::Status => templates::template_status(),
        TemplateAction::Update => templates::update_template(),
        TemplateAction::Reinstall => templates::reinstall_template(),
        TemplateAction::Create {
            course_id,
            title,
            template_type,
            sections,
        } => {
            templates::create_custom_template(course_id, title, template_type, sections.as_deref())
        }
    }
}

fn execute_config_action(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => config::show_config(),
        ConfigAction::Get { key } => config::get_config_value(key),
        ConfigAction::Set { key, value } => config::set_config_value(key, value),
        ConfigAction::Edit => config::edit_config(),
        ConfigAction::ListKeys => config::list_config_keys(),
        ConfigAction::Interactive => config::interactive_config(),
        ConfigAction::SetAuthor { name } => config::set_author(name),
        ConfigAction::SetEditor { editor } => config::set_editor(editor),
        ConfigAction::AddTemplateRepo {
            name,
            repository,
            version,
            template_path,
        } => config::add_template_repository(
            name,
            repository,
            version.as_deref(),
            template_path.as_deref(),
        ),
        ConfigAction::RemoveTemplateRepo { name } => config::remove_template_repository(name),
        ConfigAction::EnableTemplateRepo { name, enabled } => {
            config::enable_template_repository(name, *enabled)
        }
        ConfigAction::ListTemplateRepos => config::list_template_repositories(),
        ConfigAction::SetTemplateAutoUpdate { enabled } => {
            config::set_template_auto_update(*enabled)
        }
        ConfigAction::Reset => config::reset_config(),
        ConfigAction::Path => config::show_config_path(),
        ConfigAction::Check => config::check_config(),
        ConfigAction::Cleanse { yes } => config::cleanse_config(*yes),
        ConfigAction::Migrate => config::migrate_config(),
    }
}

fn execute_course_action(action: &CourseAction) -> Result<()> {
    match action {
        CourseAction::List => courses::list_courses(),
        CourseAction::Add {
            course_id,
            course_name,
        } => courses::add_course(course_id, course_name),
        CourseAction::Remove { course_id } => courses::remove_course(course_id),
        CourseAction::Browse => courses::browse_common_courses(),
    }
}

#[cfg(feature = "dev-tools")]
fn execute_dev_action(action: &DevAction) -> Result<()> {
    use crate::dev::tools;

    match action {
        DevAction::Simulate => tools::simulate_high_yield_setup(),
        DevAction::Generate {
            courses,
            notes,
            assignments,
        } => tools::generate_sample_data(*courses, *notes, *assignments),
        DevAction::Clean => tools::clean_dev_data(),
    }
}
