# API Documentation

This document provides detailed information about the internal APIs and modules of DTU Notes.

## Core Modules

### Template Engine (`core::template_engine`)

The template engine handles dynamic template generation with automatic version detection.

#### TemplateEngine

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    /// Ensure templates are available, download if necessary
    pub fn ensure_templates_available(config: &Config) -> Result<()>

    /// Generate a lecture note template
    pub fn generate_lecture_template(
        course_id: &str,
        config: &Config,
        custom_title: Option<&str>,
    ) -> Result<String>

    /// Generate an assignment template
    pub fn generate_assignment_template(
        course_id: &str,
        assignment_title: &str,
        config: &Config,
    ) -> Result<String>

    /// Generate filename for a template
    pub fn generate_filename(
        course_id: &str,
        template_type: &TemplateType,
        custom_title: Option<&str>,
    ) -> Result<String>
}
```

#### TemplateContext

```rust
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub course_id: String,
    pub course_name: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub semester: String,
    pub template_version: String,
    pub sections: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}
```

#### TemplateBuilder

```rust
pub struct TemplateBuilder {
    // Private fields
}

impl TemplateBuilder {
    pub fn new(course_id: &str, config: &Config) -> Result<Self>
    pub fn with_title(self, title: &str) -> Self
    pub fn with_type(self, template_type: TemplateType) -> Self
    pub fn with_sections(self, sections: Vec<String>) -> Self
    pub fn build(&self) -> Result<String>
    pub fn build_with_filename(&self) -> Result<(String, String)>
}
```

### Status Manager (`core::status_manager`)

Provides comprehensive system status and health monitoring.

#### StatusManager

```rust
pub struct StatusManager;

impl StatusManager {
    /// Get current semester based on configuration
    pub fn get_current_semester(config: &Config) -> String

    /// Get activity summary for all courses
    pub fn get_activity_summary(config: &Config) -> Result<ActivitySummary>

    /// Get course health information
    pub fn get_course_health(config: &Config) -> Result<Vec<CourseHealthInfo>>

    /// Resolve course name from course ID
    pub fn resolve_course_name(course_id: &str, config: &Config) -> String
}
```

#### Health Status Types

```rust
#[derive(Debug, Clone)]
pub struct ActivitySummary {
    pub total_files: usize,
    pub files_last_week: usize,
    pub active_courses: usize,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Clone)]
pub struct CourseHealthInfo {
    pub course_id: String,
    pub course_name: String,
    pub file_count: usize,
    pub days_since_last_activity: u64,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Excellent,  // Recent activity, good file count
    Good,       // Some recent activity
    Warning,    // No recent activity but has files
    Critical,   // No files or very old activity
}
```

### GitHub Template Fetcher (`core::github_template_fetcher`)

Handles downloading and managing templates from GitHub repositories.

#### GitHubTemplateFetcher

```rust
pub struct GitHubTemplateFetcher;

impl GitHubTemplateFetcher {
    /// Check status of installed templates
    pub fn check_template_status(config: &Config) -> Result<Vec<(String, Option<String>)>>

    /// Download and install templates from configured repositories
    pub fn download_and_install_templates(config: &Config, force: bool) -> Result<Vec<InstallResult>>

    /// Update existing templates to latest versions
    pub fn update_templates(config: &Config) -> Result<Vec<UpdateResult>>

    /// Get latest release information from GitHub
    pub fn get_latest_release(owner: &str, repo: &str) -> Result<GitHubRelease>
}
```

### Typst Compiler (`core::typst_compiler`)

Handles compilation of Typst files to PDF with status tracking.

#### TypstCompiler

```rust
pub struct TypstCompiler;

impl TypstCompiler {
    /// Compile a Typst file to PDF
    pub fn compile_file(filepath: &str) -> Result<CompilationResult>

    /// Watch a file for changes and auto-compile
    pub fn watch_file(filepath: &str) -> Result<()>

    /// Check compilation status of a file
    pub fn check_status(filepath: &str) -> Result<CompilationStatus>

    /// Clean generated PDF files
    pub fn clean_pdfs(directory: &str) -> Result<usize>
}
```

#### Compilation Types

```rust
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub file_size: Option<u64>,
    pub compilation_time: std::time::Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CompilationStatus {
    UpToDate,
    OutOfDate,
    NotCompiled,
    SourceNotFound,
}
```

### Assignment Manager (`core::assignment_manager`)

Manages assignment creation and tracking with health analysis.

#### AssignmentManager

```rust
pub struct AssignmentManager<'a> {
    // Private fields
}

