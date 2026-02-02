//! Discord API module
//!
//! Handles all communication with the Discord REST API.

mod client;
mod models;

pub use client::DiscordClient;
pub use models::*;
