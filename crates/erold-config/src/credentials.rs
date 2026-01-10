//! Credentials management

use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// API credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Erold API key (supports legacy `api_key` field name)
    #[serde(alias = "api_key")]
    pub erold_api_key: String,
    /// Tenant ID
    pub tenant_id: String,
    /// OpenAI API key for GPT models (optional, can be set via OPENAI_API_KEY env)
    #[serde(default)]
    pub openai_api_key: String,
}

impl Credentials {
    /// Load credentials from ~/.erold/credentials.toml
    ///
    /// Falls back to OPENAI_API_KEY environment variable if not in file.
    ///
    /// # Errors
    /// Returns error if credentials file not found or invalid
    pub fn load() -> Result<Self> {
        let path = Self::credentials_path()?;

        if !path.exists() {
            return Err(ConfigError::CredentialsNotConfigured);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut creds: Self = toml::from_str(&content)?;

        // Fall back to environment variable for OpenAI API key
        if creds.openai_api_key.is_empty() {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                creds.openai_api_key = key;
            }
        }

        Ok(creds)
    }

    /// Check if all required credentials are present
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.erold_api_key.is_empty()
            && !self.tenant_id.is_empty()
            && !self.openai_api_key.is_empty()
    }

    /// Save credentials to ~/.erold/credentials.toml
    ///
    /// # Errors
    /// Returns error if unable to write file
    pub fn save(&self) -> Result<()> {
        let path = Self::credentials_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Check if credentials are configured
    #[must_use]
    pub fn exists() -> bool {
        Self::credentials_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    fn credentials_path() -> Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| ConfigError::NotFound("home directory".to_string()))?;

        Ok(home.home_dir().join(".erold").join("credentials.toml"))
    }
}
