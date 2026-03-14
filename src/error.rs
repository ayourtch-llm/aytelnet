//! TELNET error types.

use thiserror::Error;

/// Main error type for TELNET operations.
#[derive(Error, Debug)]
pub enum TelnetError {
    /// I/O error from underlying stream operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Protocol error - invalid command or state.
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Option negotiation error.
    #[error("Negotiation error: {0}")]
    Negotiation(String),

    /// Connection timeout.
    #[error("Connection timeout")]
    Timeout,

    /// Connection was closed.
    #[error("Connection disconnected")]
    Disconnected,

    /// Invalid state for operation.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Channel send error.
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Channel receive error.
    #[error("Channel receive error: {0}")]
    ChannelRecv(String),
}

/// Result type alias for TELNET operations.
pub type Result<T> = std::result::Result<T, TelnetError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let io_err = TelnetError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(io_err.to_string().contains("I/O error"));

        let protocol_err = TelnetError::Protocol("invalid command".to_string());
        assert!(protocol_err.to_string().contains("Protocol error"));

        let timeout = TelnetError::Timeout;
        assert_eq!(timeout.to_string(), "Connection timeout");

        let disconnected = TelnetError::Disconnected;
        assert_eq!(disconnected.to_string(), "Connection disconnected");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let telnet_err: TelnetError = io_err.into();
        assert!(matches!(telnet_err, TelnetError::Io(_)));
    }
}