//! Development data generation utilities
//!
//! This module provides core functionality for generating realistic test data
//! including courses, notes, assignments, and study materials for development
//! and testing purposes.

use anyhow::Result;
use chrono::{Duration, Utc};
#[cfg(feature = "dev-tools")]
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::{
    config::Config,
    display::output::{OutputManager, Status},
};

// Global tracking of generated courses for cleanup
#[cfg(feature = "dev-tools")]
static GENERATED_COURSES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[cfg(feature = "dev-tools")]
fn get_generated_courses() -> &'static Mutex<HashSet<String>> {
    GENERATED_COURSES.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Development data generator for creating realistic test content
#[cfg(feature = "dev-tools")]
pub struct DevDataGenerator {
    rng: StdRng,
}

#[cfg(feature = "dev-tools")]
impl DevDataGenerator {
    /// Create a new generator with a deterministic seed for reproducible data
    pub fn new() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create a deterministic seed based on current time for some variation
        let mut hasher = DefaultHasher::new();
        Utc::now().timestamp().hash(&mut hasher);
        let seed = hasher.finish();

        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Create a generator with a specific seed for fully reproducible data
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Generate high-yield simulation data with many courses and files
    pub fn generate_high_yield_simulation(
        &mut self,
        config: &mut Config,
    ) -> Result<GenerationStats> {
        let notes_dir = Path::new(&config.paths.notes_dir);

        OutputManager::print_status(Status::Loading, "Setting up high-yield simulation...");
        fs::create_dir_all(notes_dir)?;

        let courses = self.get_predefined_courses();
        OutputManager::print_status(
            Status::Info,
            &format!("Generating {} courses", courses.len()),
        );

        let mut stats = GenerationStats::new();

        // Add courses to config and track them
        for course in &courses {
            config
                .courses
                .insert(course.code.clone(), course.name.clone());

            // Track generated course for cleanup
            if let Ok(mut generated) = get_generated_courses().lock() {
                generated.insert(course.code.clone());
            }
        }

        // Save updated config
        config.save()?;

        for course in &courses {
            let course_dir = notes_dir.join(&course.code);
            fs::create_dir_all(&course_dir)?;

            // Generate course info file
            self.generate_course_info(&course_dir, course)?;
            stats.files_created += 1;

            // Generate lecture notes (20-35 per course for high-yield)
            let note_count = self.rng.random_range(20..35);
            for i in 1..=note_count {
                self.generate_lecture_note(&course_dir, course, i)?;
                stats.notes_created += 1;
                stats.files_created += 1;
            }

            // Generate assignments (5-8 per course)
            let assignment_count = self.rng.random_range(5..9);
            for i in 1..=assignment_count {
                self.generate_assignment(&course_dir, course, i)?;
                stats.assignments_created += 1;
                stats.files_created += 1;
            }

            // Generate study materials
            self.generate_study_materials(&course_dir, course)?;
            stats.files_created += 3; // Summary, cheat sheet, study guide
            stats.courses_created += 1;
        }

        OutputManager::print_status(
            Status::Success,
            &format!(
                "High-yield simulation complete! Generated {} courses, {} notes, {} assignments, {} total files",
                stats.courses_created,
                stats.notes_created,
                stats.assignments_created,
                stats.files_created
            ),
        );

        Ok(stats)
    }

    /// Generate sample data with specific parameters
    pub fn generate_sample_data(
        &mut self,
        config: &mut Config,
        course_count: usize,
        notes_per_course: usize,
        assignments_per_course: usize,
    ) -> Result<GenerationStats> {
        let notes_dir = Path::new(&config.paths.notes_dir);

        OutputManager::print_status(
            Status::Loading,
            &format!(
                "Generating {} courses with {} notes and {} assignments each",
                course_count, notes_per_course, assignments_per_course
            ),
        );

        fs::create_dir_all(notes_dir)?;

        let courses: Vec<Course> = (0..course_count)
            .map(|i| self.generate_realistic_course(i))
            .collect();

        let mut stats = GenerationStats::new();

        // Add courses to config and track them
        for course in &courses {
            config
                .courses
                .insert(course.code.clone(), course.name.clone());

            // Track generated course for cleanup
            if let Ok(mut generated) = get_generated_courses().lock() {
                generated.insert(course.code.clone());
            }
        }

        // Save updated config
        config.save()?;

        for course in &courses {
            let course_dir = notes_dir.join(&course.code);
            fs::create_dir_all(&course_dir)?;

            self.generate_course_info(&course_dir, course)?;
            stats.files_created += 1;

            for i in 1..=notes_per_course {
                self.generate_lecture_note(&course_dir, course, i)?;
                stats.notes_created += 1;
                stats.files_created += 1;
            }

            for i in 1..=assignments_per_course {
                self.generate_assignment(&course_dir, course, i)?;
                stats.assignments_created += 1;
                stats.files_created += 1;
            }

            stats.courses_created += 1;
        }

        OutputManager::print_status(Status::Success, "Sample data generation complete!");
        Ok(stats)
    }

