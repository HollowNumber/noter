//! Output formatting utilities
//!
//! Consistent formatting for different types of output.

use crate::core::search_engine::SearchMatch;
use colored::*;

pub struct Formatters;

#[allow(dead_code)]
impl Formatters {
    pub fn format_course_list(courses: &[(String, String)]) -> String {
        if courses.is_empty() {
            return format!("{} No courses configured.", "ℹ️".blue());
        }

        let mut output = format!("{} Your DTU Courses:\n\n", "🎓".blue());

        for (course_id, course_name) in courses {
            output.push_str(&format!("  {} - {}\n", course_id.yellow(), course_name));
        }

        output.push('\n');
        output.push_str(&format!(
            "{} Total: {} courses\n",
            "📊".blue(),
            courses.len().to_string().green()
        ));

        output
    }

    pub fn format_search_results(results: &[SearchMatch], query: &str) -> String {
        if results.is_empty() {
            return "No results found".to_string();
        }

        let mut output = format!(
            "{} Search Results for '{}':\n\n",
            "🔍".blue(),
            query.bright_white()
        );

        for result in results {
            let highlighted = Self::highlight_precise_match(
                &result.line_content,
                result.match_start,
                result.match_end,
            );
            output.push_str(&format!(
                "{}:{}: {}\n",
                result.file_path.display().to_string().bright_blue(),
                result.line_number.to_string().dimmed(),
                highlighted
            ));
        }

        output.push_str(&format!(
            "\n{} {} results found\n",
            "📊".blue(),
            results.len().to_string().green()
        ));
        output
    }

    pub fn format_status_section(title: &str, icon: &str, content: &str) -> String {
        format!("{} {}:\n{}\n", icon.blue(), title, content)
    }

    pub fn format_success(message: &str) -> String {
        format!("{} {}", "✅".green(), message)
    }

    pub fn format_warning(message: &str) -> String {
        format!("{} {}", "⚠️".yellow(), message)
    }

    pub fn format_error(message: &str) -> String {
        format!("{} {}", "❌".red(), message)
    }

    pub fn format_info(message: &str) -> String {
        format!("{} {}", "ℹ️".blue(), message)
    }

    fn highlight_match(line: &str, query: &str) -> String {
        // Case-insensitive highlighting
        let lower_line = line.to_lowercase();
        let lower_query = query.to_lowercase();

        if let Some(pos) = lower_line.find(&lower_query) {
            let result = line.to_string();
            let actual_match = &line[pos..pos + query.len()];
            result.replace(actual_match, &format!("{}", actual_match.bright_yellow()))
        } else {
            line.to_string()
        }
    }

    // Fixed: Handle Unicode characters properly by converting byte positions to character positions
    fn highlight_precise_match(line: &str, match_start: usize, match_end: usize) -> String {
        // Convert byte positions to character positions
        let char_indices: Vec<(usize, char)> = line.char_indices().collect();
        let chars: Vec<char> = line.chars().collect();

        // Find character positions that correspond to the byte positions
        let char_start = char_indices
            .iter()
            .position(|(byte_pos, _)| *byte_pos >= match_start);
        let char_end = char_indices
            .iter()
            .position(|(byte_pos, _)| *byte_pos >= match_end);

        match (char_start, char_end) {
            (Some(start), Some(end))
                if start < chars.len() && end <= chars.len() && start < end =>
            {
                let before: String = chars[..start].iter().collect();
                let matched: String = chars[start..end].iter().collect();
                let after: String = chars[end..].iter().collect();
                format!("{}{}{}", before, matched.bright_yellow().bold(), after)
            }
            (Some(start), None) if start < chars.len() => {
                // If match_end is beyond the string, highlight from start to end
                let before: String = chars[..start].iter().collect();
                let matched: String = chars[start..].iter().collect();
                format!("{}{}", before, matched.bright_yellow().bold())
            }
            _ => {
                // Fallback: if positions are invalid, return original line
                line.to_string()
            }
        }
    }
}
