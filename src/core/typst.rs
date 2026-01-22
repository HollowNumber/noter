//! Typst compilation and file management
//!
//! Handles compiling Typst files to PDF, watching for changes, and cleaning compiled files.

use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct TypstCompiler;

#[allow(dead_code)]
impl TypstCompiler {
    /// Compile a Typst file to PDF
    pub fn compile_file(filepath: &str, config: &Config) -> Result<String> {
        let input_path = Self::resolve_input_path(filepath)?;
        let output_path = Self::determine_output_path(&input_path, config)?;

        // Clean before compiling if configured
        if config.typst.clean_before_compile {
            if let Some(parent) = output_path.parent() {
                Self::clean_directory(parent)?;
            }
        }

        // Convert paths to strings once to avoid temporary value issues
        let input_str = input_path.to_string_lossy().into_owned();
        let output_str = output_path.to_string_lossy().into_owned();

        // Build command arguments - modern Typst syntax: typst compile input.typ output.pdf
        let mut args = vec!["compile", &input_str, &output_str];

        // Add custom compile arguments
        for arg in &config.typst.compile_args {
            args.push(arg);
        }

        // Execute compilation
        let output = Command::new("typst")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Typst compilation failed: {}", stderr);
        }

        Ok(output_str)
    }

    /// Watch a Typst file for changes and auto-compile
    pub fn watch_file(filepath: &str, config: &Config) -> Result<()> {
        let input_path = Self::resolve_input_path(filepath)?;
        let output_path = Self::determine_output_path(&input_path, config)?;

        // Convert paths to strings once to avoid temporary value issues
        let input_str = input_path.to_string_lossy().into_owned();
        let output_str = output_path.to_string_lossy().into_owned();

        // Build command arguments - modern Typst syntax: typst watch input.typ output.pdf
        let mut args = vec!["watch", &input_str, &output_str];

        // Add custom watch arguments
        for arg in &config.typst.watch_args {
            args.push(arg);
        }

        // Execute watch command (this blocks until interrupted)
        let mut child = Command::new("typst").args(&args).spawn()?;

        // Wait for the child process to complete
        let status = child.wait()?;

        if !status.success() {
            anyhow::bail!("Typst watch failed with exit code: {:?}", status.code());
        }

        Ok(())
    }

    /// Clean compiled PDF files in the notes directory
    pub fn clean_files(config: &Config) -> Result<usize> {
        let mut cleaned_count = 0;

        // Clean main notes directory
        if Path::new(&config.paths.notes_dir).exists() {
            cleaned_count += Self::clean_directory_recursive(&config.paths.notes_dir)?;
        }

        // Clean obsidian directory if it exists
        if Path::new(&config.paths.obsidian_dir).exists() {
            cleaned_count += Self::clean_directory_recursive(&config.paths.obsidian_dir)?;
        }

        Ok(cleaned_count)
    }

    /// Get compilation status for a file
    pub fn get_compilation_status(filepath: &str, config: &Config) -> Result<CompilationStatus> {
        let input_path = Self::resolve_input_path(filepath)?;
        let output_path = Self::determine_output_path(&input_path, config)?;

        if !input_path.exists() {
            return Ok(CompilationStatus::SourceNotFound);
        }

        if !output_path.exists() {
            return Ok(CompilationStatus::NotCompiled);
        }

        let source_modified = fs::metadata(&input_path)?.modified()?;
        let output_modified = fs::metadata(&output_path)?.modified()?;

        if source_modified > output_modified {
            Ok(CompilationStatus::OutOfDate)
        } else {
            Ok(CompilationStatus::UpToDate)
        }
    }

    /// Check if Typst is available on the system
    pub fn check_typst_availability() -> Result<String> {
        let output = Command::new("typst").arg("--version").output()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            anyhow::bail!(
                "Typst not found. Please install Typst: https://github.com/typst/typst#installation"
            );
        }
    }

    // Private helper methods

    /// Resolve input path (add .typ extension if missing)
    fn resolve_input_path(filepath: &str) -> Result<PathBuf> {
        let mut path = PathBuf::from(filepath);

        if path.extension().is_none() {
            path.set_extension("typ");
        }

        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }

        Ok(path)
    }

    /// Determine output path based on configuration
    fn determine_output_path(input_path: &Path, config: &Config) -> Result<PathBuf> {
        let mut output_path = input_path.with_extension("pdf");

        // Use custom output directory if configured
        if let Some(ref output_dir) = config.typst.output_dir {
            if let Some(filename) = output_path.file_name() {
                let custom_dir = if Path::new(output_dir).is_absolute() {
                    PathBuf::from(output_dir)
                } else {
                    input_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(output_dir)
                };

                fs::create_dir_all(&custom_dir)?;
                output_path = custom_dir.join(filename);
            }
        }

        Ok(output_path)
    }

    /// Clean PDF files in a single directory
    fn clean_directory(dir: &Path) -> Result<usize> {
        let mut cleaned = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "pdf") {
                fs::remove_file(&path)?;
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }

    /// Clean PDF files recursively
    fn clean_directory_recursive(dir: &str) -> Result<usize> {
        let mut cleaned = 0;
        let path = Path::new(dir);

        if !path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Convert to owned string to avoid borrowing issues
                let subdir_str = entry_path.to_string_lossy().into_owned();
                cleaned += Self::clean_directory_recursive(&subdir_str)?;
            } else if entry_path.extension().is_some_and(|ext| ext == "pdf") {
                fs::remove_file(&entry_path)?;
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompilationStatus {
    UpToDate,
    OutOfDate,
    NotCompiled,
    SourceNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_input_path_adds_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.typ");
        File::create(&file_path).unwrap();

        let test_path = temp_dir.path().join("test").to_string_lossy().into_owned();
        let resolved = TypstCompiler::resolve_input_path(&test_path).unwrap();

        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_resolve_input_path_keeps_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.typ");
        File::create(&file_path).unwrap();

        let file_path_str = file_path.to_string_lossy().into_owned();
        let resolved = TypstCompiler::resolve_input_path(&file_path_str).unwrap();

        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_determine_output_path() {
        let config = Config::default();
        let input_path = PathBuf::from("/path/to/file.typ");
        let output_path = TypstCompiler::determine_output_path(&input_path, &config).unwrap();

        assert_eq!(output_path, PathBuf::from("/path/to/file.pdf"));
    }

    #[test]
    fn test_clean_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create test PDF files
        File::create(temp_dir.path().join("test1.pdf")).unwrap();
        File::create(temp_dir.path().join("test2.pdf")).unwrap();
        File::create(temp_dir.path().join("keep.txt")).unwrap();

        let cleaned = TypstCompiler::clean_directory(temp_dir.path()).unwrap();

        assert_eq!(cleaned, 2);
        assert!(!temp_dir.path().join("test1.pdf").exists());
        assert!(!temp_dir.path().join("test2.pdf").exists());
        assert!(temp_dir.path().join("keep.txt").exists());
    }
}
