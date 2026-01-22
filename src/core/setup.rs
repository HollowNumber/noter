// File: src/core/setup_manager.rs
//! Repository setup and initialization
//!
//! Handles directory creation, template installation, and project setup.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::core::files::FileOperations;
use crate::core::template::fetcher::Fetcher;

#[derive(Debug, Clone)]
pub struct SetupConfig {
    pub create_sample_courses: bool,
    pub install_templates: bool,
    pub create_readme: bool,
    pub create_gitignore: bool,
    pub force_overwrite: bool,
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            create_sample_courses: true,
            install_templates: true,
            create_readme: true,
            create_gitignore: true,
            force_overwrite: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupResult {
    pub directories_created: Vec<PathBuf>,
    pub files_created: Vec<PathBuf>,
    pub templates_installed: Vec<String>,
    pub sample_courses: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct SetupManager;

impl SetupManager {
    /// Perform complete repository setup
    pub fn setup_repository(config: &Config, setup_config: &SetupConfig) -> Result<SetupResult> {
        let mut result = SetupResult {
            directories_created: Vec::new(),
            files_created: Vec::new(),
            templates_installed: Vec::new(),
            sample_courses: Vec::new(),
            warnings: Vec::new(),
        };

        // Create base directory structure
        Self::create_directory_structure(config, &mut result)?;

        // Install templates if requested
        if setup_config.install_templates {
            Self::install_templates(config, &mut result)?;
        }

        // Create sample courses if requested
        if setup_config.create_sample_courses {
            Self::create_sample_courses(config, &mut result)?;
        }

        // Create README if requested
        if setup_config.create_readme {
            Self::create_readme(config, setup_config, &mut result)?;
        }

        // Create .gitignore if requested
        if setup_config.create_gitignore {
            Self::create_gitignore(config, setup_config, &mut result)?;
        }

        Ok(result)
    }

    /// Clean up setup (remove all created directories and files)
    pub fn clean_setup(config: &Config) -> Result<Vec<PathBuf>> {
        let mut removed_items = Vec::new();

        // Remove directories
        let dirs_to_remove = [
            &config.paths.notes_dir,
            &config.paths.obsidian_dir,
            &config.paths.templates_dir,
        ];

        for dir in dirs_to_remove {
            let path = Path::new(dir);
            if path.exists() {
                fs::remove_dir_all(path)?;
                removed_items.push(path.to_path_buf());
            }
        }

        // Remove files
        let files_to_remove = ["README.md", ".gitignore"];
        for file in files_to_remove {
            let path = Path::new(file);
            if path.exists() {
                fs::remove_file(path)?;
                removed_items.push(path.to_path_buf());
            }
        }

        Ok(removed_items)
    }

    /// Get the default sample courses
    pub fn get_sample_courses() -> &'static [(&'static str, &'static str)] {
        &[
            ("02101", "Introduction to Programming"),
            ("02102", "Algorithms and Data Structures"),
            ("01005", "Advanced Engineering Mathematics 1"),
            ("01006", "Advanced Engineering Mathematics 2"),
            ("25200", "Classical Physics 1"),
            ("22100", "Electronics 1"),
        ]
    }

    /// Check setup status
    pub fn check_setup_status(config: &Config) -> Result<SetupStatus> {
        // Check directories
        let mut status = SetupStatus {
            notes_dir_exists: Path::new(&config.paths.notes_dir).exists(),
            obsidian_dir_exists: Path::new(&config.paths.obsidian_dir).exists(),
            templates_dir_exists: Path::new(&config.paths.templates_dir).exists(),
            ..Default::default()
        };

        // Check template files
        let template_path = Path::new(&config.paths.templates_dir).join("dtu-template");
        status.templates_installed = template_path.exists();

        // Check for sample courses
        if status.notes_dir_exists {
            status.sample_courses_count = Self::count_course_directories(&config.paths.notes_dir)?;
        }

        // Check configuration
        status.author_configured = config.author != "Your Name";

        Ok(status)
    }

    // Private helper methods
    fn create_directory_structure(config: &Config, result: &mut SetupResult) -> Result<()> {
        let dirs = [
            &config.paths.notes_dir,
            &config.paths.templates_dir,
            &format!("{}/courses", config.paths.obsidian_dir),
            &format!("{}/weekly-reviews", config.paths.obsidian_dir),
            &format!("{}/concept-maps", config.paths.obsidian_dir),
        ];

        for dir in &dirs {
            let path = Path::new(dir);
            if !path.exists() {
                fs::create_dir_all(path)?;
                result.directories_created.push(path.to_path_buf());
            }
        }

        Ok(())
    }

    fn install_templates(config: &Config, result: &mut SetupResult) -> Result<()> {
        let local_template_dir = Path::new(&config.paths.templates_dir);
        let typst_local_dir = Path::new(&config.paths.typst_packages_dir);

        // Check if template directory exists in current working directory (local development)
        let repo_template_dir = Path::new("templates");

        if repo_template_dir.exists() {
            // Use local templates (development mode)
            result
                .warnings
                .push("Using local template directory for development".to_string());

            // Skip copying to local templates if source and destination are the same
            let repo_canonical = repo_template_dir.canonicalize()?;
            let local_canonical = local_template_dir
                .canonicalize()
                .unwrap_or_else(|_| local_template_dir.to_path_buf());

            if repo_canonical != local_canonical {
                // Copy templates to local directory first
                fs::create_dir_all(local_template_dir)?;
                FileOperations::copy_dir_recursive(repo_template_dir, local_template_dir)?;
            }

            // Install templates to Typst local packages
            fs::create_dir_all(typst_local_dir)?;
            Self::copy_template_contents(repo_template_dir, typst_local_dir)?;

            // List what was installed
            if let Ok(entries) = fs::read_dir(repo_template_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            result.templates_installed.push(name.to_string());
                        }
                    }
                }
            }
        } else {
            // No local templates found, download from GitHub
            result
                .warnings
                .push("No local templates found, downloading latest from GitHub...".to_string());

            let download_results = Fetcher::download_and_install_templates(config, false)?;

            for download_result in download_results {
                let template_name = if download_result
                    .installed_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("template")
                    == "official"
                {
                    "dtu-template (official)"
                } else {
                    download_result
                        .installed_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("template")
                };

                result
                    .templates_installed
                    .push(format!("{} ({})", template_name, download_result.version));

                if download_result.is_cached {
                    result
                        .warnings
                        .push(format!("Used cached {} template", template_name));
                } else {
                    result.warnings.push(format!(
                        "Downloaded {} template version {}",
                        template_name, download_result.version
                    ));
                }
            }
        }

        Ok(())
    }

    // Copy template contents (not the template directory itself)
    fn copy_template_contents(src: &Path, dst: &Path) -> Result<()> {
        if !src.is_dir() {
            return Err(anyhow::anyhow!(
                "Source is not a directory: {}",
                src.display()
            ));
        }

        fs::create_dir_all(dst)?;

        // Copy each item from templates/ to the destination
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                FileOperations::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    fn create_sample_courses(config: &Config, result: &mut SetupResult) -> Result<()> {
        let sample_courses = Self::get_sample_courses();

        for (course_id, _course_name) in sample_courses {
            let course_dir = Path::new(&config.paths.notes_dir).join(course_id);

            if !course_dir.exists() {
                fs::create_dir_all(course_dir.join("lectures"))?;
                fs::create_dir_all(course_dir.join("assignments"))?;
                result.sample_courses.push(course_id.to_string());
            }
        }

        Ok(())
    }

    fn create_readme(
        config: &Config,
        setup_config: &SetupConfig,
        result: &mut SetupResult,
    ) -> Result<()> {
        let readme_path = Path::new("README.md");

        if readme_path.exists() && !setup_config.force_overwrite {
            result
                .warnings
                .push("README.md already exists, skipped".to_string());
            return Ok(());
        }

        let content = Self::generate_readme_content(config)?;
        fs::write(readme_path, content)?;
        result.files_created.push(readme_path.to_path_buf());
        Ok(())
    }

    fn create_gitignore(
        config: &Config,
        setup_config: &SetupConfig,
        result: &mut SetupResult,
    ) -> Result<()> {
        let gitignore_path = Path::new(".gitignore");

        if gitignore_path.exists() && !setup_config.force_overwrite {
            result
                .warnings
                .push(".gitignore already exists, skipped".to_string());
            return Ok(());
        }

        let content = Self::generate_gitignore_content(config)?;
        fs::write(gitignore_path, content)?;
        result.files_created.push(gitignore_path.to_path_buf());
        Ok(())
    }

    fn generate_readme_content(config: &Config) -> Result<String> {
        Ok(format!(
            r#"# DTU Notes Repository

This repository contains lecture notes and assignments for DTU courses, organized using the DTU Notes CLI tool.

## Structure

```
{}/                    # Course notes
├── 02101/
│   ├── lectures/      # Lecture notes (.typ files)
│   └── assignments/   # Assignment files (.typ files)
└── [other courses]/

{}/           # Obsidian vault (optional)
├── courses/           # Course index files
├── weekly-reviews/    # Weekly review notes
└── concept-maps/      # Concept mapping notes

{}/              # Typst templates
└── dtu-template/      # DTU unofficial templates
```

## Getting Started

1. **Create a lecture note:**
   ```bash
   noter note 02101
   ```

2. **Create an assignment:**
   ```bash
   noter assignment 02101 "Problem Set 1"
   ```

3. **Compile to PDF:**
   ```bash
   noter compile path/to/file.typ
   ```

4. **Search through notes:**
   ```bash
   noter search "algorithms"
   ```

## Configuration

- Author: {}
- Template Version: {}

Update configuration:
```bash
noter config set-author "Your Name"
noter config set-editor code
noter config show
```

## Template System

This setup uses unofficial DTU templates following the DTU Design Guide 2018.
Templates are located in `{}/dtu-template/`.

## Tips

- Use `noter recent 02101` to see recent notes for a course
- Use `noter courses` to see all available DTU course codes
- Use `noter clean` to remove compiled PDF files
- Use `noter status` to check your setup

Happy note-taking! 📚
"#,
            config.paths.notes_dir,
            config.paths.obsidian_dir,
            config.paths.templates_dir,
            config.author,
            config.template_version,
            config.paths.templates_dir
        ))
    }

    fn generate_gitignore_content(_config: &Config) -> Result<String> {
        Ok(r#"# Compiled PDFs (uncomment to ignore PDFs)
# *.pdf

# Typst cache
.typst-cache/

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor files
.vscode/
.idea/
*.swp
*.swo
*~

# Temporary files
*.tmp
*.temp

# Backup files
*.bak
*.backup

# Log files
*.log

# Note: Configuration files are handled by the CLI tool
"#
        .to_string())
    }

    fn count_course_directories(notes_dir: &str) -> Result<usize> {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(notes_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
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
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SetupStatus {
    pub notes_dir_exists: bool,
    pub obsidian_dir_exists: bool,
    pub templates_dir_exists: bool,
    pub templates_installed: bool,
    pub sample_courses_count: usize,
    pub author_configured: bool,
    pub editor_configured: bool,
    pub typst_available: bool,
}

impl SetupStatus {
    pub fn is_complete(&self) -> bool {
        self.notes_dir_exists
            && self.templates_dir_exists
            && self.templates_installed
            && self.author_configured
            && self.typst_available
    }

    pub fn completion_percentage(&self) -> u8 {
        let checks = [
            self.notes_dir_exists,
            self.templates_dir_exists,
            self.templates_installed,
            self.author_configured,
            self.typst_available,
        ];

        let completed = checks.iter().filter(|&&x| x).count();
        ((completed as f32 / checks.len() as f32) * 100.0) as u8
    }
}
