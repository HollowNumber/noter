//! Directory scanning utilities
//!
//! Provides reusable directory scanning functionality used across
//! multiple commands.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result::Ok;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct CourseStats {
    pub last_activity: Option<FileInfo>,
    pub notes_count: usize,
    pub assignments_count: usize,
    pub total_files: usize,
}

pub struct DirectoryScanner;

#[allow(dead_code)]
impl DirectoryScanner {
    pub fn scan_course_directory<P: AsRef<Path>>(course_path: P) -> Result<CourseStats> {
        let course_path = course_path.as_ref();
        let mut stats = CourseStats {
            notes_count: 0,
            assignments_count: 0,
            last_activity: None,
            total_files: 0,
        };

        // Scan lectures directory
        let lectures_path = course_path.join("lectures");
        if !lectures_path.exists() {
            return Ok(stats);
        }

        let lecture_files = Self::scan_directory_for_files(&lectures_path, &["typ"])?;
        stats.notes_count = lecture_files.len();
        stats.total_files += lecture_files.len();

        if let Some(most_recent) = Self::find_most_recent(&lecture_files) {
            stats.last_activity = Some(most_recent);
        }

        // Scan assignments directory
        let assignments_path = course_path.join("assignments");

        if !assignments_path.exists() {
            return Ok(stats);
        }

        let assignment_files = Self::scan_directory_for_files(&assignments_path, &["typ"])?;
        stats.assignments_count = assignment_files.len();
        stats.total_files += assignment_files.len();

        let Some(most_recent) = Self::find_most_recent(&assignment_files) else {
            return Ok(stats);
        };

        match &stats.last_activity {
            None => stats.last_activity = Some(most_recent),
            Some(current) => {
                if most_recent.modified > current.modified {
                    stats.last_activity = Some(most_recent);
                }
            }
        }

        Ok(stats)
    }

    pub fn scan_directory_for_files<P: AsRef<Path>>(
        dir_path: P,
        extensions: &[&str],
    ) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        Self::scan_directory_recursive(dir_path.as_ref(), extensions, &mut files)?;
        Ok(files)
    }

    fn scan_directory_recursive(
        dir_path: &Path,
        extensions: &[&str],
        files: &mut Vec<FileInfo>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            // Handle directories recursively
            if path.is_dir() {
                Self::scan_directory_recursive(&path, extensions, files)?;
                continue;
            }

            // Skip non-files
            if !path.is_file() {
                continue;
            }

            // Get extension and skip if not in allowed list
            let Some(ext) = path.extension() else {
                continue;
            };

            let ext_str = ext.to_string_lossy().to_lowercase();
            if !extensions.contains(&ext_str.as_str()) {
                continue;
            }

            // Get metadata and push file info
            let Ok(metadata) = entry.metadata() else {
                continue;
            };

            let Ok(modified) = metadata.modified() else {
                continue;
            };

            files.push(FileInfo {
                path,
                modified,
                size: metadata.len(),
            });
        }

        Ok(())
    }

    #[must_use]
    pub fn find_most_recent(files: &[FileInfo]) -> Option<FileInfo> {
        files.iter().max_by_key(|file| file.modified).cloned()
    }

    pub fn scan_notes_directory<P: AsRef<Path>>(
        notes_dir: P,
    ) -> Result<Vec<(String, CourseStats)>> {
        let mut course_stats = Vec::new();

        for entry in fs::read_dir(notes_dir)? {
            let entry = entry?;

            // Skip non-directories
            if !entry.path().is_dir() {
                continue;
            }

            let file_name = entry.file_name();
            let Some(course_id) = file_name.to_str() else {
                continue;
            };

            // Check if it looks like a course code (5 digits)
            if course_id.len() != 5 || !course_id.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Scan the course directory and add to results
            let stats = Self::scan_course_directory(entry.path())?;
            course_stats.push((course_id.to_string(), stats));
        }

        Ok(course_stats)
    }
}
