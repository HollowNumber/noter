//! Status and information management
//!
//! Handles status checking, activity summaries, and course health monitoring.

use crate::config::Config;
use crate::core::directories::{CourseStats, DirectoryScanner};
use anyhow::Result;
use chrono::Datelike;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub directories: HashMap<String, bool>,
    pub templates: HashMap<String, bool>,
    pub configuration_warnings: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActivitySummary {
    pub total_notes: usize,
    pub total_assignments: usize,
    pub most_recent_activity: Option<RecentActivity>,
    pub most_active_course: Option<(String, usize)>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecentActivity {
    pub file_name: String,
    pub course_id: String,
    pub course_name: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CourseHealthInfo {
    pub course_id: String,
    pub course_name: String,
    pub notes_count: usize,
    pub assignments_count: usize,
    pub days_since_last_activity: u64,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum HealthStatus {
    Excellent, // Recent activity, good file count
    Good,      // Some recent activity
    Warning,   // No recent activity but has files
    Critical,  // No files or very old activity
}

pub struct StatusManager;

#[allow(dead_code)]
impl StatusManager {
    /// Get comprehensive system status
    pub fn get_system_status(config: &Config) -> Result<SystemStatus> {
        let mut directories = HashMap::new();
        let mut templates = HashMap::new();

        // Check directory status
        let paths_to_check = [
            ("Notes", &config.paths.notes_dir),
            ("Obsidian Vault", &config.paths.obsidian_dir),
            ("Templates", &config.paths.templates_dir),
            ("Typst Packages", &config.paths.typst_packages_dir),
        ];

        for (name, path) in paths_to_check {
            directories.insert(name.to_string(), Path::new(path).exists());
        }

        // Check template files
        let template_paths = [
            format!("{}/dtu-template/lib.typ", config.paths.templates_dir),
            format!(
                "{}/dtu-template/{}/lib.typ",
                config.paths.typst_packages_dir, config.template_version
            ),
            format!("{}/dtu-template/typst.toml", config.paths.templates_dir),
        ];

        for template_path in &template_paths {
            let path = Path::new(template_path);
            templates.insert(template_path.clone(), path.exists());
        }

        // Get configuration warnings
        let configuration_warnings = config.validate()?;

        Ok(SystemStatus {
            directories,
            templates,
            configuration_warnings,
        })
    }

    /// Get activity summary across all courses
    pub fn get_activity_summary(config: &Config) -> Result<ActivitySummary> {
        if !Path::new(&config.paths.notes_dir).exists() {
            return Ok(ActivitySummary {
                total_notes: 0,
                total_assignments: 0,
                most_recent_activity: None,
                most_active_course: None,
            });
        }

        let course_stats = DirectoryScanner::scan_notes_directory(&config.paths.notes_dir)?;

        let mut total_notes = 0;
        let mut total_assignments = 0;
        let mut most_recent_activity: Option<RecentActivity> = None;
        let mut course_activity: HashMap<String, usize> = HashMap::new();

        for (course_id, stats) in &course_stats {
            total_notes += stats.notes_count;
            total_assignments += stats.assignments_count;

            let total_activity = stats.notes_count + stats.assignments_count;
            course_activity.insert(course_id.clone(), total_activity);

            // Check for most recent activity
            if let Some(ref last_activity) = stats.last_activity {
                let course_name = config
                    .courses
                    .get(course_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown Course".to_string());

                let activity = RecentActivity {
                    file_name: last_activity
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    course_id: course_id.clone(),
                    course_name,
                    timestamp: last_activity.modified,
                };

                match &most_recent_activity {
                    None => most_recent_activity = Some(activity),
                    Some(current) => {
                        if activity.timestamp > current.timestamp {
                            most_recent_activity = Some(activity);
                        }
                    }
                }
            }
        }

        // Find most active course
        let most_active_course = course_activity.into_iter().max_by_key(|&(_, count)| count);

        Ok(ActivitySummary {
            total_notes,
            total_assignments,
            most_recent_activity,
            most_active_course,
        })
    }

    /// Get health information for all courses
    pub fn get_course_health(config: &Config) -> Result<Vec<CourseHealthInfo>> {
        if !Path::new(&config.paths.notes_dir).exists() {
            return Ok(Vec::new());
        }

        let mut course_health = Vec::new();

        for (course_id, course_name) in &config.courses {
            let course_path = Path::new(&config.paths.notes_dir).join(course_id);

            if course_path.exists() {
                let stats = DirectoryScanner::scan_course_directory(&course_path)?;
                let days_since_last = Self::calculate_days_since_last_activity(&stats);
                let health_status = Self::determine_health_status(&stats, days_since_last);

                course_health.push(CourseHealthInfo {
                    course_id: course_id.clone(),
                    course_name: course_name.clone(),
                    notes_count: stats.notes_count,
                    assignments_count: stats.assignments_count,
                    days_since_last_activity: days_since_last,
                    health_status,
                });
            }
        }

        // Sort by health status and then by activity
        course_health.sort_by(|a, b| a.days_since_last_activity.cmp(&b.days_since_last_activity));

        Ok(course_health)
    }

    /// Get current semester information
    pub fn get_semester_info(config: &Config) -> SemesterInfo {
        let now = chrono::Local::now();
        let year = now.year();
        let month = now.month();
        let is_spring = month <= 6;

        SemesterInfo {
            current_semester: config.format_semester(year, is_spring),
            year,
            is_spring,
            format: config.semester_format.clone(),
        }
    }

    // Private helper methods
    fn calculate_days_since_last_activity(stats: &CourseStats) -> u64 {
        if let Some(ref last_activity) = stats.last_activity {
            let duration = std::time::SystemTime::now()
                .duration_since(last_activity.modified)
                .unwrap_or_default();
            duration.as_secs() / (24 * 60 * 60)
        } else {
            999 // Never used
        }
    }

    fn determine_health_status(stats: &CourseStats, days_since_last: u64) -> HealthStatus {
        let total_files = stats.notes_count + stats.assignments_count;

        match (total_files, days_since_last) {
            (0, _) => HealthStatus::Critical,
            (_, 0..=3) => HealthStatus::Excellent,
            (_, 4..=7) if total_files > 3 => HealthStatus::Good,
            (_, 8..=14) if total_files > 1 => HealthStatus::Warning,
            _ => HealthStatus::Critical,
        }
    }

    /// Get current semester string
    pub fn get_current_semester(config: &Config) -> String {
        let now = chrono::Local::now();
        let year = now.year();
        let month = now.month();
        let is_spring = month <= 6;

        config.format_semester(year, is_spring)
    }

    /// Get course name from config with fallback to common courses
    pub fn resolve_course_name(course_id: &str, config: &Config) -> String {
        // Try user's courses first
        if let Some(name) = config.courses.get(course_id) {
            return name.clone();
        }

        // Fallback to common DTU courses
        crate::data::get_course_name(course_id)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SemesterInfo {
    pub current_semester: String,
    pub year: i32,
    pub is_spring: bool,
    pub format: crate::config::SemesterFormat,
}
