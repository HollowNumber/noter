//! # DTU Notes CLI
//!
//! A comprehensive command-line tool for managing notes and assignments at DTU
//! (Technical University of Denmark) with integrated Typst compilation and
//! Obsidian vault support.
//!
//! ## Features
//! - Dynamic template generation with automatic version detection
//! - Comprehensive project status monitoring and health analysis
//! - Seamless Typst compilation with file watching capabilities
//! - Obsidian vault integration for enhanced note management
//! - Course management with assignment tracking
//! - Extensible template system supporting custom repositories
//!
//! ## Usage Examples
//! ```bash
//! # Setup workspace
//! noter setup
//!
//! # Create lecture note
//! noter note 02101
//!
//! # Create assignment
//! noter assignment 02101 "Problem Set 1"
//!
//! # Compile to PDF
//! noter compile file.typ
//!
//! # Monitor system status
//! noter status
//! ```
#[allow(clippy::multiple_crate_versions)]
mod commands;
mod config;
mod core;
mod data;
mod ui;

use anyhow::Result;
use clap::Parser;
use noter::{AssignmentAction, Commands, ConfigAction, CourseAction, SetupAction, TemplateAction};

#[cfg(feature = "dev-tools")]
use noter::DevAction;

/// Command-line interface structure using clap derive macros.
///
/// This structure defines the main CLI application with global configuration
/// and routing to subcommands.
#[derive(Parser)]
#[command(name = "noter")]
#[command(about = "DTU note-taking CLI with official branding")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Main application entry point.
///
/// Parses command-line arguments using clap and routes execution to the
/// appropriate command handler. All errors are propagated using the `?` operator
/// and handled by anyhow's automatic error formatting.
///
/// # Returns
///
/// Returns `Ok(())` on successful execution, or an error with context
/// if any command fails.
fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::execute_command(&cli.command)?;
    Ok(())
}