impl<'a> AssignmentManager<'a> {
    pub fn new(config: &'a Config) -> Self

    /// Create a new assignment file
    pub fn create_assignment(
        &self,
        course_id: &str,
        title: &str,
    ) -> Result<String>

    /// List all assignments with metadata
    pub fn list_all_assignments(&self) -> Result<Vec<AssignmentInfo>>

    /// Get assignment health analysis
    pub fn get_assignment_health(&self) -> Result<Vec<AssignmentHealth>>
}
```

### File Operations (`core::file_operations`)

Provides safe file operations with backup and validation.

#### FileOperations

```rust
pub struct FileOperations;

impl FileOperations {
    /// Create a file with content safely
    pub fn create_file_with_content(filepath: &str, content: &str) -> Result<()>

    /// Create backup of a file
    pub fn create_backup(file_path: &Path) -> Result<()>

    /// Ensure directory exists
    pub fn ensure_directory_exists(dir_path: &str) -> Result<()>

    /// Get file modification time
    pub fn get_modification_time(filepath: &str) -> Result<std::time::SystemTime>

    /// Copy file safely with error handling
    pub fn copy_file_safe(source: &str, destination: &str) -> Result<()>

    /// Move file safely with rollback on error
    pub fn move_file_safe(source: &str, destination: &str) -> Result<()>
}
```

## Configuration System

### Config Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub author: String,
    pub template_version: String,
    pub semester_format: SemesterFormat,
    pub paths: PathConfig,
    pub courses: HashMap<String, String>,
    pub template_repositories: HashMap<String, String>,
    pub note_preferences: NotePreferences,
    pub obsidian_integration: ObsidianIntegration,
}
```

### Configuration Functions

```rust
/// Load configuration from file
pub fn get_config() -> Result<Config>

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()>

/// Get default configuration
pub fn get_default_config() -> Result<Config>

/// Initialize configuration with setup wizard
pub fn initialize_config() -> Result<Config>
```

## UI Components

### Output Manager (`ui::output`)

Handles formatted console output with status indicators.

#### OutputManager

```rust
pub struct OutputManager;

impl OutputManager {
    /// Print a status message with icon
    pub fn print_status(status: Status, message: &str)

    /// Print a section header
    pub fn print_section(title: &str, icon: Option<&str>)

    /// Print a formatted table
    pub fn print_table(columns: &[TableColumn], rows: &[Vec<String>])

    /// Print a progress indicator
    pub fn print_progress(current: usize, total: usize, description: Option<&str>)
}
```

### Status Types

```rust
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Warning,
    Error,
    Info,
    Loading,
    Complete,
}
```

### Prompt Manager (`ui::prompts`)

Handles interactive user prompts and input validation.

#### PromptManager

```rust
pub struct PromptManager;

impl PromptManager {
    /// Prompt for user confirmation
    pub fn confirm(message: &str, default: bool) -> Result<bool>

    /// Prompt for text input
    pub fn input(message: &str, default: Option<&str>) -> Result<String>

    /// Prompt for selection from options
    pub fn select(message: &str, options: &[String]) -> Result<usize>

    /// Show spinner during long operations
    pub fn with_spinner<F, T>(message: &str, f: F) -> Result<T>
    where F: FnOnce() -> Result<T>
}
```

## Error Handling

All functions return `Result<T, anyhow::Error>` for comprehensive error handling.

### Common Error Patterns

```rust
use anyhow::{Result, Context, bail};

// Context for better error messages
let content = std::fs::read_to_string(path)
    .with_context(|| format!("Failed to read file: {}", path))?;

// Early returns with custom errors
if !path.exists() {
    bail!("Template file not found: {}", path.display());
}

// Chaining operations
let result = operation1()?
    .operation2()
    .with_context(|| "Operation chain failed")?;
```

## Testing

### Unit Tests

Each module includes comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_generation() {
        let config = Config::default();
        let result = TemplateEngine::generate_lecture_template("02101", &config, None);
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Integration tests are located in `tests/` directory:

```rust
// tests/template_integration.rs
use noter::*;

#[test]
fn test_full_template_workflow() {
    // Test complete template creation and compilation
}
```

## Performance Considerations

- File operations use buffered I/O
- Template generation is cached when possible
- Large directory scans are optimized with iterators
- Memory usage is minimized with streaming operations

## Security

- All file operations are validated
- Path traversal attacks are prevented
- User input is sanitized
- Temporary files are securely handled

---

For more specific implementation details, refer to the source code documentation generated with `cargo doc --open`.
