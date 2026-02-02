use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TypstConfig {
    /// Additional compile arguments
    pub compile_args: Vec<String>,

    /// Watch mode arguments
    pub watch_args: Vec<String>,

    /// Output directory for PDFs (relative to source)
    pub output_dir: Option<String>,

    /// Whether to clean PDFs before compiling
    pub clean_before_compile: bool,
}
