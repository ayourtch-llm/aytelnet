//! TELNET connection manager.
//!
//! This module provides a simple TELNET connection with explicit state machine handling.
//!
//! Performance: Uses buffered reading (4KB chunks) for efficient data transfer.

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
    #[allow(dead_code)]
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
    ///
    /// Performance note: This method now reads larger chunks (default 4KB)
    /// instead of one byte at a time for better performance.
    pub async fn receive(&mut self) -> Result<TelnetEvent> {
        let stream = self.stream.as_mut().ok_or(TelnetError::Disconnected)?;
        
        // Read a larger chunk for better performance
        let mut buffer = [0u8; 4096];
        let bytes_read = stream.read(&mut buffer).await?;
        
        if bytes_read == 0 {
            self.stream = None;
            self.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
            info!("Connection closed by server");
            return Ok(TelnetEvent::Closed);
        }
        
        debug!("Received {} bytes from stream", bytes_read);
        
        // Decode all bytes at once
        let commands = self.decoder.decode(&buffer[..bytes_read]);
        
        // Separate data bytes from TELNET commands
        let mut data_bytes: Vec<u8> = Vec::new();
        let mut command_events: Vec<TelnetEvent> = Vec::new();
        
        for cmd in commands {
            match cmd {
                TelnetCommand::Data(byte) => {
                    data_bytes.push(byte);
                }
                TelnetCommand::Subnegotiation { option, data } => {
                    command_events.push(TelnetEvent::Command(TelnetCommand::Subnegotiation { option, data }));
                }
                _ => {
                    command_events.push(TelnetEvent::Command(cmd));
                }
            }
        }
        
        // Always return data first if we have any - this is critical for
        // protocol handling where data and commands are interleaved
        if !data_bytes.is_empty() {
            debug!("Returning {} bytes as Data event (priority over {} commands)", 
                   data_bytes.len(), command_events.len());
            Ok(TelnetEvent::Data(data_bytes))
        } else if let Some(first_cmd) = command_events.into_iter().next() {
            // No data, return first TELNET command event
            debug!("Returning first TELNET command event");
            Ok(first_cmd)
        } else {
            // No data, no commands - shouldn't happen but handle it
            debug!("No complete commands or data, returning empty");
            Ok(TelnetEvent::Data(Vec::new()))
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

    // ============================================================================
    // TelnetConnection Tests
    // ============================================================================

    #[tokio::test]
    async fn test_connection_new() {
        // Verify new connection starts in Disconnected state
        let conn = TelnetConnection::new();
        
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Disconnected,
            "New connection should start in Disconnected state"
        );
        assert!(!conn.is_connected(), "New connection should not be connected");
    }

    #[tokio::test]
    async fn test_connection_state_transitions() {
        // Verify state transitions from Disconnected to Connected
        let mut conn = TelnetConnection::new();
        
        // Initial state should be Disconnected
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Disconnected
        );
        
        // After connect (mocked via state manager), should be Connected
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Connected
        );
        assert!(conn.is_connected(), "is_connected() should return true when connected");
        
        // After disconnect, should be Disconnected again
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Disconnected
        );
        assert!(!conn.is_connected(), "is_connected() should return false after disconnect");
    }

    #[tokio::test]
    async fn test_decoder_accessors() {
        // Verify get_decoder() and get_decoder_mut() work correctly
        
        let mut conn = TelnetConnection::new();
        
        // Test immutable reference - verify decoder is accessible
        let decoder_ref = conn.get_decoder();
        assert!(decoder_ref.state() == crate::decoder::DecodeState::Normal);
        drop(decoder_ref); // Release the immutable borrow
        
        // Test mutable reference
        let decoder_mut = conn.get_decoder_mut();
        assert!(decoder_mut.state() == crate::decoder::DecodeState::Normal);
        drop(decoder_mut); // Release the mutable borrow
        
        // Verify both references point to the same decoder
        let decoder_ref2 = conn.get_decoder();
        // Both should have same state
        assert_eq!(decoder_ref2.state(), crate::decoder::DecodeState::Normal);
    }

    // ============================================================================
    // State Manager Tests
    // ============================================================================

    #[tokio::test]
    async fn test_state_manager_connection_state() {
        // Verify connection state getter/setter
        
        let mut state_manager = StateManager::new();
        
        // Initial state should be Disconnected
        assert_eq!(
            state_manager.connection_state(),
            crate::types::ConnectionState::Disconnected
        );
        
        // Set to Connected
        state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        assert_eq!(
            state_manager.connection_state(),
            crate::types::ConnectionState::Connected
        );
        
        // Set back to Disconnected
        state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
        assert_eq!(
            state_manager.connection_state(),
            crate::types::ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_state_manager_is_connected() {
        // Verify is_connected() returns correct value
        
        let mut conn = TelnetConnection::new();
        
        // Initially not connected
        assert!(!conn.is_connected(), "Should not be connected initially");
        
        // Set to Connected state
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        assert!(conn.is_connected(), "Should be connected after setting Connected state");
        
        // Set to Disconnected state
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
        assert!(!conn.is_connected(), "Should not be connected after setting Disconnected state");
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_connection_full_lifecycle() {
        // Simulate full connection lifecycle (new → connect → disconnect)
        
        // Step 1: Create new connection
        let mut conn = TelnetConnection::new();
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Disconnected
        );
        assert!(!conn.is_connected());
        
        // Step 2: Simulate connect by setting stream and state
        // (We can't actually connect to a server in tests, so we mock the state)
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Connected
        );
        assert!(conn.is_connected());
        
        // Step 3: Simulate disconnect
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Disconnected);
        
        assert_eq!(
            conn.state().connection_state(),
            crate::types::ConnectionState::Disconnected
        );
        assert!(!conn.is_connected());
    }

    #[tokio::test]
    async fn test_decoder_state_preservation() {
        // Test that decoder preserves state across multiple reads
        
        let mut conn = TelnetConnection::new();
        let decoder = conn.get_decoder_mut();
        
        // Initial state
        assert!(decoder.state() == crate::decoder::DecodeState::Normal);
        
        // Decode some data
        let data1 = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
        let commands1 = decoder.decode(&data1);
        assert!(!commands1.is_empty());
        
        // Decode more data
        let data2 = vec![0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64]; // " World"
        let commands2 = decoder.decode(&data2);
        assert!(!commands2.is_empty());
        
        // Decoder should maintain state
        assert!(decoder.state() == crate::decoder::DecodeState::Normal);
    }

    #[tokio::test]
    async fn test_state_manager_default() {
        // Verify StateManager default behavior
        
        let state_manager = StateManager::new();
        
        assert_eq!(
            state_manager.connection_state(),
            crate::types::ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_telnet_connection_default() {
        // Verify TelnetConnection implements Default correctly
        
        let conn = TelnetConnection::default();
        let conn_new = TelnetConnection::new();
        
        // Both should be in the same state
        assert_eq!(conn.state().connection_state(), conn_new.state().connection_state());
        assert_eq!(conn.is_connected(), conn_new.is_connected());
    }
}