    /// Clean all generated development data
    pub fn clean_dev_data(config: &mut Config) -> Result<CleanupStats> {
        let notes_dir = Path::new(&config.paths.notes_dir);

        if !notes_dir.exists() {
            OutputManager::print_status(Status::Info, "No notes directory found, nothing to clean");
            return Ok(CleanupStats::new());
        }

        OutputManager::print_status(Status::Loading, "Cleaning dev data...");

        let predefined_dev_courses = [
            "02101", "02102", "02105", "02110", "02157", "02180", "02223", "02266", "02343",
            "02450",
        ];

        // Get all courses to remove (predefined + dynamically generated)
        let mut all_courses_to_remove = HashSet::new();

        // Add predefined dev courses
        for course_code in &predefined_dev_courses {
            all_courses_to_remove.insert(course_code.to_string());
        }

        // Add dynamically generated courses (from tracking and pattern matching)
        if let Ok(generated) = get_generated_courses().lock() {
            for course_code in generated.iter() {
                all_courses_to_remove.insert(course_code.clone());
            }
        }

        // Also detect courses that match our generated pattern (02100xx format)
        for (course_code, _) in &config.courses {
            if course_code.starts_with("021") && course_code.len() == 7 {
                // This looks like a dynamically generated course code
                all_courses_to_remove.insert(course_code.clone());
            }
        }

        let mut stats = CleanupStats::new();

        // Remove courses from config
        let mut courses_removed_from_config = 0;
        for course_code in &all_courses_to_remove {
            if config.courses.remove(course_code).is_some() {
                courses_removed_from_config += 1;
            }
        }

        // Save updated config if courses were removed
        if courses_removed_from_config > 0 {
            config.save()?;
            OutputManager::print_status(
                Status::Info,
                &format!(
                    "Removed {} courses from config",
                    courses_removed_from_config
                ),
            );
        }

        for course_code in &all_courses_to_remove {
            let course_dir = notes_dir.join(course_code);
            if course_dir.exists() {
                // Count files before removal
                if let Ok(entries) = fs::read_dir(&course_dir) {
                    stats.files_removed += entries.count();
                }

                fs::remove_dir_all(&course_dir)?;
                OutputManager::print_status(Status::Info, &format!("Removed {}", course_code));
                stats.directories_removed += 1;
            }
        }

        // Clear the generated courses tracking
        if let Ok(mut generated) = get_generated_courses().lock() {
            generated.clear();
        }

        OutputManager::print_status(
            Status::Success,
            &format!(
                "Dev data cleanup complete! Removed {} directories and {} files",
                stats.directories_removed, stats.files_removed
            ),
        );

        Ok(stats)
    }

