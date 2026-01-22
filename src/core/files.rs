//! File operations and utilities
//!
//! Centralized file operations including opening files, creating directories,
//! managing backups, and handling file system operations.

use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use humansize::format_size;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileOperations;

#[allow(dead_code)]
impl FileOperations {
    /// Open a file with the configured editor or system default
    pub fn open_file(filepath: &Path, config: &Config) -> Result<()> {
        // Get preferred editor
        let editors = config.get_editor_list();

        for editor in editors {
            println!("  Trying {}...", editor.dimmed());

            if Self::try_command(&editor, filepath).is_ok() {
                return Ok(());
            }
        }

        // Fall back
        if opener::open(filepath).is_ok() {
            println!("{} Opened file with system default", "✅".green());
            return Ok(());
        }

        println!(
            "{} No suitable editor found. File created at: {}",
            "⚠️".yellow(),
            filepath.to_string_lossy()
        );

        Ok(())
    }

    fn try_command(editor: &str, path: &Path) -> Result<()> {
        std::process::Command::new(editor)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context(format!("Failed to spawn editor: {}", editor))?;

        println!("{} Opened with {}", "✅".green(), editor);
        Ok(())
    }
    //TOOD: Deduplicate code

    /// Opens a given filepath's parent directory
    fn open_file_directory(filepath: &Path, config: &Config) -> Result<()> {
        let editors = config.get_editor_list();
        let dir = filepath
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory"))?;

        for editor in editors {
            println!("  Trying {}...", editor.dimmed());

            if Self::try_command(&editor, dir).is_ok() {
                return Ok(());
            }
        }

        // if opener::open(dir).is_ok() {
        //     println!("Opened file with system default");
        //     return Ok(());
        // }

        println!(
            "{} No suitable editor found. File created at: {}",
            "⚠️".yellow(),
            filepath.to_string_lossy()
        );

        Ok(())
    }

    pub fn generate_filename(course_id: &str, type_: &str, title: Option<&str>) -> String {
        let date = chrono::Local::now().format("%Y-%m-%d");
        match title {
            Some(t) => format!(
                "{}-{}-{}.typ",
                date,
                course_id,
                t.to_lowercase().replace(' ', "-")
            ),
            None => format!("{}-{}-{}.typ", date, course_id, type_),
        }
    }

    /// Open a file via Obsidian URI
    pub fn open_obsidian_file(vault_path: &Path, relative_file_path: &str) -> Result<()> {
        let vault_name = vault_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("vault");

        let obsidian_uri = format!(
            "obsidian://open?vault={}&file={}",
            vault_name, relative_file_path
        );
        opener::open(obsidian_uri)?;
        println!("{} Opened in Obsidian", "✅".green());
        Ok(())
    }

    pub fn create_file_with_content_and_open(
        filepath: &Path,
        content: &str,
        config: &Config,
        auto_open: bool,
    ) -> Result<bool> {
        Self::create_file_with_content(filepath, content, config)?;

        if auto_open && config.note_preferences.auto_open_file {
            Self::open_file(filepath, config)?;
        } else if auto_open && config.note_preferences.auto_open_dir {
            Self::open_file_directory(filepath, config)?;
        }

        Ok(!filepath.exists())
    }

    /// Create a file with content, handling backups and overwrites
    pub fn create_file_with_content(filepath: &Path, content: &str, config: &Config) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = filepath.parent() {
            fs::create_dir_all(parent)?;
        }

        // Handle existing file
        if filepath.exists() {
            if config.note_preferences.create_backups {
                Self::create_backup(filepath)?;
            } else {
                anyhow::bail!("File already exists: {}", filepath.to_string_lossy());
            }
        }

