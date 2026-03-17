//! Vendor-neutral raw TELNET session.
//!
//! Wraps [`TelnetConnection`] with a simple `send`/`receive` API that handles
//! TELNET protocol events internally, returning only raw data bytes to the caller.

#![deny(unused_must_use)]

use std::time::Duration;

use crate::connection::TelnetConnection;
use crate::error::{Result, TelnetError};
use crate::types::TelnetEvent;

/// A vendor-neutral raw TELNET session.
///
/// This is a thin wrapper over [`TelnetConnection`] that filters out TELNET
/// protocol events (commands, option negotiations) and exposes only raw data
/// bytes via `send()` / `receive()`.
///
/// Use this when you want to drive login, prompt detection, and command
/// execution from an external state machine (e.g. TextFSMPlus templates)
/// rather than relying on vendor-specific logic baked into the transport.
#[derive(Debug)]
pub struct RawTelnetSession {
    conn: TelnetConnection,
}

impl RawTelnetSession {
    /// Connect to a TELNET server and negotiate standard options
    /// (echo, binary, suppress go-ahead).
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let conn = TelnetConnection::start_with_config(host, port, true, true, true).await?;
        Ok(Self { conn })
    }

    /// Create from an already-connected [`TelnetConnection`].
    pub fn from_connection(conn: TelnetConnection) -> Self {
        Self { conn }
    }

    /// Send raw bytes to the remote end.
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.conn.send(data).await
    }

    /// Receive raw bytes from the remote end.
    ///
    /// Semantics:
    /// - If data is immediately available, returns it right away.
    /// - Only blocks up to `timeout` if there is no data yet.
    /// - Returns an empty `Vec` if the timeout expires with no data (not an error).
    /// - Filters out TELNET protocol events internally — only `Data` events
    ///   are returned to the caller.
    /// - Returns an error on connection close.
    pub async fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Ok(vec![]);
            }
            let remaining = deadline - now;
            match tokio::time::timeout(remaining, self.conn.receive()).await {
                Ok(Ok(TelnetEvent::Data(data))) => return Ok(data),
                Ok(Ok(TelnetEvent::Closed)) => return Err(TelnetError::Disconnected),
                Ok(Ok(TelnetEvent::Error(e))) => return Err(e),
                Ok(Ok(_)) => continue, // Command, OptionNegotiated — skip
                Ok(Err(e)) => return Err(e),
                Err(_) => return Ok(vec![]), // timeout expired
            }
        }
    }

    /// Close the connection.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.conn.disconnect().await
    }
}
