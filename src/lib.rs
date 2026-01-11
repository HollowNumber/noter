//! # DTU Notes Library
//!
//! This library provides the core functionality for the DTU Notes CLI application,
//! but can also be used as a standalone library for programmatic access to
//! note and template management functionality.
//!
//! ## Features
//!
//! - **Template Engine**: Dynamic template generation with version detection
//! - **Course Management**: Course information and organization
//! - **File Operations**: Safe file operations with backup and validation
//! - **Configuration Management**: JSON-based configuration system
//! - **Status Monitoring**: Health analysis and activity tracking
//! - **Typst Integration**: Compilation and file watching capabilities
//!
//! ## Usage Examples
//!
//! ### Basic Template Generation
//!
//! ```rust
//! use noter::core::template_engine::{TemplateEngine, TemplateType};
//! use noter::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::default();
//!
//! // Generate a lecture template
//! let lecture = TemplateEngine::generate_lecture_template("02101", &config, None)?;
//!
//! // Generate an assignment template
//! let assignment = TemplateEngine::generate_assignment_template(
//!     "02101",
//!     "Problem Set 1",
//!     &config
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Configuration Management
//!
//! ```rust
//! use noter::config::get_config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = get_config()?;
//!
//! // Access configuration settings
//! println!("Author: {}", config.author);
//! println!("Courses: {:?}", config.courses);
//! # Ok(())
//! # }
//! ```
//!
//! ### Template Builder Pattern
//!
//! ```rust
//! use noter::core::template_engine::{TemplateBuilder, TemplateType};
//! use noter::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::default();
//!
//! let (content, filename) = TemplateBuilder::new("02101", &config)?
//!     .with_title("Advanced Topics")
//!     .with_type(TemplateType::Custom("research".to_string()))
//!     .with_sections(vec!["Methodology".to_string(), "Results".to_string()])
//!     .build_with_filename()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`config`] - Configuration management and serialization
//! - [`core`] - Core business logic modules
//!   - [`core::template_engine`] - Template generation and management
//!   - [`core::status_manager`] - System status and health monitoring
//!   - [`core::typst_compiler`] - Typst compilation and file watching
//!   - [`core::file_operations`] - Safe file operations
//!   - [`core::github_template_fetcher`] - Template repository management
//! - [`ui`] - User interface components
//! - [`data`] - Static data and course information
//!
//! ## Error Handling
//!
//! All functions return `Result<T, anyhow::Error>` for comprehensive error handling
//! with contextual information. Errors can be easily converted to other error types
//! or displayed to users with helpful context.
//!
//! ## Thread Safety
//!
//! Most operations are designed to be thread-safe, though file operations should
//! be coordinated to avoid conflicts. The configuration system uses atomic writes
//! to prevent corruption during concurrent access.

pub mod commands;
pub mod config;
pub mod core;
pub mod data;
#[cfg(feature = "dev-tools")]
pub mod dev;
pub mod ui;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new lecture note
    #[command(alias = "n")]
    Note {
        /// Course code (e.g., 02101)
        course_id: String,

        /// Custom title for the note (optional)
        #[arg(short, long)]
        title: Option<String>,

        /// Template variant to use (e.g., math, programming)
        #[arg(short, long)]
        variant: Option<String>,

        /// Custom sections (comma-separated)
        #[arg(short, long)]
        sections: Option<String>,

        /// Skip auto opening for file
        #[arg(long)]
        no_open: bool,
    },
    /// Create a new assignment
    #[command(alias = "a")]
    Assignment {
        /// Course code (e.g., 02101)
        course_id: String,
        /// Assignment title
        title: String,
    },
    /// Compile a Typst file to PDF
    #[command(alias = "c")]
    Compile {
        /// Path to the .typ file (with or without extension)
        filepath: String,
        /// Check compilation status before compiling
        #[arg(long)]
        check_status: bool,
    },
    /// Watch and auto-compile a Typst file
    #[command(alias = "w")]
    Watch {
        /// Path to the .typ file (with or without extension)
        filepath: String,
    },
    /// Check compilation status of files
    Check {
        /// Path to specific file (optional - checks all if omitted)
        filepath: Option<String>,
        /// Show detailed status information
        #[arg(long)]
        detailed: bool,
    },
    /// List recent notes for a course
    #[command(alias = "r")]
    Recent {
        /// Course code
        course_id: String,
    },
    /// Initialize repository structure
    Setup {
        #[command(subcommand)]
        action: Option<SetupAction>,
    },
    /// Create Obsidian course index
    #[command(alias = "i")]
    Index {
        /// Course code
        course_id: String,
    },
    /// Search through notes
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,
    },
    /// Rebuild search index
    #[command(name = "rebuild-index", alias = "ri")]
    RebuildIndex {
        /// Force rebuild even if index is fresh
        #[arg(long, short)]
        force: bool,
    },
    /// Assignment management
    Assignments {
        #[command(subcommand)]
        action: AssignmentAction,
    },

    /// Course management
    Courses {
        #[command(subcommand)]
        action: CourseAction,
    },

    /// Open most recent note for a course
    #[command(alias = "o")]
    Open {
        /// Course code
        course_id: String,
    },

    /// Show comprehensive status dashboard
    Status,

    /// Clean up compiled PDFs
    Clean,
    /// Show current semester info
    Semester,
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Template management
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// Development tools (hidden in release builds)
    #[cfg(feature = "dev-tools")]
    #[command(hide = true)]
    Dev {
        #[command(subcommand)]
        action: DevAction,
    },
}

