//! LLM integration for the Erold CLI
//!
//! Provides a client for the Claude API with streaming and tool support.
//!
//! # Example
//!
//! ```ignore
//! use erold_llm::{LlmClient, ChatSession, models::Message};
//!
//! let client = LlmClient::new("api-key")?;
//! let mut session = ChatSession::new(client)
//!     .system("You are a helpful assistant");
//!
//! let response = session.send("Hello!").await?;
//! println!("{}", response.text());
//! ```

mod error;
mod client;
pub mod models;

pub use error::{LlmError, Result};
pub use client::{LlmClient, ChatSession, StreamUpdate};
pub use models::{
    Role, Message, ContentBlock, ChatRequest, ChatResponse,
    Tool, StopReason, Usage,
};
