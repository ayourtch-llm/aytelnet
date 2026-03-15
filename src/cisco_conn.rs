//! CiscoConn - High-level Cisco device command executor
//!
//! This module provides a simple interface for executing single commands
//! on Cisco devices via TELNET.

#![deny(unused_must_use)]

use std::time::Duration;

use crate::cisco_telnet::CiscoTelnet;
use crate::error::TelnetError;
use tracing::{debug, info};

/// Connection type for Cisco devices
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    /// TELNET connection (default)
    CiscoTelnet,
}

/// Configuration for CiscoConn command execution
#[derive(Debug, Clone)]
pub struct CiscoConnConfig {
    /// Target device address (IPv4/IPv6)
    pub target: String,
    /// Connection type (TELNET, SSH, etc.)
    pub conntype: ConnectionType,
    /// Authentication username
    pub username: String,
    /// Authentication password
    pub password: String,
    /// Command to execute on the device
    pub cmd: String,
    /// Connection timeout
    pub timeout: Duration,
    /// Read timeout for command output
    pub read_timeout: Duration,
    /// Custom prompts to detect command completion
    pub prompts: Vec<String>,
}

impl Default for CiscoConnConfig {
    fn default() -> Self {
        Self {
            target: String::new(),
            conntype: ConnectionType::CiscoTelnet,
            username: String::new(),
            password: String::new(),
            cmd: String::new(),
            timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(30),
            prompts: vec![
                "Router#".to_string(),
                "Switch#".to_string(),
                "config#".to_string(),
                "cli#".to_string(),
            ],
        }
    }
}

/// High-level Cisco device command executor
///
/// This struct provides a simple interface for executing a single command
/// on a Cisco device and returning the output. It handles connection,
/// authentication, command execution, and disconnection automatically.
///
/// # Example
///
/// ```no_run
/// use aytelnet::cisco_conn::{CiscoConn, ConnectionType};
///
/// let conn = CiscoConn::new(
///     "192.168.1.1",
///     ConnectionType::CiscoTelnet,
///     "admin",
///     "password",
///     "show version"
/// ).unwrap();
///
/// let output = conn.execute();
/// println!("Command output: {}", output);
/// ```
pub struct CiscoConn {
    config: CiscoConnConfig,
}

impl CiscoConn {
    /// Create a new CiscoConn with default timeouts
    ///
    /// # Arguments
    ///
    /// * `target` - Device address (IPv4/IPv6, with optional port)
    /// * `conntype` - Connection type (currently only CiscoTelnet)
    /// * `username` - Authentication username
    /// * `password` - Authentication password
    /// * `cmd` - Command to execute
    ///
    /// # Returns
    ///
    /// * `Ok(CiscoConn)` - Successfully created connection
    /// * `Err(TelnetError)` - Failed to create connection
    pub fn new(
        target: &str,
        conntype: ConnectionType,
        username: &str,
        password: &str,
        cmd: &str,
    ) -> Result<Self, TelnetError> {
        Ok(Self {
            config: CiscoConnConfig {
                target: target.to_string(),
                conntype,
                username: username.to_string(),
                password: password.to_string(),
                cmd: cmd.to_string(),
                ..Default::default()
            },
        })
    }

    /// Create a new CiscoConn with custom timeouts
    ///
    /// # Arguments
    ///
    /// * `target` - Device address (IPv4/IPv6, with optional port)
    /// * `conntype` - Connection type (currently only CiscoTelnet)
    /// * `username` - Authentication username
    /// * `password` - Authentication password
    /// * `cmd` - Command to execute
    /// * `timeout` - Connection timeout
    /// * `read_timeout` - Read timeout for command output
    ///
    /// # Returns
    ///
    /// * `Ok(CiscoConn)` - Successfully created connection
    /// * `Err(TelnetError)` - Failed to create connection
    pub fn with_timeouts(
        target: &str,
        conntype: ConnectionType,
        username: &str,
        password: &str,
        cmd: &str,
        timeout: Duration,
        read_timeout: Duration,
    ) -> Result<Self, TelnetError> {
        Ok(Self {
            config: CiscoConnConfig {
                target: target.to_string(),
                conntype,
                username: username.to_string(),
                password: password.to_string(),
                cmd: cmd.to_string(),
                timeout,
                read_timeout,
                ..Default::default()
            },
        })
    }

