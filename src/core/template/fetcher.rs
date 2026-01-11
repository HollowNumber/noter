//! GitHub template fetcher
//!
//! Handles downloading templates from multiple GitHub repositories with fallback support

// TOOD: Rewrite this to also accept gitlab. and other hosting services.
// Seems pretty big scope, so not sure how feasible this is.

use crate::config::{Config, Metadata, ObsidianIntegrationConfig, TemplateRepository};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_TEMPLATE_REPO: &str = "HollowNumber/dtu-note-template";
const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug)]
pub struct TemplateDownloadResult {
    pub version: String,
    pub installed_path: PathBuf,
    pub is_cached: bool,
}

pub struct Fetcher;

#[allow(dead_code)]
impl Fetcher {
    /// Get the latest release information from a specific GitHub repository
    pub fn get_latest_release(repo: &str) -> Result<GitHubRelease> {
        let url = format!("{GITHUB_API_BASE}/repos/{repo}/releases/latest");

        let mut response = ureq::get(&url)
            .header("User-Agent", "dtu-notes-cli")
            .call()
            .context("Failed to fetch latest release information")?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!(
                "GitHub API request failed with status: {}",
                response.status()
            ));
        }

        let body_str = response
            .body_mut()
            .read_to_string()
            .context("Failed to read response body")?;

        let release: GitHubRelease =
            serde_json::from_str(&body_str).context("Failed to parse GitHub API response")?;

        Ok(release)
    }

    /// Download and install templates from configured repositories with fallback
    pub fn download_and_install_templates(
        config: &Config,
        force_update: bool,
    ) -> Result<Vec<TemplateDownloadResult>> {
        let mut results = Vec::new();
        let mut success = false;

        // Try custom repositories first
        for repo_config in &config.templates.custom_repositories {
            if !repo_config.enabled {
                continue;
            }

            match Self::download_from_repository(config, repo_config, force_update) {
                Ok(result) => {
                    results.push(result);
                    success = true;
                    break; // Use first successful repository
                }
                Err(e) => {
                    eprintln!("Failed to download from {}: {}", repo_config.name, e);
                    continue;
                }
            }
        }

        // Fallback to official repository if no custom repos succeeded
        if !success && config.templates.use_official_fallback {
            let official_repo = TemplateRepository {
                name: "dtu_template".to_string(),
                repository: DEFAULT_TEMPLATE_REPO.to_string(),
                version: None,
                branch: None,
                template_path: None,
                enabled: true,
            };

            match Self::download_from_repository(config, &official_repo, force_update) {
                Ok(result) => {
                    results.push(result);
                    success = true;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to download from official repository: {}",
                        e
                    ));
                }
            }
        }

        if !success {
            return Err(anyhow::anyhow!(
                "No template repositories were successfully downloaded"
            ));
        }

        Ok(results)
    }

    /// Download from a specific repository configuration
    fn download_from_repository(
        config: &Config,
        repo_config: &TemplateRepository,
        force_update: bool,
    ) -> Result<TemplateDownloadResult> {
        let release = Self::get_latest_release(&repo_config.repository)?;

        // Check if we already have this version cached
        let cache_path = Self::get_cache_path(&repo_config.name, &release.tag_name)?;
        let template_installed_marker = Path::new(&config.paths.templates_dir)
            .join(&repo_config.name)
            .join(".template_version");

        let is_already_installed = if template_installed_marker.exists() {
            if let Ok(installed_version) = fs::read_to_string(&template_installed_marker) {
                installed_version.trim() == release.tag_name
            } else {
                false
            }
        } else {
            false
        };

        if is_already_installed && !force_update {
            return Ok(TemplateDownloadResult {
                version: release.tag_name,
                installed_path: PathBuf::from(&config.paths.templates_dir).join(&repo_config.name),
                is_cached: true,
            });
        }

        // Download if not cached or force update
        if !cache_path.exists() || force_update {
            Self::download_release(&release, &cache_path)?;
        }

        // Extract and install template
        Self::extract_and_install(
            &cache_path,
            &config.paths.templates_dir,
            &config.paths.typst_packages_dir,
            &release.tag_name,
            repo_config,
        )?;

        Ok(TemplateDownloadResult {
            version: release.tag_name,
            installed_path: PathBuf::from(&config.paths.templates_dir).join(&repo_config.name),
            is_cached: cache_path.exists(),
        })
    }

    /// Get cache directory path for templates
    fn get_cache_path(repo_name: &str, version: &str) -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .context("Could not determine cache directory")?
            .join("dtu-notes")
            .join("templates");

        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir.join(format!("{}-{}.tar.gz", repo_name, version)))
    }

    /// Download the release asset (not tarball)
    fn download_release(release: &GitHubRelease, cache_path: &Path) -> Result<()> {
        // Look for a release asset that looks like a template (zip or tar.gz)
        let template_asset = release
            .assets
            .iter()
            .find(|asset| {
                let name = asset.name.to_lowercase();
                name.contains("dtu-template") || name.contains("template")
            })
            .or_else(|| {
                // Fallback: look for any zip or tar.gz file
                release.assets.iter().find(|asset| {
                    let name = asset.name.to_lowercase();
                    name.ends_with(".zip") || name.ends_with(".tar.gz")
                })
            });

        let download_url = if let Some(asset) = template_asset {
            &asset.browser_download_url
        } else {
            // Fallback to tarball if no assets found
            &release.tarball_url
        };

        let response = ureq::get(download_url)
            .header("User-Agent", "dtu-notes-cli")
            .call()
            .context("Failed to download template release")?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!(
                "Failed to download template: HTTP {}",
                response.status()
            ));
        }

        // Read response body using ureq's read_to_vec method
        let bytes = response
            .into_body()
            .read_to_vec()
            .context("Failed to read response body")?;

        // Ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(cache_path, bytes).context("Failed to write downloaded template to cache")?;

        Ok(())
    }

    /// Extract and install template files
    fn extract_and_install(
        archive_path: &Path,
        _templates_dir: &str,
        typst_packages_dir: &str,
        _version: &str,
        repo_config: &TemplateRepository,
    ) -> Result<()> {
        // For official template, extract directly to dtu-template directory
        let is_official_template = repo_config.repository == "HollowNumber/dtu-note-template"
            || repo_config.name == "dtu_template";

        if is_official_template {
            // Extract directly to typst packages/local
            let target_dir = Path::new(typst_packages_dir);
            fs::create_dir_all(target_dir)?;

            let dtu_template_dir = target_dir.join("dtu-template");

            // Check if the archive is a zip or tar.gz file
            let archive_name = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if archive_name.ends_with(".zip") {
                // Handle ZIP file using zip crate
                use zip::ZipArchive;

                let file = fs::File::open(archive_path)
                    .context("Failed to open downloaded template file")?;
                let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

                // Extract with unwrapped root directory - this automatically handles
                // archives that have a single root folder and extracts contents directly
                archive
                    .extract(&dtu_template_dir)
                    .context("Failed to extract ZIP file")?;
            } else {
                // Handle TAR.GZ file (fallback)
                use flate2::read::GzDecoder;
                use tar::Archive;

                let file = fs::File::open(archive_path)
                    .context("Failed to open downloaded template file")?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);

                // Extract the archive directly to a temporary location
                let temp_dir = target_dir.join("temp_extract");
                if temp_dir.exists() {
                    fs::remove_dir_all(&temp_dir)?;
                }
                fs::create_dir_all(&temp_dir)?;

                archive.unpack(&temp_dir)?;

                // Look for the extracted directory and move it to "dtu-template"
                let extracted_dirs: Vec<_> = fs::read_dir(&temp_dir)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                            && (entry
                                .file_name()
                                .to_string_lossy()
                                .starts_with("dtu-note-template-")
                                || entry
                                    .file_name()
                                    .to_string_lossy()
                                    .starts_with("dtu-template"))
                    })
                    .collect();

                if let Some(extracted_dir) = extracted_dirs.first() {
                    fs::rename(extracted_dir.path(), &dtu_template_dir)?;
                }

                // Clean up temp directory
                if temp_dir.exists() {
                    fs::remove_dir_all(&temp_dir)?;
                }
            }
        } else {
            // For custom templates, extract to the template name directory
            let target_dir = Path::new(typst_packages_dir).join(&repo_config.name);
            if target_dir.exists() {
                fs::remove_dir_all(&target_dir)?;
            }
            fs::create_dir_all(&target_dir)?;

            // Handle both zip and tar.gz
            let archive_name = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if archive_name.ends_with(".zip") {
                use zip::ZipArchive;

                let file = fs::File::open(archive_path)
                    .context("Failed to open downloaded template file")?;
                let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

                // Extract with unwrapped root directory
                archive
                    .extract(&target_dir)
                    .context("Failed to extract ZIP file")?;
            } else {
                use flate2::read::GzDecoder;
                use tar::Archive;

                let file = fs::File::open(archive_path)
                    .context("Failed to open downloaded template file")?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                archive.unpack(&target_dir)?;
            }
        }

        Ok(())
    }

    /// Copy template structure preserving directory layout
    fn copy_template_structure(source: &Path, dest: &Path) -> Result<()> {
        use crate::core::files::FileOperations;

        if !source.exists() {
            return Err(anyhow::anyhow!(
                "Source directory does not exist: {}",
                source.display()
            ));
        }

        // Ensure destination exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the entire directory structure
        FileOperations::copy_dir_recursive(source, dest)?;

        Ok(())
    }

    /// Check if templates are installed and get version info
    pub fn check_template_status(config: &Config) -> Result<Vec<(String, Option<String>)>> {
        let mut statuses = Vec::new();

        // Check custom repositories
        for repo_config in config
            .templates
            .custom_repositories
            .iter()
            .filter(|r| r.enabled)
        {
            let version = Self::get_custom_template_version(config, &repo_config.name)?;
            statuses.push((repo_config.name.clone(), version));
        }

        // Check official template if fallback is enabled
        if config.templates.use_official_fallback {
            let version = Self::get_official_template_version(&config.paths.typst_packages_dir);
            statuses.push(("dtu_template".to_string(), version));
        }

        Ok(statuses)
    }

    /// Get version information for a custom template
    fn get_custom_template_version(config: &Config, template_name: &str) -> Result<Option<String>> {
        let version_marker = Path::new(&config.paths.templates_dir)
            .join(template_name)
            .join(".template_version");

        if version_marker.exists() {
            let version =
                fs::read_to_string(&version_marker).context("Failed to read template version")?;
            return Ok(Some(version.trim().to_string()));
        }

        let template_dir = Path::new(&config.paths.templates_dir).join(template_name);
        Ok(if template_dir.exists() {
            Some("unknown".to_string())
        } else {
            None
        })
    }

    /// Get version information for the official dtu-template
    fn get_official_template_version(typst_packages_dir: &str) -> Option<String> {
        let dtu_template_dir = Path::new(typst_packages_dir).join("dtu-template");

        if !dtu_template_dir.exists() {
            return None;
        }

        let version = Self::find_template_version_in_directory(&dtu_template_dir)
            .unwrap_or_else(|| "unknown".to_string());

        Some(version)
    }

    /// Find the template version by scanning version directories
    fn find_template_version_in_directory(template_dir: &Path) -> Option<String> {
        let entries = fs::read_dir(template_dir).ok()?;

        let versions: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| Self::extract_version_from_directory(&entry.path()))
            .collect();

        if versions.is_empty() {
            None
        } else {
            // Sort versions and take the highest one
            let mut sorted_versions = versions;
            sorted_versions.sort();
            sorted_versions.into_iter().last()
        }
    }

    /// Extract version from a directory path by checking for version patterns and typst.toml
    fn extract_version_from_directory(path: &Path) -> Option<String> {
        if !path.is_dir() {
            return None;
        }

        let name = path.file_name()?.to_str()?;

        // Check if this looks like a version directory
        let is_version_dir =
            name.chars().next().is_some_and(|c| c.is_ascii_digit()) || name.starts_with('v');

        if !is_version_dir {
            return None;
        }

        // Try to read version from typst.toml
        let typst_toml = path.join("typst.toml");
        if let Ok(toml_content) = fs::read_to_string(&typst_toml) {
            if let Some(version) = Self::parse_version_from_toml(&toml_content) {
                return Some(version);
            }
        }

        // Fallback to directory name
        Some(name.to_string())
    }

    /// Parse version from typst.toml content
    fn parse_version_from_toml(toml_content: &str) -> Option<String> {
        toml_content
            .lines()
            .find(|line| line.starts_with("version"))
            .and_then(|line| line.split('=').nth(1))
            .map(|version| version.trim().trim_matches('"').to_string())
    }

    /// Legacy method for backward compatibility
    pub fn download_and_install_template(
        templates_dir: &str,
        typst_packages_dir: &str,
        force_update: bool,
    ) -> Result<TemplateDownloadResult> {
        // Create a minimal config for backward compatibility
        use crate::config::UserTemplateConfig;
        let template_config = UserTemplateConfig::default();

        let config = Config {
            author: "Unknown".to_string(),
            preferred_editor: None,
            template_version: "0.1.0".to_string(),
            semester_format: crate::config::SemesterFormat::YearSeason,
            note_preferences: crate::config::NotePreferences::default(),
            paths: crate::config::PathConfig {
                notes_dir: "notes".to_string(),
                obsidian_dir: "obsidian-vault".to_string(),
                templates_dir: templates_dir.to_string(),
                typst_packages_dir: typst_packages_dir.to_string(),
            },
            templates: template_config,
            typst: crate::config::TypstConfig::default(),
            search: crate::config::SearchConfig::default(),
            courses: std::collections::HashMap::new(),
            obsidian_integration: ObsidianIntegrationConfig::default(),
            metadata: Metadata::default(),
        };

        let results = Self::download_and_install_templates(&config, force_update)?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No templates were downloaded"))
    }

    /// Update templates to latest versions
    pub fn update_templates(config: &Config) -> Result<Vec<TemplateDownloadResult>> {
        Self::download_and_install_templates(config, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fetch_latest_release() {
        let result = Fetcher::get_latest_release(DEFAULT_TEMPLATE_REPO);
        assert!(result.is_ok(), "Should be able to fetch latest release");

        let release = result.unwrap();
        assert!(!release.tag_name.is_empty());
        assert!(!release.tarball_url.is_empty());
    }

    #[test]
    fn test_cache_path_generation() {
        let path = Fetcher::get_cache_path("test-template", "v1.0.0").unwrap();
        assert!(
            path.to_string_lossy()
                .contains("test-template-v1.0.0.tar.gz")
        );
    }

    #[test]
    fn test_template_status_check_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            author: "Test".to_string(),
            preferred_editor: None,
            template_version: "0.1.0".to_string(),
            semester_format: crate::config::SemesterFormat::YearSeason,
            note_preferences: crate::config::NotePreferences::default(),
            paths: crate::config::PathConfig {
                notes_dir: "notes".to_string(),
                obsidian_dir: "obsidian-vault".to_string(),
                templates_dir: temp_dir.path().to_str().unwrap().to_string(),
                typst_packages_dir: "packages".to_string(),
            },
            templates: crate::config::UserTemplateConfig::default(),
            typst: crate::config::TypstConfig::default(),
            search: crate::config::SearchConfig::default(),
            courses: std::collections::HashMap::new(),
            obsidian_integration: todo!(),
            metadata: todo!(),
        };

        let status = Fetcher::check_template_status(&config).unwrap();
        assert_eq!(status, vec![("dtu_template".to_string(), None)]);
    }
}
