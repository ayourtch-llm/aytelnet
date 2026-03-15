//! TELNET connection manager.
//!
//! This module provides a simple TELNET connection with explicit state machine handling.

#![deny(unused_must_use)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::decoder::TelnetDecoder;
use crate::encoder::TelnetEncoder;
use crate::error::TelnetError;
use crate::error::Result;
use crate::options::OptionNegotiator;
use crate::state::StateManager;
use crate::types::{TelnetCommand, TelnetEvent};
use tracing::debug;
use tracing::info;

/// TELNET connection with explicit state machine handling.
///
/// This connection handles I/O directly without separate tasks,
/// making the state machine explicit and easier to reason about.
pub struct TelnetConnection {
    /// TCP stream
    stream: Option<TcpStream>,
    
    /// State manager
    state_manager: StateManager,
    
    /// Option negotiator
    option_negotiator: OptionNegotiator,
    
    /// Data encoder
    encoder: TelnetEncoder,
    
    /// Data decoder (preserves state across reads)
    decoder: TelnetDecoder,
}

impl TelnetConnection {
    /// Create a new connection.
    pub fn new() -> Self {
        Self {
            stream: None,
            state_manager: StateManager::new(),
            option_negotiator: OptionNegotiator::new(),
            encoder: TelnetEncoder::new(),
            decoder: TelnetDecoder::new(),
        }
    }

    /// Connect to a TELNET server.
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        info!("TcpStream::connect({}:{}) starting...", host, port);
        let stream = TcpStream::connect((host, port)).await?;
        debug!("TcpStream::connect({}:{}) completed", host, port);
        
        let mut conn = Self::new();
        conn.stream = Some(stream);
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        
        info!("TelnetConnection::connect({}:{}) completed successfully", host, port);
        
        Ok(conn)
    }

    /// Start the connection with configuration.
    pub async fn start_with_config(
        host: &str,
        port: u16,
        enable_echo: bool,
        enable_binary: bool,
        enable_suppress_ga: bool,
    ) -> Result<Self> {
        let mut conn = Self::connect(host, port).await?;
        
        // Negotiate options
        if enable_echo {
            conn.negotiate_option(crate::protocol::OPT_ECHO, true).await?;
        }
        if enable_binary {
            conn.negotiate_option(crate::protocol::OPT_BINARY, true).await?;
        }
        if enable_suppress_ga {
            conn.negotiate_option(crate::protocol::OPT_SUPPRESS_GA, true).await?;
        }
        
        Ok(conn)
    }

    /// Negotiate an option.
    pub async fn negotiate_option(&mut self, option: u8, enable: bool) -> Result<()> {
        if enable {
            debug!("Requesting to enable option: {}", option);
            // Request to enable option
            let cmd = self.option_negotiator.request_enable(option);
            if cmd != TelnetCommand::Nop {
                debug!("Sending command to enable option {}: {:?}", option, cmd);
                self.send_command(&cmd).await?;
            }
        } else {
            debug!("Requesting to disable option: {}", option);
            // Request to disable option
            let cmd = self.option_negotiator.request_disable(option);
            if cmd != TelnetCommand::Nop {
                debug!("Sending command to disable option {}: {:?}", option, cmd);
                self.send_command(&cmd).await?;
            }
        }
        
        info!("Option negotiation completed: {} enabled={}", option, enable);
        Ok(())
    }

    /// Send a TELNET command.
    pub async fn send_command(&mut self, command: &TelnetCommand) -> Result<()> {
        let encoded = TelnetEncoder::encode_command(command);
        if let Some(stream) = &mut self.stream {
            debug!("Sending TELNET command ({} bytes): {:?}", encoded.len(), command);
            stream.write_all(&encoded).await?;
            stream.flush().await?;
            debug!("TELNET command sent successfully");
        }
        Ok(())
    }

    /// Send data to the server.
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        let encoded = TelnetEncoder::encode_data(data);
        if let Some(stream) = &mut self.stream {
            debug!("Sending data ({} bytes)", data.len());
            stream.write_all(&encoded).await?;
            stream.flush().await?;
            debug!("Data sent successfully");
        }
        Ok(())
    }

    /// Receive an event from the server.
    ///
    /// This reads from the stream and decodes TELNET commands.
    /// The decoder maintains state across calls, so commands spanning
    /// multiple reads are handled correctly.
    pub async fn receive(&mut self) -> Result<TelnetEvent> {
        let stream = self.stream.as_mut().ok_or(TelnetError::Disconnected)?;
        
        // Read a single byte to get events one at a time
        let mut buffer = [0u8; 1];
        let bytes_read = stream.read(&mut buffer).await?;
        
        // Create human-readable representation
        let readable = if buffer[0] >= 32 && buffer[0] <= 126 {
            // Printable ASCII character
            format!("'{}' (0x{:02x})", buffer[0] as char, buffer[0])
        } else if buffer[0] == 10 {
            "LF (\\n, 0x0a)".to_string()
        } else if buffer[0] == 13 {
            "CR (\\r, 0x0d)".to_string()
        } else if buffer[0] == 0 {
            "NUL (0x00)".to_string()
        } else {
            // Non-printable control character
            format!("0x{:02x} (control)", buffer[0])
        };
        
        debug!("Received {} byte(s): {} [raw: {:?}]", bytes_read, readable, buffer);
        
        // Check for EOF (server closed connection)
        if bytes_read == 0 {
            self.stream = None;
            self.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
            info!("Connection closed by server");
            return Ok(TelnetEvent::Closed);
        }
        
        // Decode the byte
        if let Some(cmd) = self.decoder.decode_byte(buffer[0]) {
            let event = match cmd {
                TelnetCommand::Data(byte) => TelnetEvent::Data(vec![byte]),
                TelnetCommand::Subnegotiation { option, data } => {
                    TelnetEvent::Command(TelnetCommand::Subnegotiation { option, data })
                }
                _ => TelnetEvent::Command(cmd),
            };
            debug!("Decoded event: {:?}", event);
            Ok(event)
        } else {
            // No complete command yet, return as data
            debug!("Byte treated as data (not a complete TELNET command): {} [raw: {:?}]", readable, buffer);
            Ok(TelnetEvent::Data(buffer.to_vec()))
        }
    }

    /// Disconnect from the server.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.stream = None;
        self.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
        Ok(())
    }

    /// Get the current state.
    pub fn state(&self) -> &StateManager {
        &self.state_manager
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.state_manager.connection_state() == crate::types::ConnectionState::Connected
    }

    /// Get a reference to the decoder for testing/debugging.
    pub fn get_decoder(&self) -> &TelnetDecoder {
        &self.decoder
    }

    /// Get a mutable reference to the decoder for testing/debugging.
    pub fn get_decoder_mut(&mut self) -> &mut TelnetDecoder {
        &mut self.decoder
    }
}

impl Default for TelnetConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_connection() {
        let conn = TelnetConnection::new();
        assert_eq!(conn.state().connection_state(), crate::types::ConnectionState::Disconnected);
    }

    #[test]
    fn test_is_connected() {
        let conn = TelnetConnection::new();
        assert!(!conn.is_connected());
    }
}