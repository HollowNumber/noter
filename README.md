# Noter CLI

[![Rust](https://img.shields.io/badge/rust-1.85.0%2B-brightgreen.svg)](https://www.rust-lang.org) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Version](https://img.shields.io/badge/version-0.5.2-blue.svg)](https://github.com/HollowNumber/noter/releases/latest)

A comprehensive command-line tool for managing lecture notes and assignments at the Technical University of Denmark (DTU). Built with Rust for performance and reliability, designed to work seamlessly with Typst, Obsidian, and DTU-branded templates.

## Features

### Academic Focus

- **DTU Integration**: Pre-configured for DTU courses and academic structure
- **Course Management**: Automatic course detection and organization
- **Assignment Tracking**: Health monitoring with visual status indicators (🟢🟡🟠🔴)
- **Semester Support**: Automatic semester detection and formatting

### Template System

- **Dynamic Templates**: Automatic template version detection and resolution
- **Official Branding**: DTU Design Guide 2018 compliant templates
- **Flexible Types**: Support for lectures, assignments, and custom templates
- **Template Repositories**: Custom template sources from GitHub repositories
- **Section Management**: Configurable sections based on document type

### Workflow Integration

- **Obsidian Support**: Generate vault structures and index files
- **File Watching**: Auto-compilation with `noter watch` command
- **Status Monitoring**: Comprehensive project health analysis
- **Search Functionality**: Fast full-text search across all documents
- **Compilation Status**: Detailed analysis of document compilation states

### Developer Experience

- **Setup Wizard**: Guided first-time configuration
- **Cross-Platform**: Windows, macOS, and Linux support
- **Rich CLI**: Comprehensive help text and command aliases
- **Error Handling**: Detailed error messages with actionable suggestions
- **Performance**: Optimized for large document collections

## Quick Start

### Prerequisites

- **[Rust](https://rustup.rs/)** (1.85.0+) - For building from source
- **[Typst](https://typst.app/)** (latest) - For PDF compilation
- **Git** - For template management
- **Text Editor** - VS Code, Neovim, or your preferred editor

### Installation

**Directly from GitHub**:

```bash
cargo install --git https://github.com/HollowNumber/noter.git
```

**From Source:**

```bash
git clone https://github.com/HollowNumber/noter.git
cd dtu-notes
```

Build and install:

```bash
cargo build --release
cargo build --release
# Add to PATH or use directly from target/release/noter
```

### Initial Setup

Run the interactive setup wizard to configure your workspace:

```bash
noter setup
```

The setup wizard will:

- Create directory structure
- Download DTU templates
- Configure author information
- Set up Obsidian vault (optional)
- Install Typst packages

**Manual Configuration (optional):**

```bash
# Configure personal information
noter config set-author "Your Full Name"
noter config add-course 02101 "Introduction to Programming"

# Set up paths
noter config set-path notes "path/to/your/notes"
noter config set-path obsidian "path/to/obsidian/vault"
```

For more advanced configuration you can traverse the config like a json file:

```bash
noter config show # shows all availables parameters
noter config set author "YourName"
noter set note_preferences.auto_open_file true
```

Alternatively you can edit the config file directly via your editor of choice.

Run
```bash
noter config path
```
to get the path.

**Verify Setup:**

```bash
noter status  # Check system status
```

## Usage Guide

### Core Document Creation

**Create Lecture Notes:**

```bash
noter note 02101                           # Default lecture note
noter template create 02101 "Custom Title" # Custom lecture note
```

**Create Assignments:**

```bash
noter assignment 02101 "Problem Set 1"     # Assignment template
noter a 02101 "Midterm Project"           # Using alias
```

**Advanced Template Creation:**

````bash
# Create custom template types
noter template create 02101 "Research Notes" --type custom

```bash
noter open 02101           # or: noter o 02101
````

View recent notes for a course:

```bash
noter recent 02101         # or: noter r 02101
```

### Course Management

List your courses:

```bash
noter courses list
```

Add a new course:

```bash
noter courses add 02102 "Algorithms and Data Structures"
```

Remove a course:

```bash
noter courses remove 02102
```

Browse common DTU courses:

```bash
noter courses browse
```

### Compilation & Development

Compile a note to PDF:

```bash
noter compile notes/02101/lectures/2025-01-15-02101-lecture.typ
```

Watch for changes and auto-compile:

```bash
noter watch notes/02101/lectures/2025-01-15-02101-lecture.typ
```

Clean up compiled PDFs:

```bash
noter clean
```

### Search & Discovery

Search through all notes:

```bash
noter search "algorithm"   # or: noter s "algorithm"
```

View comprehensive status:

```bash
noter status
```

Show current semester info:

```bash
noter semester
```

### Configuration

Show current configuration:

```bash
noter config show
```

Update author name:

```bash
noter config set-author "Your Name"
```

Set preferred editor:

```bash
noter config set-editor nvim
```

Check configuration health:

```bash
noter config check
```

Reset to defaults:

```bash
noter config reset
```

### Obsidian Integration

Create course index for Obsidian:

```bash
noter index 02101          # or: noter i 02101
```

## Project Structure

After running `noter setup`, your project will have this structure:

```
your-notes/
├── notes/                    # Course notes
│   ├── 02101/
│   │   ├── lectures/         # Lecture notes (.typ files)
│   │   └── assignments/      # Assignment files (.typ files)
│   └── [other courses]/
├── obsidian-vault/          # Obsidian vault (optional)
│   ├── courses/             # Course index files
│   ├── weekly-reviews/      # Weekly review notes
│   └── concept-maps/        # Concept mapping notes
├── templates/               # DTU templates
│   └── dtu-template/        # DTU unofficial templates
├── README.md
└── .gitignore
```


## Templates

DTU Notes uses unofficial DTU templates that follow the DTU Design Guide 2018. Templates include:

- **Lecture Notes**: Structured format with DTU branding
- **Assignments**: Problem set templates with proper formatting
- **Custom Sections**: Configurable sections for different note types

Templates are automatically installed to your Typst packages directory during setup.

## Contributing

We welcome contributions! Here's how you can help:

### Development Setup

Clone and build:

```bash
git clone <repository-url>
cd dtu-notes
cargo build
```

Run tests:

```bash
cargo test
```

Run with debug logging:

```bash
RUST_LOG=debug cargo run -- status
```

### Code Style

- Follow Rust conventions (`cargo fmt`)
- Run Clippy for linting (`cargo clippy`)
- Add tests for new functionality
- Update documentation for new features

### Areas for Contribution

-  **Bug Fixes**: Report and fix issues
-  **New Features**: Assignment due dates, better search, etc.
-  **Documentation**: Improve docs and examples
-  **Templates**: Additional DTU course templates
-  **Testing**: More comprehensive test coverage

### Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo clippy`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Status Dashboard

The `noter status` command provides a comprehensive overview:

- **System Status**: Directory and template validation
- **Recent Activity**: Latest notes and file statistics
- **Course Health**: Activity levels per course
- **Quick Suggestions**: Next recommended actions

## Search Features

Powerful search capabilities across all your notes:

- **Case-insensitive search** by default
- **Configurable file types** (.typ, .md by default)
- **Context lines** around matches
- **Highlighted results** for easy scanning

## Tips & Best Practices

- **Consistent naming**: Course codes should be 5 digits (e.g., 02101)
- **Regular commits**: Use git to track changes to your notes
- **Backup important work**: Keep PDFs of important assignments
- **Use search**: Leverage full-text search to find information quickly
- **Status checks**: Run `noter status` regularly to stay organized

## Troubleshooting

### Common Issues

**Templates not found**

```bash
noter status  # Check template status
noter setup   # Reinstall templates if needed
```

**Typst compilation fails**

```bash
typst --version  # Ensure Typst is installed
noter config check  # Validate configuration
```

**Files not opening**

```bash
noter config set-editor code  # Set your preferred editor
noter config show  # Check current settings
```

### Getting Help

- Check `noter --help` for command documentation
- Use `noter status` to diagnose setup issues
- Review configuration with `noter config show`
- Check the issue tracker for known problems

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Technical University of Denmark for the official branding guidelines
- The Typst team for the excellent typesetting system
- The Rust community for amazing tools and libraries
- Contributors who help improve this tool

---

**Happy note-taking!**