        // Write the file
        fs::write(filepath, content)?;
        Ok(())
    }

    /// Create a backup of an existing file
    pub fn create_backup(file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Ok(());
        }

        let backup_path = Self::generate_backup_path(file_path)?;
        fs::copy(file_path, &backup_path)?;
        println!("{} Created backup: {}", "💾".blue(), backup_path.display());
        Ok(())
    }

    /// Generate a backup file path with timestamp
    fn generate_backup_path(original_path: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let mut backup_path = original_path.to_path_buf();

        if let Some(extension) = original_path.extension() {
            let new_extension = format!("{}.bak.{}", extension.to_string_lossy(), timestamp);
            backup_path.set_extension(&new_extension);
        } else {
            let new_name = format!(
                "{}.bak.{}",
                original_path.file_name().unwrap().to_string_lossy(),
                timestamp
            );
            backup_path.set_file_name(&new_name);
        }

        Ok(backup_path)
    }

    /// Ensure directory exists, create if it doesn't
    pub fn ensure_directory_exists(dir_path: &Path) -> Result<()> {
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
            println!(
                "{} Created directory: {}",
                "📁".blue(),
                dir_path.to_string_lossy().dimmed()
            );
        }
        Ok(())
    }

    /// Ensure course directory structure exists (lectures and assignments)
    pub fn ensure_course_structure(
        base_path: &Path,
        course_id: &str,
    ) -> Result<(PathBuf, PathBuf)> {
        let course_dir = base_path.join(course_id);
        let lectures_dir = course_dir.join("lectures");
        let assignments_dir = course_dir.join("assignments");

        Self::ensure_directory_exists(&lectures_dir)?;
        Self::ensure_directory_exists(&assignments_dir)?;

        Ok((lectures_dir, assignments_dir))
    }

    /// Get file modification time
    pub fn get_modification_time(filepath: &str) -> Result<std::time::SystemTime> {
        let metadata = fs::metadata(filepath)?;
        Ok(metadata.modified()?)
    }

    /// Check if file exists and is readable
    pub fn is_file_accessible(filepath: &str) -> bool {
        Path::new(filepath).is_file()
    }

    /// Get file size in bytes
    pub fn get_file_size(filepath: &str) -> Result<u64> {
        let metadata = fs::metadata(filepath)?;
        Ok(metadata.len())
    }

    /// Get file size formatted as human readable string
    pub fn get_file_size_formatted(filepath: &str) -> Result<String> {
        let size = Self::get_file_size(filepath)?;
        Ok(Self::format_file_size(size))
    }

    /// Format file size as human readable string
    pub fn format_file_size(size: u64) -> String {
        format_size(size, humansize::DECIMAL)
    }

    /// Remove file if it exists
    pub fn remove_file_if_exists(filepath: &str) -> Result<bool> {
        let path = Path::new(filepath);
        if path.exists() {
            fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Copy file with better error handling
    pub fn copy_file_safe(source: &str, destination: &str) -> Result<()> {
        let src_path = Path::new(source);
        let dst_path = Path::new(destination);

        if !src_path.exists() {
            anyhow::bail!("Source file does not exist: {}", source);
        }

        // Create destination directory if needed
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Handle existing destination file
        if dst_path.exists() {
            // Try to remove the destination file first if it exists
            match fs::remove_file(dst_path) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    anyhow::bail!("Permission denied: Cannot overwrite {}", destination);
                }
                Err(e) => return Err(e.into()),
            }
        }

        fs::copy(src_path, dst_path)?;
        Ok(())
    }

    /// Move file with better error handling
    pub fn move_file_safe(source: &str, destination: &str) -> Result<()> {
        Self::copy_file_safe(source, destination)?;
        Self::remove_file_if_exists(source)?;
        Ok(())
    }

    /// List files in directory with specific extensions
    pub fn list_files_with_extensions(dir_path: &str, extensions: &[&str]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let path = Path::new(dir_path);

        if !path.exists() {
            return Ok(files);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.contains(&ext_str.as_str()) {
                        files.push(entry_path);
                    }
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Count files in directory with specific extensions
    pub fn count_files_with_extensions(dir_path: &str, extensions: &[&str]) -> Result<usize> {
        let files = Self::list_files_with_extensions(dir_path, extensions)?;
        Ok(files.len())
    }

    /// Generate unique filename if file already exists
    pub fn generate_unique_filename(base_path: &str, filename: &str) -> Result<String> {
        let path = Path::new(base_path).join(filename);

        if !path.exists() {
            return Ok(filename.to_string());
        }

        // Extract name and extension
        let stem = path.file_stem().unwrap().to_string_lossy();
        let extension = path
            .extension()
            .map(|ext| format!(".{}", ext.to_string_lossy()))
            .unwrap_or_default();

        // Try numbered variants
        for i in 1..=999 {
            let new_filename = format!("{}-{}{}", stem, i, extension);
            let new_path = Path::new(base_path).join(&new_filename);

            if !new_path.exists() {
                return Ok(new_filename);
            }
        }

        anyhow::bail!("Could not generate unique filename for: {}", filename);
    }

    /// Clean up temporary files in directory
    pub fn clean_temp_files(dir_path: &str) -> Result<usize> {
        let temp_extensions = &["tmp", "temp", "bak", "swp", "swo"];
        let temp_files = Self::list_files_with_extensions(dir_path, temp_extensions)?;
        let count = temp_files.len();

        for file in temp_files {
            fs::remove_file(&file)?;
        }

        Ok(count)
    }

    /// Recursively copy a directory and all its contents
    pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        if !src.is_dir() {
            return Err(anyhow::anyhow!(
                "Source is not a directory: {}",
                src.display()
            ));
        }

        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                // Handle file copying with better error handling for Windows
                match fs::copy(&src_path, &dst_path) {
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            // Try to remove the destination file first if it exists
                            if dst_path.exists() {
                                match fs::remove_file(&dst_path) {
                                    Ok(_) => {
                                        // Now try copying again
                                        fs::copy(&src_path, &dst_path)?;
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "Warning: Could not overwrite {}. File may be in use.",
                                            dst_path.display()
                                        );
                                        continue;
                                    }
                                }
                            } else {
                                return Err(e.into());
                            }
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[ignore = "Currently incompatible with Humansize"]
    fn test_format_file_size() {
        assert_eq!(FileOperations::format_file_size(512), "512 B");
        assert_eq!(FileOperations::format_file_size(1024), "1.00 KB");
        assert_eq!(FileOperations::format_file_size(1048576), "1.00 MB");
        assert_eq!(FileOperations::format_file_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_generate_unique_filename() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create a test file
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        let unique_name = FileOperations::generate_unique_filename(temp_path, "test.txt").unwrap();
        assert_eq!(unique_name, "test-1.txt");
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("new_dir");

        assert!(!test_path.exists());
        FileOperations::ensure_directory_exists(&test_path).unwrap();
        assert!(Path::new(&test_path).exists());
    }
}
