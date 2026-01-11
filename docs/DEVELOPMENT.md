# Development Guide

This guide covers development practices, architecture decisions, and contribution guidelines for DTU Notes.

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (1.85.0+)
- **Git**: For version control and template management
- **Typst CLI**: For PDF compilation testing
- **IDE**: VS Code with rust-analyzer recommended

### Development Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/HollowNumber/dtu-notes.git
   cd dtu-notes
   ```

2. **Install dependencies**:

   ```bash
   cargo build
   ```

3. **Run tests**:

   ```bash
   cargo test
   ```

4. **Set up development environment**:

   ```bash
   # Create test workspace
   mkdir test-workspace
   cd test-workspace

   # Run development build
   ../target/debug/noter setup
   ```

## Architecture Overview

### Design Principles

1. **Separation of Concerns**: Clear separation between CLI, business logic, and I/O
2. **Error Handling**: Comprehensive error handling with context
3. **Modularity**: Independent modules with well-defined interfaces
4. **Testability**: All components are unit testable
5. **Extensibility**: Easy to add new commands and features

### Layer Architecture

```
┌─────────────────┐
│   CLI Layer     │  main.rs, commands/*
├─────────────────┤
│  Business Layer │  core/*
├─────────────────┤
│   UI Layer      │  ui/*
├─────────────────┤
│  Config Layer   │  config.rs, data.rs
└─────────────────┘
```

### Module Responsibilities

#### CLI Layer (`main.rs`, `commands/*`)

- **Purpose**: Handle command-line interface and user interactions
- **Responsibilities**:
  - Parse command-line arguments
  - Validate user input
  - Coordinate between UI and business logic
  - Handle command routing

#### Business Layer (`core/*`)

- **Purpose**: Implement core business logic
- **Responsibilities**:
  - Template generation and management
  - File operations and validation
  - External service integration (GitHub, Typst)
  - Status monitoring and health checks

#### UI Layer (`ui/*`)

- **Responsibilities**:
  - Format console output
  - Handle user prompts
  - Manage progress indicators
  - Style and color management

#### Configuration Layer (`config.rs`, `data.rs`)

- **Responsibilities**:
  - Configuration file management
  - Default values and validation
  - Static data (course information)

## Code Standards

### Naming Conventions

- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Error Handling

Always use `anyhow::Result` for error handling:

```rust
use anyhow::{Result, Context, bail};

pub fn create_file(path: &str, content: &str) -> Result<()> {
    // Provide context for errors
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path))?;

    // Use bail! for custom errors
    if content.is_empty() {
        bail!("Content cannot be empty");
    }

    Ok(())
}
```

### Documentation

All public functions must have documentation:

````rust
/// Create a new lecture template for the specified course.
///
/// # Arguments
///
/// * `course_id` - The course identifier (e.g., "02101")
/// * `config` - Application configuration
/// * `custom_title` - Optional custom title for the lecture
///
/// # Returns
///
/// Returns the generated template content as a string.
///
/// # Errors
///
/// This function will return an error if:
/// - The course is not found in configuration
/// - Template generation fails
/// - File system operations fail
///
/// # Examples
///
/// ```rust
/// use noter::core::template_engine::TemplateEngine;
///
/// let config = Config::default();
/// let template = TemplateEngine::generate_lecture_template("02101", &config, None)?;
/// ```
pub fn generate_lecture_template(
    course_id: &str,
    config: &Config,
    custom_title: Option<&str>
) -> Result<String> {
    // Implementation
}
````

### Testing Strategy

#### Unit Tests

- Test individual functions in isolation
- Use dependency injection for testability
- Mock external dependencies

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
        let template = result.unwrap();
        assert!(template.contains("02101"));
        assert!(template.contains("#import"));
    }

    #[test]
    fn test_invalid_course_id() {
        let config = Config::default();
        let result = TemplateEngine::generate_lecture_template("", &config, None);

        assert!(result.is_err());
    }
}
```

#### Integration Tests

- Test complete workflows
- Use temporary directories for file operations

```rust
// tests/integration_tests.rs
use noter::*;
use tempfile::tempdir;

#[test]
fn test_complete_workflow() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Test complete workflow from setup to compilation
    // ...
}
```

### Performance Guidelines

1. **Avoid Unnecessary Allocations**: Use `&str` instead of `String` when possible
2. **Use Iterators**: Prefer iterator chains over explicit loops
3. **Cache Results**: Cache expensive operations when appropriate
4. **Lazy Evaluation**: Use lazy evaluation for expensive computations

```rust
// Good: Iterator chain
let active_courses: Vec<_> = courses
    .iter()
    .filter(|(_, info)| info.is_active())
    .map(|(id, _)| id)
    .collect();

// Good: Lazy computation
fn get_course_info(&self, course_id: &str) -> Option<&CourseInfo> {
    self.courses.get(course_id)
        .or_else(|| self.load_course_from_disk(course_id))
}
```

## Adding New Features

### 1. Planning Phase

Before implementing a new feature:

1. **Design the API**: Define public interfaces
2. **Consider Dependencies**: Identify required modules
3. **Plan Testing**: How will you test the feature?
4. **Update Documentation**: What docs need updating?

### 2. Implementation Steps

1. **Add Command Definition**:

   ```rust
   // In main.rs
   #[derive(Subcommand)]
   pub enum Commands {
       // ... existing commands

       /// New feature description
       NewFeature {
           /// Feature parameter
           parameter: String,
       },
   }
   ```

2. **Implement Command Handler**:

   ```rust
   // In commands/new_feature.rs
   use anyhow::Result;
   use crate::config::get_config;

   pub fn handle_new_feature(parameter: &str) -> Result<()> {
       let config = get_config()?;
       // Implementation
       Ok(())
   }
   ```

3. **Add Core Logic**:

   ```rust
   // In core/new_feature_manager.rs
   pub struct NewFeatureManager;

   impl NewFeatureManager {
       pub fn process_feature(parameter: &str) -> Result<String> {
           // Core business logic
           Ok(result)
       }
   }
   ```

4. **Update Routing**:

   ```rust
   // In main.rs
   match cli.command {
       // ... existing commands
       Commands::NewFeature { parameter } => {
           commands::new_feature::handle_new_feature(&parameter)?;
       }
   }
   ```

5. **Add Tests**:
   ```rust
   // In commands/new_feature.rs
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_new_feature() {
           // Test implementation
       }
   }
   ```

### 3. Testing Checklist

- [ ] Unit tests for all new functions
- [ ] Integration tests for complete workflows
- [ ] Error case testing
- [ ] Edge case validation
- [ ] Performance testing (if applicable)

### 4. Documentation Updates

- [ ] Update README.md with new feature
- [ ] Add API documentation
- [ ] Update help text
- [ ] Add usage examples

## Debugging

### Logging

Use structured logging for debugging:

```rust
use log::{debug, info, warn, error};

pub fn process_template(path: &str) -> Result<()> {
    debug!("Processing template: {}", path);

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read template: {}", path))?;

    info!("Template loaded, size: {} bytes", content.len());

    // Process content
    if content.is_empty() {
        warn!("Template is empty: {}", path);
    }

    Ok(())
}
```

### Environment Variables

- `RUST_LOG=debug`: Enable debug logging
- `RUST_BACKTRACE=1`: Show backtraces on errors
- `RUST_BACKTRACE=full`: Show full backtraces

### Debugging Commands

```bash
# Debug specific command
RUST_LOG=debug cargo run -- note 02101

# Debug with backtrace
RUST_BACKTRACE=1 cargo run -- note 02101

# Profile performance
cargo run --release -- note 02101
```

## Release Process

### Version Management

1. **Update Version**: Update version in `Cargo.toml`
2. **Update Changelog**: Document changes in `CHANGELOG.md`
3. **Tag Release**: Create git tag with version
4. **Build Release**: `cargo build --release`

### Testing Before Release

```bash
# Run all tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Test installation
cargo install --path .

# Integration testing
noter setup --help
noter note 02101
```

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Version bumped
- [ ] Changelog updated
- [ ] No compiler warnings
- [ ] Integration tests pass
- [ ] Performance acceptable

## Troubleshooting

### Common Development Issues

#### "Cannot find module" errors

- Check module declarations in `mod.rs` files
- Verify file names match module declarations
- Ensure proper `pub use` statements

#### Template compilation issues

- Verify Typst CLI is installed
- Check template syntax
- Validate import statements

#### Test failures

- Ensure tests use temporary directories
- Clean up resources in tests
- Check for race conditions in parallel tests

### Debug Tools

```bash
# Examine generated templates
cargo run -- template create 02101 "Test" --debug

# Verbose compilation
cargo run -- compile file.typ --verbose

# Status information
cargo run -- status --debug
```

## Contributing

### Pull Request Process

1. **Fork** the repository
2. **Create branch** from main: `git checkout -b feature/new-feature`
3. **Implement** changes following this guide
4. **Test** thoroughly
5. **Document** changes
6. **Submit** pull request

### Code Review Guidelines

- **Functionality**: Does it work as intended?
- **Tests**: Are there sufficient tests?
- **Documentation**: Is it well documented?
- **Style**: Does it follow project conventions?
- **Performance**: Are there any performance concerns?

### Commit Message Format

```
type(scope): brief description

Longer description if needed.

- List specific changes
- Reference issues: Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Typst Documentation](https://typst.app/docs/)
- [Anyhow Error Handling](https://docs.rs/anyhow/)
- [Clap CLI Framework](https://docs.rs/clap/)
- [Serde Serialization](https://docs.rs/serde/)

---

_Happy coding! 🦀_
