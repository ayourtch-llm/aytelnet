//! Cisco Telnet client with automated login.
//!
//! This module provides a high-level TELNET client specifically designed for
//! connecting to Cisco devices with automated username/password authentication.

#![deny(unused_must_use)]

//! # Example
//!
//! ```no_run
//! use aytelnet::cisco_telnet::CiscoTelnet;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = CiscoTelnet::new("192.168.1.1", "admin", "password");
//!     client.connect().await?;
//!     
//!     // Send commands
//!     client.send(b"show version\n").await?;
//!     
//!     // Receive output
//!     let output = client.receive_until(b"Router#", std::time::Duration::from_secs(30)).await?;
//!     println!("Router output: {}", output);
//!     
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```

use std::time::Duration;

use crate::connection::TelnetConnection;
use crate::error::{Result, TelnetError};
use crate::types::TelnetEvent;
use tracing::{debug, info, warn};

/// CiscoTelnet - A TELNET client for Cisco devices with automated login.
///
/// This struct provides a high-level interface for connecting to Cisco devices
/// with automatic username/password authentication. It uses a state machine
/// to handle the login process.
pub struct CiscoTelnet {
    /// Server address (host:port or just host)
    address: String,
    
    /// Username for authentication
    username: String,
    
    /// Password for authentication
    password: String,
    
    /// Underlying TELNET connection
    telnet: Option<TelnetConnection>,
    
    /// Connection timeout
    timeout: Duration,
    
    /// Read timeout for each operation
    read_timeout: Duration,
    
    /// Current state of the connection
    state: CiscoTelnetState,
    
    /// Buffer for accumulating received data
    buffer: Vec<u8>,
    
    /// Custom prompt patterns to detect login completion
    custom_prompts: Vec<String>,
}

impl std::fmt::Debug for CiscoTelnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CiscoTelnet")
            .field("address", &self.address)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("connected", &self.telnet.is_some())
            .field("timeout", &self.timeout)
            .field("read_timeout", &self.read_timeout)
            .field("state", &self.state)
            .field("buffer_len", &self.buffer.len())
            .field("custom_prompts", &self.custom_prompts)
            .finish()
    }
}

/// Connection states for the CiscoTelnet state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiscoTelnetState {
    /// Initial state, not yet connected
    Disconnected,
    
    /// Connecting to the server
    Connecting,
    
    /// Connected, waiting for login prompt
    Connected,
    
    /// Sending username
    SendingUsername,
    
    /// Sending password
    SendingPassword,
    
    /// Waiting for login to complete
    WaitingForLogin,
    
    /// Successfully logged in
    LoggedIn,
    
    /// Login failed
    LoginFailed,
    
    /// Error occurred
    Error(String),
}

impl Default for CiscoTelnetState {
    fn default() -> Self {
        CiscoTelnetState::Disconnected
    }
}

impl CiscoTelnet {
    /// Create a new CiscoTelnet client.
    ///
    /// # Arguments
    ///
    /// * `address` - IP address or hostname of the Cisco device (e.g., "192.168.1.1", "router.local", "[::1]")
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    ///
    /// # Example
    ///
    /// ```
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// let client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
    /// ```
    pub fn new(address: &str, username: &str, password: &str) -> Self {
        Self {
            address: address.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            telnet: None,
            timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(10),
            state: CiscoTelnetState::Disconnected,
            buffer: Vec::new(),
            custom_prompts: Vec::new(),
        }
    }

    /// Set the connection timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for connection
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
    ///     .with_timeout(Duration::from_secs(60));
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the read timeout for each operation.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for each read operation
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
    ///     .with_read_timeout(Duration::from_secs(5));
    /// ```
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Add a custom prompt pattern to detect login completion.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to match (supports * wildcard)
    ///
    /// # Example
    ///
    /// ```
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
    ///     .with_prompt("Router*");
    /// ```
    pub fn with_prompt(mut self, pattern: &str) -> Self {
        self.custom_prompts.push(pattern.to_string());
        self
    }

    /// Add multiple custom prompt patterns.
    pub fn with_prompts(mut self, patterns: &[&str]) -> Self {
        for pattern in patterns {
            self.custom_prompts.push(pattern.to_string());
        }
        self
    }

    /// Connect to the Cisco device and attempt to login.
    ///
    /// This method will:
    /// 1. Connect to the server
    /// 2. Negotiate TELNET options
    /// 3. Wait for login prompts
    /// 4. Send username and password
    /// 5. Wait for successful login prompt
    ///
    /// # Returns
    ///
    /// * `Ok(())` if connection and login were successful
    /// * `Err(TelnetError)` if connection or login failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
    ///     client.connect().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(&mut self) -> Result<()> {
        debug!("CiscoTelnet::connect() starting for address: {}", self.address);

        // Reset state
        self.state = CiscoTelnetState::Connecting;
        self.buffer.clear();

        // Parse address to get host and port
        let (host, port) = self.parse_address()?;
        debug!("Parsed address: {}:{}", host, port);

        // Connect to the server
        let telnet = TelnetConnection::connect(&host, port).await?;
        self.telnet = Some(telnet);
        self.state = CiscoTelnetState::Connected;
        debug!("Connection established");

        // Negotiate options
        if let Err(e) = self.negotiate_options().await {
            self.cleanup_on_error(&e.to_string()).await;
            return Err(e);
        }

        // Authenticate
        if let Err(e) = self.authenticate().await {
            self.cleanup_on_error(&e.to_string()).await;
            return Err(e);
        }

        // Send "term len 0" to disable paging
        debug!("Sending 'term len 0'");
        if let Err(e) = self.send(b"term len 0\n").await {
            self.cleanup_on_error(&e.to_string()).await;
            return Err(e);
        }

        // Wait for prompt after term len 0, using confirm_prompt for reliability
        self.buffer.clear();
        if let Err(_) = self.wait_for_confirmed_prompt(Duration::from_secs(5)).await {
            warn!("Timeout waiting for 'term len 0' response, continuing anyway");
        }

        // Clear buffer for a clean slate
        self.buffer.clear();

        info!("Connected and authenticated to {}", self.address);
        Ok(())
    }

    /// Clean up connection state after an error during connect.
    async fn cleanup_on_error(&mut self, error: &str) {
        if let Some(ref mut telnet) = self.telnet {
            let _ = telnet.disconnect().await;
        }
        self.telnet = None;
        if !matches!(self.state, CiscoTelnetState::LoginFailed | CiscoTelnetState::Error(_)) {
            self.state = CiscoTelnetState::Error(error.to_string());
        }
    }

    /// Parse the address into host and port.
    ///
    /// Supports both IPv4 and IPv6 addresses.
    /// If no port is specified, defaults to 23.
    ///
    /// # Examples
    ///
    /// * "192.168.1.1" -> ("192.168.1.1", 23)
    /// * "192.168.1.1:23" -> ("192.168.1.1", 23)
    /// * "[::1]" -> ("::1", 23)
    /// * "[::1]:23" -> ("::1", 23)
    fn parse_address(&self) -> Result<(String, u16)> {
        let addr = &self.address;
        
        // Check for IPv6 address in brackets
        if addr.starts_with('[') {
            if let Some(end_bracket) = addr.find(']') {
                let host = addr[1..end_bracket].to_string();
                let port = if addr.len() > end_bracket + 1 && addr[end_bracket + 1..].starts_with(':') {
                    addr[end_bracket + 2..].parse().unwrap_or(23)
                } else {
                    23
                };
                return Ok((host, port));
            }
        }
        
        // Check for IPv4 with port (last colon)
        if let Some(colon_pos) = addr.rfind(':') {
            let host = addr[..colon_pos].to_string();
            let port = addr[colon_pos + 1..].parse().unwrap_or(23);
            return Ok((host, port));
        }
        
        // No port specified, default to 23
        Ok((addr.to_string(), 23))
    }

    /// Negotiate TELNET options.
    async fn negotiate_options(&mut self) -> Result<()> {
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        
        debug!("Enabling TELNET options for compatibility...");
        debug!("  - Disabling ECHO");
        telnet.negotiate_option(crate::protocol::OPT_ECHO, false).await?;
        
        debug!("  - Enabling BINARY mode");
        telnet.negotiate_option(crate::protocol::OPT_BINARY, true).await?;
        
        debug!("  - Enabling SUPPRESS_GA");
        telnet.negotiate_option(crate::protocol::OPT_SUPPRESS_GA, true).await?;
        
        debug!("TELNET options negotiation completed");
        Ok(())
    }

    /// Authenticate with the Cisco device.
    ///
    /// Handles three scenarios:
    /// - Device requires username + password
    /// - Device requires password only (no username prompt)
    /// - Device needs no authentication (direct CLI prompt)
    async fn authenticate(&mut self) -> Result<()> {
        info!("Starting authentication for user: {}", self.username);

        self.wait_for_login_prompt().await?;

        match self.state {
            CiscoTelnetState::LoggedIn => {
                info!("Device does not require authentication");
                return Ok(());
            }
            CiscoTelnetState::SendingPassword => {
                debug!("Device went straight to password prompt, skipping username");
            }
            CiscoTelnetState::SendingUsername => {
                self.send_username().await?;
                self.wait_for_password_prompt().await?;
            }
            ref state => {
                warn!("Unexpected state after wait_for_login_prompt: {:?}", state);
            }
        }

        self.send_password_and_wait().await?;

        info!("Authentication completed successfully");
        Ok(())
    }

    /// Wait for a login prompt, password prompt, or device CLI prompt.
    ///
    /// Checks all patterns after each receive (not sequentially), and also
    /// detects if the device goes straight to password or needs no auth.
    async fn wait_for_login_prompt(&mut self) -> Result<()> {
        let deadline = tokio::time::Instant::now() + self.read_timeout;

        debug!("wait_for_login_prompt() starting");

        loop {
            // Check buffer for login prompts (case-insensitive)
            let buffer_lower = String::from_utf8_lossy(&self.buffer).to_lowercase();

            if buffer_lower.contains("username:") || buffer_lower.contains("login:")
                || buffer_lower.contains("user:") || buffer_lower.contains("name:")
            {
                debug!("Login prompt detected in buffer");
                self.state = CiscoTelnetState::SendingUsername;
                return Ok(());
            }

            // Some devices skip username and go straight to password
            if buffer_lower.contains("password:") {
                debug!("Password prompt detected without username prompt");
                self.state = CiscoTelnetState::SendingPassword;
                return Ok(());
            }

            // Device might not require authentication — check for a CLI prompt
            if Self::buffer_ends_with_prompt(&self.buffer) {
                info!("Device prompt detected without login — no authentication required");
                self.state = CiscoTelnetState::LoggedIn;
                return Ok(());
            }

            match tokio::time::timeout_at(deadline, async {
                match self.telnet.as_mut() {
                    Some(conn) => conn.receive().await,
                    None => Err(TelnetError::Disconnected),
                }
            })
            .await
            {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    self.buffer.extend_from_slice(&data);
                    debug!("Received {} bytes, buffer: {} bytes", data.len(), self.buffer.len());
                }
                Ok(Ok(TelnetEvent::Closed)) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(Ok(TelnetEvent::Error(e))) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    warn!("Timeout waiting for login prompt");
                    if !self.buffer.is_empty() {
                        warn!(
                            "Buffer contents: {}",
                            String::from_utf8_lossy(&self.buffer)
                        );
                    }
                    return Err(TelnetError::Timeout);
                }
            }
        }
    }

    /// Send username to the device (does not wait for password prompt).
    async fn send_username(&mut self) -> Result<()> {
        let mut send_data = self.username.as_bytes().to_vec();
        send_data.push(b'\n');

        debug!("Sending username: {}", self.username);
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(&send_data).await?;
        debug!("Username sent successfully");
        Ok(())
    }

    /// Wait for password prompt with proper deadline-based timeout.
    async fn wait_for_password_prompt(&mut self) -> Result<()> {
        let deadline = tokio::time::Instant::now() + self.read_timeout;

        debug!("Waiting for password prompt");

        loop {
            let buffer_lower = String::from_utf8_lossy(&self.buffer).to_lowercase();

            if buffer_lower.contains("password:") {
                debug!("Password prompt found");
                self.state = CiscoTelnetState::SendingPassword;
                return Ok(());
            }

            // Check for early auth failure
            if buffer_lower.contains("authentication failed")
                || buffer_lower.contains("access denied")
                || buffer_lower.contains("% login invalid")
                || buffer_lower.contains("% bad")
            {
                self.state = CiscoTelnetState::LoginFailed;
                return Err(TelnetError::Protocol("Authentication failed".to_string()));
            }

            match tokio::time::timeout_at(deadline, async {
                match self.telnet.as_mut() {
                    Some(conn) => conn.receive().await,
                    None => Err(TelnetError::Disconnected),
                }
            })
            .await
            {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    self.buffer.extend_from_slice(&data);
                    debug!("Received {} bytes, buffer: {} bytes", data.len(), self.buffer.len());
                }
                Ok(Ok(TelnetEvent::Closed)) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(Ok(TelnetEvent::Error(e))) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(TelnetError::Timeout),
            }
        }
    }

    /// Send password and wait for a confirmed CLI prompt.
    ///
    /// After sending the password, waits for a proper CLI prompt (not just
    /// a bare `#`). Detects authentication failure messages on timeout.
    async fn send_password_and_wait(&mut self) -> Result<()> {
        let mut send_data = self.password.as_bytes().to_vec();
        send_data.push(b'\n');

        info!("Sending password (length: {})", self.password.len());
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(&send_data).await?;
        debug!("Password sent");

        // Clear buffer so we only inspect post-password output
        self.buffer.clear();

        match self.wait_for_confirmed_prompt(self.read_timeout).await {
            Ok(()) => {
                info!("Login prompt confirmed, authentication complete");
                self.state = CiscoTelnetState::LoggedIn;
                Ok(())
            }
            Err(TelnetError::Timeout) => {
                // Distinguish auth failure from generic timeout
                let buffer_lower = String::from_utf8_lossy(&self.buffer).to_lowercase();
                self.state = CiscoTelnetState::LoginFailed;
                if buffer_lower.contains("authentication failed")
                    || buffer_lower.contains("access denied")
                    || buffer_lower.contains("% login invalid")
                    || buffer_lower.contains("% bad")
                    || buffer_lower.contains("% authentication")
                {
                    Err(TelnetError::Protocol("Authentication failed".to_string()))
                } else {
                    Err(TelnetError::Timeout)
                }
            }
            Err(e) => {
                self.state = CiscoTelnetState::Error(e.to_string());
                Err(e)
            }
        }
    }

    /// Heuristic check: does the buffer end with what looks like a Cisco CLI prompt?
    ///
    /// A Cisco prompt sits at the end of output with no trailing newline:
    ///   `Router#`  `Switch>`  `R1(config-if)#`
    ///
    /// Banners like `##########` or `>> Welcome >>` are rejected because they
    /// either appear as full lines (trailing `\r\n`) or lack alphanumeric
    /// content before the `#`/`>` suffix.
    fn buffer_ends_with_prompt(buffer: &[u8]) -> bool {
        if buffer.is_empty() {
            return false;
        }

        // Trim trailing spaces/tabs/nulls (some devices pad the prompt)
        let end = buffer
            .iter()
            .rposition(|b| *b != b' ' && *b != b'\t' && *b != 0)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        if end == 0 {
            return false;
        }

        let buffer = &buffer[..end];

        // Prompt must end with '#' or '>'
        let last = buffer[buffer.len() - 1];
        if last != b'#' && last != b'>' {
            return false;
        }

        // Find the start of the last line
        let last_line_start = buffer[..buffer.len() - 1]
            .iter()
            .rposition(|&b| b == b'\n' || b == b'\r')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        let last_line = &buffer[last_line_start..];

        // Reject unreasonably long "prompts"
        if last_line.len() > 80 {
            return false;
        }

        // The text before '#'/'>' must contain at least one alphanumeric char
        // (this rejects pure-punctuation banners like "######" or ">>>>>>")
        let before_suffix = &last_line[..last_line.len() - 1];
        before_suffix.iter().any(|b| b.is_ascii_alphanumeric())
    }

    /// Wait for a candidate prompt, then confirm it by sending `\n` and
    /// checking that the same prompt-like line reappears.
    ///
    /// This is the most reliable prompt detection: banners containing `#` or
    /// `>` will not re-echo when you press enter, but a real CLI prompt will.
    async fn wait_for_confirmed_prompt(&mut self, timeout: Duration) -> Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;

        // Phase 1: wait for the heuristic to fire
        self.wait_for_prompt_heuristic(deadline).await?;

        // Extract the candidate prompt (last line of buffer)
        let candidate = Self::extract_last_line(&self.buffer);
        if candidate.is_empty() {
            return Err(TelnetError::Timeout);
        }
        debug!("Candidate prompt: {:?}", String::from_utf8_lossy(&candidate));

        // Phase 2: send \n and see if we get the same prompt back
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(b"\n").await?;
        self.buffer.clear();

        // Wait for the prompt to reappear
        loop {
            match tokio::time::timeout_at(deadline, async {
                match self.telnet.as_mut() {
                    Some(conn) => conn.receive().await,
                    None => Err(TelnetError::Disconnected),
                }
            })
            .await
            {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    self.buffer.extend_from_slice(&data);
                    if Self::buffer_ends_with_prompt(&self.buffer) {
                        let new_prompt = Self::extract_last_line(&self.buffer);
                        if new_prompt == candidate {
                            debug!("Prompt confirmed: {:?}", String::from_utf8_lossy(&candidate));
                            return Ok(());
                        }
                        // Different prompt-like line — could be more banner.
                        // Use it as the new candidate and probe again if we have time.
                        debug!(
                            "Different prompt after enter: {:?} (expected {:?}), re-probing",
                            String::from_utf8_lossy(&new_prompt),
                            String::from_utf8_lossy(&candidate)
                        );
                        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
                        telnet.send(b"\n").await?;
                        let candidate_inner = new_prompt;
                        self.buffer.clear();

                        // One more round
                        loop {
                            match tokio::time::timeout_at(deadline, async {
                                match self.telnet.as_mut() {
                                    Some(conn) => conn.receive().await,
                                    None => Err(TelnetError::Disconnected),
                                }
                            })
                            .await
                            {
                                Ok(Ok(TelnetEvent::Data(data2))) => {
                                    self.buffer.extend_from_slice(&data2);
                                    if Self::buffer_ends_with_prompt(&self.buffer) {
                                        let final_prompt = Self::extract_last_line(&self.buffer);
                                        if final_prompt == candidate_inner {
                                            debug!("Prompt confirmed on retry");
                                            return Ok(());
                                        }
                                        // Still different — accept it as best-effort
                                        debug!("Accepting prompt after second probe");
                                        return Ok(());
                                    }
                                }
                                Ok(Ok(TelnetEvent::Closed)) => {
                                    return Err(TelnetError::Disconnected);
                                }
                                Ok(Ok(TelnetEvent::Error(e))) => return Err(e),
                                Ok(Ok(_)) => continue,
                                Ok(Err(e)) => return Err(e),
                                Err(_) => return Err(TelnetError::Timeout),
                            }
                        }
                    }
                }
                Ok(Ok(TelnetEvent::Closed)) => return Err(TelnetError::Disconnected),
                Ok(Ok(TelnetEvent::Error(e))) => return Err(e),
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(TelnetError::Timeout),
            }
        }
    }

    /// Wait until `buffer_ends_with_prompt` fires (heuristic only, no confirmation).
    async fn wait_for_prompt_heuristic(
        &mut self,
        deadline: tokio::time::Instant,
    ) -> Result<()> {
        // Check buffer first
        if Self::buffer_ends_with_prompt(&self.buffer) {
            return Ok(());
        }

        loop {
            match tokio::time::timeout_at(deadline, async {
                match self.telnet.as_mut() {
                    Some(conn) => conn.receive().await,
                    None => Err(TelnetError::Disconnected),
                }
            })
            .await
            {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    self.buffer.extend_from_slice(&data);
                    debug!(
                        "wait_for_prompt: received {} bytes, buffer: {} bytes",
                        data.len(),
                        self.buffer.len()
                    );
                    if Self::buffer_ends_with_prompt(&self.buffer) {
                        return Ok(());
                    }
                }
                Ok(Ok(TelnetEvent::Closed)) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(Ok(TelnetEvent::Error(e))) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(TelnetError::Timeout),
            }
        }
    }

    /// Extract the last line from the buffer (the candidate prompt text).
    fn extract_last_line(buffer: &[u8]) -> Vec<u8> {
        // Trim trailing spaces/tabs/nulls
        let end = buffer
            .iter()
            .rposition(|b| *b != b' ' && *b != b'\t' && *b != 0)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        if end == 0 {
            return Vec::new();
        }

        let trimmed = &buffer[..end];
        let start = trimmed
            .iter()
            .rposition(|&b| b == b'\n' || b == b'\r')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        trimmed[start..].to_vec()
    }

    /// Helper function to check if buffer contains bytes.
    fn buffer_contains(buffer: &[u8], bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }
        if bytes.len() > buffer.len() {
            return false;
        }
        
        for i in 0..=(buffer.len() - bytes.len()) {
            if &buffer[i..i + bytes.len()] == bytes {
                return true;
            }
        }
        false
    }

    /// Helper function to check if buffer ends with bytes.
    fn buffer_ends_with(buffer: &[u8], suffix: &[u8]) -> bool {
        if suffix.is_empty() {
            return true;
        }
        if suffix.len() > buffer.len() {
            return false;
        }
        &buffer[buffer.len() - suffix.len()..] == suffix
    }

    /// Send data to the device.
    ///
    /// # Arguments
    ///
    /// * `data` - Bytes to send
    ///
    /// # Example
    ///
    /// ```
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
    ///     client.connect().await?;
    ///     client.send(b"show version\n").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        if self.state != CiscoTelnetState::LoggedIn {
            debug!("Cannot send: not logged in, state: {:?}", self.state);
            return Err(TelnetError::InvalidState(
                "Not logged in".to_string(),
            ));
        }
        
        debug!("Sending {} bytes", data.len());
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(data).await?;
        debug!("Send completed successfully");
        Ok(())
    }

    /// Receive data until a specific pattern is found.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Bytes to wait for (e.g., b"Router#")
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// The received data including the pattern
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
    ///     client.connect().await?;
    ///     client.send(b"show version\n").await?;
    ///     let output = client.receive_until(b"Router#", std::time::Duration::from_secs(30)).await?;
    ///     println!("{}", output);
    ///     Ok(())
    /// }
    /// ```
    pub async fn receive_until(&mut self, pattern: &[u8], timeout: Duration) -> Result<String> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut output = String::new();

        info!(
            "receive_until() waiting for pattern {:?} with timeout {:?}",
            String::from_utf8_lossy(pattern),
            timeout
        );

        loop {
            match tokio::time::timeout_at(deadline, async {
                match self.telnet.as_mut() {
                    Some(conn) => conn.receive().await,
                    None => Err(TelnetError::Disconnected),
                }
            })
            .await
            {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    let text = String::from_utf8_lossy(&data);
                    output.push_str(&text);
                    self.buffer.extend_from_slice(&data);

                    debug!("Received {} bytes, total output: {} bytes", data.len(), output.len());

                    if output.contains(&String::from_utf8_lossy(pattern).as_ref()) {
                        info!("Pattern found after {} bytes", output.len());
                        return Ok(output);
                    }
                }
                Ok(Ok(TelnetEvent::Closed)) => return Err(TelnetError::Disconnected),
                Ok(Ok(TelnetEvent::Error(e))) => return Err(e),
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    warn!(
                        "Timeout waiting for pattern: {:?}",
                        String::from_utf8_lossy(pattern)
                    );
                    return Err(TelnetError::Timeout);
                }
            }
        }
    }

    /// Receive raw data bytes from the connection, stripping TELNET protocol (IAC sequences).
    ///
    /// Semantics:
    /// - If data is already buffered or immediately available, return it RIGHT AWAY
    ///   (do NOT wait for more data or for the timeout to expire)
    /// - Only block up to `timeout` if there is NO data available yet
    /// - Returns an empty Vec if the timeout expires with no data
    /// - This means: first chunk arrives fast, caller can call again for more
    ///
    /// This enables the caller to do fast-paced incremental pattern matching
    /// without being blocked waiting for a full buffer or timeout.
    ///
    /// This is a low-level method — no prompt detection or delimiter matching.
    pub async fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        let conn = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match tokio::time::timeout_at(deadline, conn.receive()).await {
                Ok(Ok(TelnetEvent::Data(bytes))) => return Ok(bytes.to_vec()),
                Ok(Ok(TelnetEvent::Closed)) => return Err(TelnetError::Disconnected),
                Ok(Ok(_)) => continue, // protocol event, try again within same deadline
                Ok(Err(e)) => return Err(e),
                Err(_) => return Ok(vec![]), // timeout, no data
            }
        }
    }

    /// Receive a single line of output.
    ///
    /// # Returns
    ///
    /// The received line without the newline character
    ///
    /// # Example
    ///
    /// ```
    /// use aytelnet::cisco_telnet::CiscoTelnet;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
    ///     client.connect().await?;
    ///     client.send(b"show version\n").await?;
    ///     let line = client.receive_line().await?;
    ///     println!("{}", line);
    ///     Ok(())
    /// }
    /// ```
    pub async fn receive_line(&mut self) -> Result<String> {
        let mut output = String::new();
        
        loop {
            match self.telnet.as_mut().ok_or(TelnetError::Disconnected)?.receive().await {
                Ok(TelnetEvent::Data(data)) => {
                    for byte in &data {
                        if *byte == b'\n' || *byte == b'\r' {
                            return Ok(output);
                        }
                        output.push(*byte as char);
                    }
                }
                Ok(TelnetEvent::Closed) => {
                    return Err(TelnetError::Disconnected);
                }
                Ok(TelnetEvent::Error(e)) => {
                    return Err(e);
                }
                _ => {}
            }
        }
    }

    /// Disconnect from the device.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut telnet) = self.telnet {
            let _ = telnet.disconnect().await;
        }
        self.telnet = None;
        self.state = CiscoTelnetState::Disconnected;
        self.buffer.clear();
        Ok(())
    }

    /// Check if connected and logged in.
    pub fn is_logged_in(&self) -> bool {
        self.state == CiscoTelnetState::LoggedIn
    }

    /// Get the current connection state.
    pub fn state(&self) -> &CiscoTelnetState {
        &self.state
    }

    /// Get a reference to the underlying TELNET connection.
    pub fn telnet(&self) -> Option<&TelnetConnection> {
        self.telnet.as_ref()
    }

    /// Get a mutable reference to the underlying TELNET connection.
    pub fn telnet_mut(&mut self) -> Option<&mut TelnetConnection> {
        self.telnet.as_mut()
    }
}