    /// Execute the configured command and return output
    ///
    /// This method will:
    /// 1. Connect to the device
    /// 2. Authenticate
    /// 3. Execute the command
    /// 4. Capture output until prompt is detected
    /// 5. Disconnect
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Command output
    /// * `Err(TelnetError)` - Connection or execution error
    pub async fn execute(&self) -> Result<String, TelnetError> {
        debug!("Starting CiscoConn::execute for target: {}", self.config.target);
        debug!("Command: {}", self.config.cmd);

        // Determine connection type and create appropriate client
        let mut client = match self.config.conntype {
            ConnectionType::CiscoTelnet => {
                debug!("Creating CiscoTelnet client with timeout: {:?}, read_timeout: {:?}", self.config.timeout, self.config.read_timeout);
                let client = CiscoTelnet::new(&self.config.target, &self.config.username, &self.config.password);
                client.with_timeout(self.config.timeout).with_read_timeout(self.config.read_timeout)
            }
        };

        // Add custom prompts if provided
        for prompt in &self.config.prompts {
            debug!("Adding prompt: {}", prompt);
            client = client.with_prompt(prompt);
        }

        info!("Connecting to device...");
        // Connect and authenticate
        client.connect().await?;
        info!("Connected successfully");

        // Send the command with newline
        let command_with_newline = format!("{}\n", self.config.cmd);
        debug!("Sending command: {}", command_with_newline);
        client.send(command_with_newline.as_bytes()).await?;
        debug!("Command sent successfully");

        // Wait for command output until prompt is detected
        // Instead of waiting for newline, wait for privilege prompt (#)
        info!("Waiting for command output until prompt detected (timeout: {:?})", self.config.read_timeout);
        let output = client.receive_until(b"#", self.config.read_timeout).await?;
        debug!("Received output ({} bytes)", output.len());

        // Disconnect
        info!("Disconnecting from device...");
        client.disconnect().await?;
        info!("Disconnected successfully");

        debug!("Command execution completed successfully");
        Ok(output)
    }

    /// Execute the command and return output as bytes
    ///
    /// This is similar to `execute()` but returns raw bytes instead of String.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Command output as bytes
    /// * `Err(TelnetError)` - Connection or execution error
    pub async fn execute_bytes(&self) -> Result<Vec<u8>, TelnetError> {
        // Determine connection type and create appropriate client
        let mut client = match self.config.conntype {
            ConnectionType::CiscoTelnet => {
                let client = CiscoTelnet::new(&self.config.target, &self.config.username, &self.config.password);
                client.with_timeout(self.config.timeout).with_read_timeout(self.config.read_timeout)
            }
        };

        // Add custom prompts if provided
        for prompt in &self.config.prompts {
            client = client.with_prompt(prompt);
        }

        // Connect and authenticate
        client.connect().await?;

        // Send the command
        client.send(self.config.cmd.as_bytes()).await?;

        // Wait for command output until prompt is detected
        let output = client.receive_until(b"\n", self.config.read_timeout).await?;

        // Disconnect
        client.disconnect().await?;

        Ok(output.into_bytes())
    }

    /// Get the configured target address
    pub fn target(&self) -> &str {
        &self.config.target
    }

    /// Get the configured command
    pub fn cmd(&self) -> &str {
        &self.config.cmd
    }

    /// Get the configured username
    pub fn username(&self) -> &str {
        &self.config.username
    }

    /// Get the connection type
    pub fn conntype(&self) -> &ConnectionType {
        &self.config.conntype
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let conn = CiscoConn::new(
            "192.168.1.1",
            ConnectionType::CiscoTelnet,
            "admin",
            "password",
            "show version",
        ).unwrap();

        assert_eq!(conn.target(), "192.168.1.1");
        assert_eq!(conn.cmd(), "show version");
        assert_eq!(conn.username(), "admin");
        assert_eq!(conn.conntype(), &ConnectionType::CiscoTelnet);
    }

    #[test]
    fn test_new_client_with_timeouts() {
        let timeout = Duration::from_secs(60);
        let read_timeout = Duration::from_secs(20);

        let conn = CiscoConn::with_timeouts(
            "192.168.1.1:2323",
            ConnectionType::CiscoTelnet,
            "admin",
            "password",
            "show running-config",
            timeout,
            read_timeout,
        ).unwrap();

        assert_eq!(conn.target(), "192.168.1.1:2323");
        assert_eq!(conn.cmd(), "show running-config");
    }

    #[test]
    fn test_connection_type_enum() {
        let conn = CiscoConn::new(
            "router.local",
            ConnectionType::CiscoTelnet,
            "user",
            "pass",
            "help",
        ).unwrap();

        assert!(matches!(conn.conntype(), &ConnectionType::CiscoTelnet));
    }

    #[test]
    fn test_config_defaults() {
        let conn = CiscoConn::new(
            "192.168.1.1",
            ConnectionType::CiscoTelnet,
            "admin",
            "password",
            "show version",
        ).unwrap();

        assert_eq!(conn.target(), "192.168.1.1");
        assert_eq!(conn.cmd(), "show version");
    }

    #[test]
    fn test_ipv6_address() {
        let conn = CiscoConn::new(
            "[::1]:23",
            ConnectionType::CiscoTelnet,
            "admin",
            "password",
            "show version",
        ).unwrap();

        assert_eq!(conn.target(), "[::1]:23");
    }

    #[test]
    fn test_empty_command() {
        let conn = CiscoConn::new(
            "192.168.1.1",
            ConnectionType::CiscoTelnet,
            "admin",
            "password",
            "",
        ).unwrap();

        assert_eq!(conn.cmd(), "");
    }
}
