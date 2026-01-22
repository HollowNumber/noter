use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NotePreferences {
    /// Whether to automatically open files after creation
    pub auto_open_file: bool,

    /// Whether to open the file directory after creation
    pub auto_open_dir: bool,

    /// Include date in lecture note titles
    pub include_date_in_title: bool,

    /// Default sections for lecture notes
    pub lecture_sections: Vec<String>,

    /// Default sections for assignments
    pub assignment_sections: Vec<String>,

    /// Whether to create backup of existing files
    pub create_backups: bool,
}

impl Default for NotePreferences {
    fn default() -> Self {
        Self {
            auto_open_file: true,
            auto_open_dir: false,
            include_date_in_title: true,
            lecture_sections: vec![
                "Key Concepts".to_string(),
                "Mathematical Framework".to_string(),
                "Examples".to_string(),
                "Important Points".to_string(),
                "Questions & Follow-up".to_string(),
                "Connections to Previous Material".to_string(),
                "Next Class Preview".to_string(),
            ],
            assignment_sections: vec![
                "Problem 1".to_string(),
                "Problem 2".to_string(),
                "Problem 3".to_string(),
            ],
            create_backups: false,
        }
    }
}
