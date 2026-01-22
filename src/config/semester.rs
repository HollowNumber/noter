use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum SemesterFormat {
    /// "2024 Spring", "2024 Fall"
    #[default]
    YearSeason,
    /// "Spring 2024", "Fall 2024"
    SeasonYear,
    /// "S24", "F24"
    ShortForm,
    /// Custom format string
    Custom(String),
}
