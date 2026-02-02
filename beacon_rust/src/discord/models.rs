//! Discord API data models
//!
//! Structures for serializing/deserializing Discord API requests and responses.

use serde::{Deserialize, Serialize};

// ============================================================================
// API Response Models
// ============================================================================

/// Discord User object
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// User's unique ID
    pub id: String,
    /// User's username
    pub username: String,
    /// User's discriminator (legacy, often "0" now)
    pub discriminator: String,
}

/// Discord Channel object
#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    /// Channel's unique ID
    pub id: String,
    /// Channel name (None for DM channels)
    pub name: Option<String>,
}

/// Discord Attachment object
#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
    /// Attachment's unique ID
    pub id: String,
    /// Filename
    pub filename: String,
    /// Size in bytes
    pub size: u64,
    /// URL to download the attachment
    pub url: String,
    /// Proxy URL
    pub proxy_url: String,
}

/// Discord Message object
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    /// Message's unique ID
    pub id: String,
    /// Message content
    pub content: String,
    /// Message author
    pub author: User,
    /// Attachments (files)
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

// ============================================================================
// API Request Models
// ============================================================================

/// Request body for creating a channel
#[derive(Debug, Serialize)]
pub struct CreateChannelRequest {
    /// Channel name
    pub name: String,
    /// Channel type (0 = text channel)
    #[serde(rename = "type")]
    pub channel_type: u8,
}

/// Request body for sending a message
#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    /// Message content
    pub content: String,
}
