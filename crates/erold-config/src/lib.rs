//! Configuration management for the Erold CLI
//!
//! Handles loading and saving configuration from:
//! - ~/.erold/config.toml (global config)
//! - ~/.erold/credentials.toml (API keys)
//! - .erold/project.json (project linking)

mod error;
mod types;
mod loader;
mod credentials;

pub use error::{ConfigError, Result};
pub use types::*;
pub use loader::ConfigLoader;
pub use credentials::Credentials;