    fn generate_course_info(&self, course_dir: &Path, course: &Course) -> Result<()> {
        let content = super::sample::CourseInfoTemplate::generate(course);
        let file_path = course_dir.join("course_info.typ");
        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_lecture_note(
        &mut self,
        course_dir: &Path,
        course: &Course,
        lecture_num: usize,
    ) -> Result<()> {
        let topics = super::sample::get_lecture_topics(&course.code);
        let topic = &topics[lecture_num % topics.len()];
        let date = Utc::now() - Duration::days(self.rng.random_range(1..180));

        let content = super::sample::LectureTemplate::generate(
            lecture_num,
            topic,
            course,
            &date.format("%Y-%m-%d").to_string(),
        );

        let file_path = course_dir.join(format!("lecture_{:02}.typ", lecture_num));
        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_assignment(
        &mut self,
        course_dir: &Path,
        course: &Course,
        assignment_num: usize,
    ) -> Result<()> {
        let assignment_types = [
            "Programming",
            "Theoretical",
            "Analysis",
            "Design",
            "Research",
        ];
        let assignment_type = assignment_types[assignment_num % assignment_types.len()];
        let due_date = Utc::now() + Duration::days(self.rng.random_range(7..30));
        let points = self.rng.random_range(50..100);

        let assignments_dir = course_dir.join("assignments");
        fs::create_dir_all(&assignments_dir)?;

        let content = super::sample::AssignmentTemplate::generate(
            assignment_num,
            assignment_type,
            course,
            &due_date.format("%Y-%m-%d").to_string(),
            points,
        );

        let file_path = assignments_dir.join(format!("assignment_{:02}.typ", assignment_num));
        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_study_materials(&self, course_dir: &Path, course: &Course) -> Result<()> {
        // Generate course summary
        let summary_content =
            super::sample::StudyMaterialsTemplate::generate("Summary", course, "Course Overview");
        let summary_path = course_dir.join("course_summary.typ");
        fs::write(summary_path, summary_content)?;

        // Generate cheat sheet
        let cheat_sheet_content = super::sample::StudyMaterialsTemplate::generate(
            "Cheat Sheet",
            course,
            "Quick Reference",
        );
        let cheat_sheet_path = course_dir.join("cheat_sheet.typ");
        fs::write(cheat_sheet_path, cheat_sheet_content)?;

        // Generate study guide
        let study_guide_content = super::sample::StudyMaterialsTemplate::generate(
            "Study Guide",
            course,
            "Exam Preparation",
        );
        let study_guide_path = course_dir.join("study_guide.typ");
        fs::write(study_guide_path, study_guide_content)?;

        Ok(())
    }

    fn get_predefined_courses(&self) -> Vec<Course> {
        vec![
            Course {
                code: "02101".to_string(),
                name: "Introduction to Programming".to_string(),
                description: "Basic programming concepts using Python".to_string(),
                credits: 5.0,
                semester: "Fall".to_string(),
            },
            Course {
                code: "02102".to_string(),
                name: "Algorithms and Data Structures 1".to_string(),
                description: "Fundamental algorithms and data structures".to_string(),
                credits: 10.0,
                semester: "Spring".to_string(),
            },
            Course {
                code: "02105".to_string(),
                name: "Algorithms and Data Structures 2".to_string(),
                description: "Advanced algorithms and complexity analysis".to_string(),
                credits: 10.0,
                semester: "Fall".to_string(),
            },
            Course {
                code: "02110".to_string(),
                name: "Algorithms and Data Structures".to_string(),
                description: "Comprehensive study of algorithms".to_string(),
                credits: 7.5,
                semester: "Spring".to_string(),
            },
            Course {
                code: "02157".to_string(),
                name: "Functional Programming".to_string(),
                description: "Programming with functional languages".to_string(),
                credits: 5.0,
                semester: "Fall".to_string(),
            },
            Course {
                code: "02180".to_string(),
                name: "Introduction to Artificial Intelligence".to_string(),
                description: "Basic AI concepts and techniques".to_string(),
                credits: 7.5,
                semester: "Spring".to_string(),
            },
            Course {
                code: "02223".to_string(),
                name: "Model-Based System Development".to_string(),
                description: "System modeling and verification".to_string(),
                credits: 7.5,
                semester: "Fall".to_string(),
            },
            Course {
                code: "02266".to_string(),
                name: "User Experience Engineering".to_string(),
                description: "Human-computer interaction and UX design".to_string(),
                credits: 5.0,
                semester: "Spring".to_string(),
            },
            Course {
                code: "02343".to_string(),
                name: "Functional and Parallel Programming".to_string(),
                description: "Advanced functional programming concepts".to_string(),
                credits: 7.5,
                semester: "Fall".to_string(),
            },
            Course {
                code: "02450".to_string(),
                name: "Introduction to Machine Learning and Data Mining".to_string(),
                description: "Machine learning algorithms and applications".to_string(),
                credits: 7.5,
                semester: "Spring".to_string(),
            },
        ]
    }

    fn generate_realistic_course(&self, index: usize) -> Course {
        let course_data = [
            (
                "Advanced Calculus",
                "Mathematical analysis and applications",
            ),
            (
                "Database Systems",
                "Design and implementation of database systems",
            ),
            (
                "Computer Networks",
                "Network protocols and distributed systems",
            ),
            ("Software Engineering", "Large-scale software development"),
            (
                "Operating Systems",
                "System-level programming and OS concepts",
            ),
            ("Computer Graphics", "3D rendering and visualization"),
            ("Cryptography", "Security and encryption algorithms"),
            (
                "Distributed Systems",
                "Building scalable distributed applications",
            ),
            (
                "Compiler Design",
                "Language implementation and optimization",
            ),
            (
                "Computer Vision",
                "Image processing and pattern recognition",
            ),
        ];

        let (name, description) = course_data[index % course_data.len()];

        Course {
            code: format!("0{}{:02}", 2100 + (index / 10), index % 100),
            name: name.to_string(),
            description: description.to_string(),
            credits: [5.0, 7.5, 10.0][index % 3],
            semester: if index % 2 == 0 { "Fall" } else { "Spring" }.to_string(),
        }
    }
}

#[cfg(feature = "dev-tools")]
impl Default for DevDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Course structure for data generation
#[cfg(feature = "dev-tools")]
#[derive(Debug, Clone)]
pub struct Course {
    pub code: String,
    pub name: String,
    pub description: String,
    pub credits: f32,
    pub semester: String,
}

/// Statistics for data generation operations
#[cfg(feature = "dev-tools")]
#[derive(Debug, Default)]
pub struct GenerationStats {
    pub courses_created: usize,
    pub notes_created: usize,
    pub assignments_created: usize,
    pub files_created: usize,
}

#[cfg(feature = "dev-tools")]
impl GenerationStats {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Statistics for cleanup operations
#[cfg(feature = "dev-tools")]
#[derive(Debug, Default)]
pub struct CleanupStats {
    pub directories_removed: usize,
    pub files_removed: usize,
}

#[cfg(feature = "dev-tools")]
impl CleanupStats {
    pub fn new() -> Self {
        Self::default()
    }
}