impl Default for CiscoTelnet {
    fn default() -> Self {
        Self::new("", "", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_client() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
        assert_eq!(client.state(), &CiscoTelnetState::Disconnected);
        assert_eq!(client.username, "admin");
        assert_eq!(client.password, "secret");
    }

    #[tokio::test]
    async fn test_with_timeout() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
            .with_timeout(Duration::from_secs(60));
        assert_eq!(client.timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_with_read_timeout() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
            .with_read_timeout(Duration::from_secs(5));
        assert_eq!(client.read_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_with_prompt() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
            .with_prompt("Router*");
        assert_eq!(client.custom_prompts.len(), 1);
        assert_eq!(client.custom_prompts[0], "Router*");
    }

    #[tokio::test]
    async fn test_with_prompts() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret")
            .with_prompts(&["Router*", "Switch*"]);
        assert_eq!(client.custom_prompts.len(), 2);
    }

    #[tokio::test]
    async fn test_is_logged_in() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
        assert!(!client.is_logged_in());
    }

    #[test]
    fn test_parse_address_no_port() {
        let client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
        let (host, port) = client.parse_address().unwrap();
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 23);
    }

    #[test]
    fn test_parse_address_with_port() {
        let client = CiscoTelnet::new("192.168.1.1:2323", "admin", "secret");
        let (host, port) = client.parse_address().unwrap();
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 2323);
    }

    #[test]
    fn test_parse_address_ipv6_no_port() {
        let client = CiscoTelnet::new("[::1]", "admin", "secret");
        let (host, port) = client.parse_address().unwrap();
        assert_eq!(host, "::1");
        assert_eq!(port, 23);
    }

    #[test]
    fn test_parse_address_ipv6_with_port() {
        let client = CiscoTelnet::new("[::1]:2323", "admin", "secret");
        let (host, port) = client.parse_address().unwrap();
        assert_eq!(host, "::1");
        assert_eq!(port, 2323);
    }

    #[test]
    fn test_buffer_contains() {
        let buffer = vec![1, 2, 3, 4, 5];
        assert!(CiscoTelnet::buffer_contains(&buffer, &[2, 3]));
        assert!(!CiscoTelnet::buffer_contains(&buffer, &[6, 7]));
    }

    #[test]
    fn test_buffer_ends_with() {
        let buffer = vec![1, 2, 3, 4, 5];
        assert!(CiscoTelnet::buffer_ends_with(&buffer, &[4, 5]));
        assert!(!CiscoTelnet::buffer_ends_with(&buffer, &[1, 2]));
    }

    #[test]
    fn test_buffer_ends_with_prompt_privileged() {
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Router#"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Switch#"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Router(config)#"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"R1(config-if)#"));
    }

    #[test]
    fn test_buffer_ends_with_prompt_unprivileged() {
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Router>"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Switch>"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"R1>"));
    }

    #[test]
    fn test_buffer_ends_with_prompt_after_banner() {
        // Banner lines followed by a real prompt
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"##########\r\nRouter#"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b">> Welcome >>\r\nSwitch>"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(
            b"***  WARNING: Authorized access only  ***\r\n##########\r\nRouter#"
        ));
    }

    #[test]
    fn test_buffer_ends_with_prompt_rejects_banner() {
        // Banner lines ending with # or > — NOT real prompts
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"##########\r\n"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"##########"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b">>>>>>>"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"## Welcome ##\r\n"));
        // Pure punctuation on the last line
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"####"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b">>>>"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"---#"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"...>"));
    }

    #[test]
    fn test_buffer_ends_with_prompt_edge_cases() {
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b""));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"#"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b">"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"no prompt here"));
        // Single-letter hostname
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"R#"));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"R>"));
    }

    #[test]
    fn test_buffer_ends_with_prompt_trailing_space() {
        // Some devices send trailing spaces after the prompt
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Router# "));
        assert!(CiscoTelnet::buffer_ends_with_prompt(b"Switch> \t"));
    }

    #[test]
    fn test_buffer_ends_with_prompt_trailing_newline_rejected() {
        // A trailing newline means output is still in progress — not a prompt
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"Router#\r\n"));
        assert!(!CiscoTelnet::buffer_ends_with_prompt(b"Switch>\n"));
    }

    #[test]
    fn test_extract_last_line() {
        assert_eq!(
            CiscoTelnet::extract_last_line(b"Router#"),
            b"Router#"
        );
        assert_eq!(
            CiscoTelnet::extract_last_line(b"banner\r\nRouter#"),
            b"Router#"
        );
        assert_eq!(
            CiscoTelnet::extract_last_line(b"banner\r\nRouter# "),
            b"Router#"
        );
        assert_eq!(
            CiscoTelnet::extract_last_line(b""),
            Vec::<u8>::new()
        );
    }

    #[tokio::test]
    async fn test_receive_on_disconnected_client() {
        let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
        let result = client.receive(Duration::from_millis(100)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TelnetError::Disconnected));
    }

    #[tokio::test]
    async fn test_receive_signature_compiles() {
        // Verify that receive() accepts Duration and returns Result<Vec<u8>>
        let mut client = CiscoTelnet::new("192.168.1.1", "admin", "secret");
        let result: std::result::Result<Vec<u8>, TelnetError> =
            client.receive(Duration::from_millis(1)).await;
        // Should fail because not connected, but the types must match
        assert!(result.is_err());
    }
}
