//! Cisco Telnet client with automated login.
//!
//! This module provides a high-level TELNET client specifically designed for
//! connecting to Cisco devices with automated username/password authentication.
//!
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
        // Reset state
        self.state = CiscoTelnetState::Connecting;
        self.buffer.clear();
        
        // Parse address to get host and port
        let (host, port) = self.parse_address()?;
        
        // Connect to the server
        let telnet = TelnetConnection::connect(&host, port).await?;
        self.telnet = Some(telnet);
        self.state = CiscoTelnetState::Connected;
        
        // Negotiate options
        self.negotiate_options().await?;
        
        // Wait for login prompts and authenticate
        self.authenticate().await?;
        
        Ok(())
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
        
        // Enable common options for better compatibility
        telnet.negotiate_option(crate::protocol::OPT_ECHO, false).await?;
        telnet.negotiate_option(crate::protocol::OPT_BINARY, true).await?;
        telnet.negotiate_option(crate::protocol::OPT_SUPPRESS_GA, true).await?;
        
        Ok(())
    }

    /// Authenticate with the Cisco device.
    async fn authenticate(&mut self) -> Result<()> {
        // Wait for login prompt
        self.wait_for_login_prompt().await?;
        
        // Send username
        self.send_username().await?;
        
        // Wait for password prompt
        self.wait_for_password_prompt().await?;
        
        // Send password
        self.send_password().await?;
        
        // Wait for login to complete
        self.wait_for_login_complete().await?;
        
        Ok(())
    }

    /// Wait for login prompt.
    async fn wait_for_login_prompt(&mut self) -> Result<()> {
        // Common login prompt patterns
        let prompts = [
            b"Username:".as_slice(),
            b"login:".as_slice(),
            b"user:".as_slice(),
            b"name:".as_slice(),
        ];
        
        for prompt in &prompts {
            if self.wait_for_bytes(prompt, self.read_timeout).await.is_ok() {
                self.state = CiscoTelnetState::SendingUsername;
                return Ok(());
            }
        }
        
        // If none matched, we might already be logged in or need to try different prompts
        Ok(())
    }

    /// Send username to the device.
    async fn send_username(&mut self) -> Result<()> {
        let mut send_data = self.username.as_bytes().to_vec();
        send_data.push(b'\n');
        
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(&send_data).await?;
        
        // Wait for response (consume any response)
        self.wait_for_bytes(b"Password:".as_slice(), self.read_timeout).await?;
        
        self.state = CiscoTelnetState::SendingPassword;
        Ok(())
    }

    /// Wait for password prompt.
    async fn wait_for_password_prompt(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > self.read_timeout {
                return Err(TelnetError::Timeout);
            }
            
            match self.telnet.as_mut().ok_or(TelnetError::Disconnected)?.receive().await {
                Ok(TelnetEvent::Data(data)) => {
                    self.buffer.extend_from_slice(&data);
                    
                    // Check for password prompt
                    let buffer_str = String::from_utf8_lossy(&self.buffer);
                    if buffer_str.contains("Password:") {
                        self.state = CiscoTelnetState::SendingPassword;
                        return Ok(());
                    }
                    
                    // Check for error conditions
                    if buffer_str.contains("Authentication failed") ||
                       buffer_str.contains("Access denied") ||
                       buffer_str.contains("Authentication fail") {
                        self.state = CiscoTelnetState::LoginFailed;
                        return Err(TelnetError::Protocol("Authentication failed".to_string()));
                    }
                }
                Ok(TelnetEvent::Closed) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(TelnetEvent::Error(e)) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                _ => {}
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Send password to the device.
    async fn send_password(&mut self) -> Result<()> {
        let mut send_data = self.password.as_bytes().to_vec();
        send_data.push(b'\n');
        
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(&send_data).await?;
        
        // Wait for response (consume any response)
        self.wait_for_bytes(b"Router#".as_slice(), self.read_timeout).await?;
        
        Ok(())
    }

    /// Send data and wait for a specific prompt.
    async fn send_with_prompt(&mut self, data: &str, prompt: &[u8]) -> Result<()> {
        // Send data with newline
        let mut send_data = data.as_bytes().to_vec();
        send_data.push(b'\n');
        
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(&send_data).await?;
        
        // Wait for the prompt (consume any response)
        self.wait_for_bytes(prompt, self.read_timeout).await?;
        
        Ok(())
    }

    /// Wait for bytes with timeout.
    async fn wait_for_bytes(&mut self, bytes: &[u8], timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        
        loop {
            // Check if we've timed out
            if start.elapsed() > timeout {
                return Err(TelnetError::Timeout);
            }
            
            // Try to receive data
            match tokio::time::timeout(self.read_timeout, async {
                if let Some(ref mut telnet) = self.telnet {
                    telnet.receive().await
                } else {
                    Err(TelnetError::Disconnected)
                }
            }).await {
                Ok(Ok(TelnetEvent::Data(data))) => {
                    self.buffer.extend_from_slice(&data);
                    
                    // Check if we found the prompt
                    if bytes.is_empty() || Self::buffer_contains(&self.buffer, bytes) {
                        return Ok(());
                    }
                    
                    // Check for error conditions
                    let buffer_str = String::from_utf8_lossy(&self.buffer);
                    if buffer_str.contains("Authentication failed") ||
                       buffer_str.contains("Access denied") ||
                       buffer_str.contains("Authentication fail") {
                        return Err(TelnetError::Protocol("Authentication failed".to_string()));
                    }
                }
                Ok(Ok(TelnetEvent::Command(cmd))) => {
                    // Handle TELNET commands
                    eprintln!("Received command: {:?}", cmd);
                }
                Ok(Ok(TelnetEvent::OptionNegotiated { .. })) => {
                    // Option negotiated, continue waiting
                }
                Ok(Ok(TelnetEvent::Closed)) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(Ok(TelnetEvent::Error(e))) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                Ok(Err(e)) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                Err(_) => {
                    // Timeout waiting for data
                    continue;
                }
            }
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Wait for login to complete.
    async fn wait_for_login_complete(&mut self) -> Result<()> {
        // Common prompt patterns for Cisco devices
        let common_prompts: Vec<Vec<u8>> = vec![
            b">".to_vec(),  // User mode
            b"#".to_vec(),  // Privileged mode
            b"Router>".to_vec(),
            b"Router#".to_vec(),
            b"Switch>".to_vec(),
            b"Switch#".to_vec(),
            b"ASA>".to_vec(),
            b"ASA#".to_vec(),
            b"config".to_vec(),
        ];
        
        // Add custom prompts (convert patterns to simple suffix checks)
        let mut all_prompts = common_prompts;
        for pattern in &self.custom_prompts {
            if pattern.ends_with('*') {
                // Prefix pattern - check if buffer ends with prefix
                let prefix = pattern[..pattern.len() - 1].as_bytes().to_vec();
                all_prompts.push(prefix);
            } else {
                // Exact match
                all_prompts.push(pattern.as_bytes().to_vec());
            }
        }
        
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > self.timeout {
                return Err(TelnetError::Timeout);
            }
            
            match self.telnet.as_mut().ok_or(TelnetError::Disconnected)?.receive().await {
                Ok(TelnetEvent::Data(data)) => {
                    self.buffer.extend_from_slice(&data);
                    
                    // Check if we've received a prompt
                    for prompt in &all_prompts {
                        if Self::buffer_ends_with(&self.buffer, prompt) {
                            self.state = CiscoTelnetState::LoggedIn;
                            return Ok(());
                        }
                    }
                    
                    // Check for error conditions
                    let buffer_str = String::from_utf8_lossy(&self.buffer);
                    if buffer_str.contains("Authentication failed") ||
                       buffer_str.contains("Access denied") ||
                       buffer_str.contains("Authentication fail") {
                        self.state = CiscoTelnetState::LoginFailed;
                        return Err(TelnetError::Protocol("Authentication failed".to_string()));
                    }
                }
                Ok(TelnetEvent::Closed) => {
                    self.state = CiscoTelnetState::Error("Connection closed".to_string());
                    return Err(TelnetError::Disconnected);
                }
                Ok(TelnetEvent::Error(e)) => {
                    self.state = CiscoTelnetState::Error(e.to_string());
                    return Err(e);
                }
                _ => {}
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
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
            return Err(TelnetError::InvalidState(
                "Not logged in".to_string(),
            ));
        }
        
        let telnet = self.telnet.as_mut().ok_or(TelnetError::Disconnected)?;
        telnet.send(data).await
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
        let start = std::time::Instant::now();
        let mut output = String::new();
        
        loop {
            if start.elapsed() > timeout {
                return Err(TelnetError::Timeout);
            }
            
            match self.telnet.as_mut().ok_or(TelnetError::Disconnected)?.receive().await {
                Ok(TelnetEvent::Data(data)) => {
                    // Convert to string, replacing invalid UTF-8
                    let text = String::from_utf8_lossy(&data);
                    output.push_str(&text);
                    
                    // Check if we've found the pattern
                    if output.contains(&String::from_utf8_lossy(pattern).as_ref()) {
                        break;
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
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Ok(output)
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
            telnet.disconnect().await?;
        }
        self.state = CiscoTelnetState::Disconnected;
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
}
