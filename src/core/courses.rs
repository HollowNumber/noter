//! Course management business logic
//!
//! Handles course operations like adding, removing, listing courses
//! without CLI-specific concerns.

use anyhow::Result;

use crate::config::Config;

pub struct CourseManager<'a> {
    config: &'a mut Config,
}

#[allow(dead_code)]
impl<'a> CourseManager<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Self { config }
    }

    pub fn add_course(&mut self, course_id: &str, course_name: &str) -> Result<()> {
        if self.config.courses.contains_key(course_id) {
            return Err(anyhow::anyhow!("Course {} already exists", course_id));
        }

        self.config
            .add_course(course_id.to_string(), course_name.to_string())?;
        Ok(())
    }

    pub fn remove_course(&mut self, course_id: &str) -> Result<String> {
        if let Some(course_name) = self.config.courses.get(course_id) {
            let course_name = course_name.clone();
            self.config.remove_course(course_id)?;
            Ok(course_name)
        } else {
            Err(anyhow::anyhow!("Course {} not found", course_id))
        }
    }

    pub fn list_courses(&self) -> Vec<(String, String)> {
        self.config.list_courses()
    }

    pub fn get_course_name(&self, course_id: &str) -> Option<String> {
        self.config.courses.get(course_id).cloned()
    }
}

/// Common DTU courses organized by category
#[must_use]
pub const fn get_common_courses()
-> &'static [(&'static str, &'static [(&'static str, &'static str)])] {
    &[
        (
            "Mathematics & Computer Science",
            &[
                ("01005", "Advanced Engineering Mathematics 1"),
                ("01006", "Advanced Engineering Mathematics 2"),
                ("02101", "Introduction to Programming"),
                ("02102", "Algorithms and Data Structures"),
                // ... more courses
            ],
        ),
        (
            "Physics & Engineering",
            &[
                ("25200", "Classical Physics 1"),
                ("25201", "Classical Physics 2"),
                // ... more courses
            ],
        ),
    ]
}
