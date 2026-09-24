use clap_complete::engine::{CompletionCandidate, PathCompleter};

use crate::{config::Config, core::template::discovery::TemplateDiscovery};

/// Course IDs with the course name as help text.
pub fn course_ids() -> Vec<CompletionCandidate> {
    let Ok(config) = Config::load_readonly() else {
        return Vec::new();
    };

    let mut courses: Vec<_> = config.courses.iter().collect();
    courses.sort_by(|a, b| a.0.cmp(b.0));

    courses
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.clone().into())))
        .collect()
}

/// Files ending in .typ (directories are still offered so you can descend).
pub fn typst_files() -> PathCompleter {
    PathCompleter::file().filter(|p| p.extension().is_some_and(|ext| ext == "typ"))
}

pub fn variants() -> Vec<CompletionCandidate> {
    let Ok(user_config) = Config::load_readonly() else {
        return Vec::new();
    };

    let Ok(configs) = TemplateDiscovery::load_template_configs(&user_config) else {
        return Vec::new();
    };

    let variants = TemplateDiscovery::get_all_variants(&configs);
    let variant_names: Vec<_> = variants
        .into_iter()
        .map(|v| (v.name, v.display_name))
        .collect();

    variant_names
        .into_iter()
        .map(|(name, display_name)| {
            CompletionCandidate::new(name).help(Some(display_name.clone().into()))
        })
        .collect()
}
