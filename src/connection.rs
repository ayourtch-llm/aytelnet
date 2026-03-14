//! TELNET connection manager.
//!
//! This module handles the TCP connection and async read/write tasks.

use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::decoder::TelnetDecoder;
use crate::encoder::TelnetEncoder;
use crate::error::TelnetError;
use crate::error::Result;
use crate::options::OptionNegotiator;
use crate::state::StateManager;
use crate::types::{TelnetCommand, TelnetEvent};

/// TELNET connection.
///
/// Manages the TCP connection and async read/write tasks.
pub struct TelnetConnection {
    /// TCP stream
    stream: Option<TcpStream>,
    
    /// State manager
    state_manager: StateManager,
    
    /// Option negotiator
    option_negotiator: OptionNegotiator,
    
    /// Data encoder
    encoder: TelnetEncoder,
    
    /// Data decoder
    decoder: TelnetDecoder,
    
    /// Send channel for commands to write task
    tx: Option<mpsc::Sender<Vec<u8>>>,
    
    /// Receive channel for events from read task
    rx: Option<mpsc::Receiver<TelnetEvent>>,
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
            tx: None,
            rx: None,
        }
    }

    /// Connect to a TELNET server.
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let mut conn = Self::new();
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connecting);
        
        let stream = TcpStream::connect((host, port)).await?;
        conn.stream = Some(stream);
        conn.state_manager.set_connection_state(crate::types::ConnectionState::Connected);
        
        // Start read/write tasks
        // tx sends Vec<u8> to write task, rx receives TelnetEvent from read task
        let (tx, _rx) = mpsc::channel::<Vec<u8>>(64);
        let (_rx2, rx) = mpsc::channel::<TelnetEvent>(64);
        conn.tx = Some(tx);
        conn.rx = Some(rx);
        
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
            // Request to enable option
            let cmd = self.option_negotiator.request_enable(option);
            if cmd != TelnetCommand::Nop {
                self.send_command(&cmd).await?;
            }
        } else {
            // Request to disable option
            let cmd = self.option_negotiator.request_disable(option);
            if cmd != TelnetCommand::Nop {
                self.send_command(&cmd).await?;
            }
        }
        
        Ok(())
    }

    /// Send a TELNET command.
    pub async fn send_command(&mut self, command: &TelnetCommand) -> Result<()> {
        let encoded = TelnetEncoder::encode_command(command);
        if let Some(tx) = &self.tx {
            tx.send(encoded).await.map_err(|e| {
                TelnetError::ChannelSend(e.to_string())
            })?;
        }
        Ok(())
    }

    /// Send data to the server.
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        let encoded = TelnetEncoder::encode_data(data);
        if let Some(tx) = &self.tx {
            tx.send(encoded).await.map_err(|e| {
                TelnetError::ChannelSend(e.to_string())
            })?;
        }
        Ok(())
    }

    /// Receive an event from the server.
    pub async fn receive(&mut self) -> Result<TelnetEvent> {
        if let Some(rx) = &mut self.rx {
            rx.recv().await.ok_or(TelnetError::ChannelRecv(
                "channel closed".to_string()
            ))
        } else {
            Err(TelnetError::Disconnected)
        }
    }

    /// Disconnect from the server.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.stream = None;
        self.tx = None;
        self.rx = None;
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