//! Configuration loader

use crate::error::{ConfigError, Result};
use crate::types::{EroldConfig, ProjectLink};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from default locations
    ///
    /// # Errors
    /// Returns error if config file is invalid
    pub fn load() -> Result<EroldConfig> {
        let path = Self::config_path()?;

        if !path.exists() {
            debug!("Config file not found, using defaults");
            return Ok(EroldConfig::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: EroldConfig = toml::from_str(&content)?;

        Ok(config)
    }

    /// Save configuration to default location
    ///
    /// # Errors
    /// Returns error if unable to write file
    pub fn save(config: &EroldConfig) -> Result<()> {
        let path = Self::config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Load project link from current directory
    ///
    /// # Errors
    /// Returns error if project not linked
    pub fn load_project_link(cwd: &Path) -> Result<ProjectLink> {
        let path = cwd.join(".erold").join("project.json");

        if !path.exists() {
            return Err(ConfigError::ProjectNotLinked);
        }

        let content = std::fs::read_to_string(&path)?;
        let link: ProjectLink = serde_json::from_str(&content)?;

        Ok(link)
    }

    /// Save project link to current directory
    ///
    /// # Errors
    /// Returns error if unable to write file
    pub fn save_project_link(cwd: &Path, link: &ProjectLink) -> Result<()> {
        let dir = cwd.join(".erold");
        std::fs::create_dir_all(&dir)?;

        let path = dir.join("project.json");
        let content = serde_json::to_string_pretty(link)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Find project link by walking up directory tree
    pub fn find_project_link(start: &Path) -> Result<(PathBuf, ProjectLink)> {
        let mut current = start.to_path_buf();

        loop {
            let erold_dir = current.join(".erold").join("project.json");
            if erold_dir.exists() {
                let link = Self::load_project_link(&current)?;
                return Ok((current, link));
            }

            if !current.pop() {
                return Err(ConfigError::ProjectNotLinked);
            }
        }
    }

    fn config_path() -> Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| ConfigError::NotFound("home directory".to_string()))?;

        Ok(home.home_dir().join(".erold").join("config.toml"))
    }
}
