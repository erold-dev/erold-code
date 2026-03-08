//! Erold API Client
//!
//! HTTP client for interacting with the Erold API.
//! Handles projects, tasks, knowledge, context, and guidelines.

mod client;
mod error;
mod guidelines;
mod models;
mod v2;

pub use client::EroldClient;
pub use error::{ApiError, Result};
pub use guidelines::{GuidelinesClient, GuidelinesFilter, GUIDELINES_API_URL};
pub use models::*;
pub use v2::EroldV2Client;
