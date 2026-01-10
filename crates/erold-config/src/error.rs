//! Configuration error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Credentials not configured. Run 'erold login' first.")]
    CredentialsNotConfigured,

    #[error("Project not linked. Run 'erold link' first.")]
    ProjectNotLinked,
}
