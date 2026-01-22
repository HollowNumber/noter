//! Output utilities for consistent CLI presentation
//!
//! Handles different types of output including tables, lists, progress indicators,
//! and structured displays.

use colored::*;
use std::io::{self, Write};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TableColumn {
    pub header: String,
    pub width: usize,
    pub align: Alignment,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

pub struct OutputManager;

#[allow(dead_code)]
impl OutputManager {
    /// Print a formatted table with headers and data
    pub fn print_table(columns: &[TableColumn], rows: &[Vec<String>]) {
        if columns.is_empty() || rows.is_empty() {
            return;
        }

        Self::print_table_header(columns);
        Self::print_table_separator(columns);

        for row in rows {
            Self::print_table_row(columns, row);
        }
    }

    /// Print a simple list with bullet points
    pub fn print_list(items: &[String], bullet: Option<&str>) {
        let bullet = bullet.unwrap_or("•");
        for item in items {
            println!("  {} {}", bullet.dimmed(), item);
        }
    }

    /// Print a numbered list
    pub fn print_numbered_list(items: &[String]) {
        for (i, item) in items.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().dimmed(), item);
        }
    }

    /// Print a section header with optional icon
    pub fn print_section(title: &str, icon: Option<&str>) {
        let icon_str = icon.unwrap_or("📄");
        println!();
        println!("{} {}:", icon_str.blue(), title.bright_white());
        println!();
    }

    /// Print a separator line
    pub fn print_separator(width: Option<usize>) {
        let width = width.unwrap_or(50);
        println!("{}", "─".repeat(width).dimmed());
    }

    /// Print status with icon and color coding
    pub fn print_status(status: Status, message: &str) {
        match status {
            Status::Success => println!("{} {}", "✅".green(), message),
            Status::Warning => println!("{} {}", "⚠️".yellow(), message),
            Status::Error => println!("{} {}", "❌".red(), message),
            Status::Info => println!("{} {}", "ℹ️".blue(), message),
            Status::Loading => println!("{} {}", "⏳".blue(), message),
            Status::Complete => println!("{} {}", "🎉".green(), message),
        }
    }

    /// Print a progress bar (simple text-based)
    pub fn print_progress(current: usize, total: usize, description: Option<&str>) {
        let percentage = if total > 0 {
            (current as f32 / total as f32 * 100.0) as usize
        } else {
            0
        };

        let bar_width = 20;
        let filled = (percentage * bar_width) / 100;
        let empty = bar_width - filled;

        let bar = format!(
            "[{}{}]",
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed()
        );

        if let Some(desc) = description {
            print!(
                "\r{} {} {}% ({}/{}) - {}",
                "⏳".blue(),
                bar,
                percentage,
                current,
                total,
                desc
            );
        } else {
            print!(
                "\r{} {} {}% ({}/{})",
                "⏳".blue(),
                bar,
                percentage,
                current,
                total
            );
        }

        io::stdout().flush().unwrap();

        if current >= total {
            println!(); // New line when complete
        }
    }

    /// Print a key-value pair list
    pub fn print_key_value_pairs(pairs: &[(String, String)], indent: Option<usize>) {
        let indent_str = " ".repeat(indent.unwrap_or(2));
        let max_key_width = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

        for (key, value) in pairs {
            println!(
                "{}{}: {}",
                indent_str,
                format!("{:<width$}", key, width = max_key_width).bright_blue(),
                value
            );
        }
    }

    /// Print a box around text
    pub fn print_box(content: &str, title: Option<&str>) {
        let lines: Vec<&str> = content.lines().collect();
        let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let box_width = max_width + 4; // 2 spaces padding on each side

        // Top border
        if let Some(title) = title {
            let title_len = title.len();
            let padding = if box_width > title_len + 4 {
                (box_width - title_len - 4) / 2
            } else {
                0
            };

            println!(
                "┌{}┤ {} ├{}┐",
                "─".repeat(padding).dimmed(),
                title.bright_white(),
                "─".repeat(box_width - padding - title_len - 4).dimmed()
            );
        } else {
            println!("┌{}┐", "─".repeat(box_width - 2).dimmed());
        }

        // Content
        for line in lines {
            println!("│ {:<width$} │", line, width = max_width);
        }

        // Bottom border
        println!("└{}┘", "─".repeat(box_width - 2).dimmed());
    }

    /// Print command examples with syntax highlighting
    pub fn print_command_examples(examples: &[(&str, &str)]) {
        println!("{}", "Command Examples:".bright_green());
        for (command, description) in examples {
            println!(
                "  {} {}",
                command.bright_white(),
                format!("# {}", description).dimmed()
            );
        }
    }

    /// Clear the current line (for progress updates)
    pub fn clear_line() {
        print!("\r{}\r", " ".repeat(80));
        io::stdout().flush().unwrap();
    }

    // Private helper methods
    fn print_table_header(columns: &[TableColumn]) {
        print!("│");
        for column in columns {
            let padded = Self::pad_text(&column.header, column.width, &column.align);
            print!(" {} │", padded.bright_white().bold());
        }
        println!();
    }

    fn print_table_separator(columns: &[TableColumn]) {
        print!("├");
        for (i, column) in columns.iter().enumerate() {
            print!("{}", "─".repeat(column.width + 2));
            if i < columns.len() - 1 {
                print!("┼");
            }
        }
        println!("┤");
    }

    fn print_table_row(columns: &[TableColumn], row: &[String]) {
        print!("│");
        for (i, column) in columns.iter().enumerate() {
            let value = row.get(i).map(String::as_str).unwrap_or("");
            let padded = Self::pad_text(value, column.width, &column.align);
            print!(" {} │", padded);
        }
        println!();
    }

    fn pad_text(text: &str, width: usize, align: &Alignment) -> String {
        if text.len() >= width {
            return text[..width].to_string();
        }

        match align {
            Alignment::Left => format!("{:<width$}", text, width = width),
            Alignment::Right => format!("{:>width$}", text, width = width),
            Alignment::Center => {
                let padding = width - text.len();
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Status {
    Success,
    Warning,
    Error,
    Info,
    Loading,
    Complete,
}

/// Helper trait for easy status printing
pub trait StatusPrint {
    fn print_success(&self);
    fn print_warning(&self);
    fn print_error(&self);
    fn print_info(&self);
}

impl StatusPrint for str {
    fn print_success(&self) {
        OutputManager::print_status(Status::Success, self);
    }

    fn print_warning(&self) {
        OutputManager::print_status(Status::Warning, self);
    }

    fn print_error(&self) {
        OutputManager::print_status(Status::Error, self);
    }

    fn print_info(&self) {
        OutputManager::print_status(Status::Info, self);
    }
}

impl StatusPrint for String {
    fn print_success(&self) {
        self.as_str().print_success();
    }

    fn print_warning(&self) {
        self.as_str().print_warning();
    }

    fn print_error(&self) {
        self.as_str().print_error();
    }

    fn print_info(&self) {
        self.as_str().print_info();
    }
}