#[derive(Subcommand)]
pub enum SetupAction {
    /// Show setup status and completion
    Status,
    /// Clean/reset the entire setup
    Clean,
}

#[derive(Subcommand)]
pub enum AssignmentAction {
    /// List recent assignments for a course
    Recent {
        /// Course code
        course_id: String,
        /// Number of recent assignments to show
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    /// Show assignment statistics for a course
    Stats {
        /// Course code
        course_id: String,
    },
    /// List all assignments across courses with activity summary
    List,
    /// Show assignment health and activity analysis
    Health {
        /// Course code (optional - shows all courses if omitted)
        course_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CourseAction {
    /// List all courses
    List,
    /// Add a new course
    Add {
        /// Course code (e.g., 02101)
        course_id: String,
        /// Course name
        course_name: String,
    },
    /// Remove a course
    Remove {
        /// Course code to remove
        course_id: String,
    },
    /// Show common DTU course codes
    #[command(alias = "common")]
    Browse,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Get a specific configuration value using dot notation (e.g., "author" or "paths.notes_dir")
    Get {
        /// Configuration key path (e.g., "author", "paths.notes_dir", "templates.auto_update")
        key: String,
    },
    /// Set a configuration value using dot notation (e.g., "author" "John Doe")
    Set {
        /// Configuration key path (e.g., "author", "paths.notes_dir", "templates.auto_update")
        key: String,
        /// Value to set (will be parsed based on the field type)
        value: String,
    },
    /// Open configuration file in editor
    Edit,
    /// List all available configuration keys
    ListKeys,
    /// Interactive configuration wizard
    Interactive,
    /// Set author name
    SetAuthor {
        /// Author name
        name: String,
    },
    /// Set preferred editor
    SetEditor {
        /// Editor command (e.g., code, nvim)
        editor: String,
    },
    /// Add a custom template repository
    AddTemplateRepo {
        /// Repository name
        name: String,
        /// GitHub repository (owner/repo)
        repository: String,
        /// Specific version (optional)
        #[arg(long)]
        version: Option<String>,
        /// Template subdirectory path (optional)
        #[arg(long)]
        template_path: Option<String>,
    },
    /// Remove a template repository
    RemoveTemplateRepo {
        /// Repository name to remove
        name: String,
    },
    /// Enable/disable a template repository
    EnableTemplateRepo {
        /// Repository name
        name: String,
        /// Whether to enable (true) or disable (false)
        enabled: bool,
    },
    /// List all template repositories
    ListTemplateRepos,
    /// Enable/disable template auto-update
    SetTemplateAutoUpdate {
        /// Enable auto-update
        enabled: bool,
    },
    /// Reset configuration to defaults
    Reset,
    /// Purge configuration and start fresh
    #[command(alias = "purge")]
    Cleanse {
        /// Skip confirmation prompt
        #[arg(long, short)]
        yes: bool,
    },
    /// Show config file path
    Path,
    /// Validate current configuration
    Check,
    /// Migrate configuration to latest format (usually happens automatically)
    Migrate,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// Check template status and version
    Status,
    /// Update to the latest template version
    Update,
    /// Force reinstall templates
    Reinstall,
    /// Create a custom template file
    Create {
        /// Course code
        course_id: String,
        /// Template title
        title: String,
        /// Template type (lecture, assignment, or custom)
        #[arg(short, long, default_value = "lecture")]
        template_type: String,
        /// Custom sections (comma-separated)
        #[arg(short, long)]
        sections: Option<String>,
    },
}

#[cfg(feature = "dev-tools")]
#[derive(Subcommand)]
pub enum DevAction {
    /// Generate high-yield simulation data (many courses, notes, assignments)
    Simulate,
    /// Generate sample data with specific parameters
    Generate {
        /// Number of courses to generate
        #[arg(short, long, default_value = "5")]
        courses: usize,
        /// Number of notes per course
        #[arg(short, long, default_value = "10")]
        notes: usize,
        /// Number of assignments per course
        #[arg(short, long, default_value = "3")]
        assignments: usize,
    },
    /// Clean all generated development data
    Clean,
}

// Re-export commonly used types for easier access
pub use config::{Config, get_config};
pub use core::status::{HealthStatus, StatusManager};
pub use core::typst::{CompilationStatus, TypstCompiler};
#[cfg(feature = "dev-tools")]
pub use dev::generator::{CleanupStats, Course, DevDataGenerator, GenerationStats};

/// Current version of the DTU Notes library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library result type for consistent error handling
pub type Result<T> = std::result::Result<T, anyhow::Error>;
