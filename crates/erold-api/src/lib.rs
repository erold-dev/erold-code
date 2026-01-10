//! Erold API Client
//!
//! HTTP client for interacting with the Erold API.
//! Handles projects, tasks, knowledge, and context.

mod client;
mod error;
mod models;

pub use client::EroldClient;
pub use error::{ApiError, Result};
pub use models::*;
