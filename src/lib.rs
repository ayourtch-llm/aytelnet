//! TELNET client library for Rust using async/await and tokio.
//!
//! This library provides a fully async TELNET client implementation
//! following RFC 854 and related TELNET specifications.

#![deny(unused_must_use)]

//! # Example
//!
//! ```no_run
//! use aytelnet::TelnetConnection;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = TelnetConnection::connect("example.com", 23).await?;
//!     
//!     // Negotiate options
//!     client.negotiate_option(aytelnet::OPT_ECHO, true).await?;
//!     client.negotiate_option(aytelnet::OPT_BINARY, true).await?;
//!     
//!     // Send data
//!     client.send(b"echo Hello, TELNET!\n").await?;
//!     
//!     // Receive response
//!     let event = client.receive().await?;
//!     println!("Received: {:?}", event);
//!     
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```

pub mod cisco_conn;
pub mod cisco_telnet;
pub mod connection;
pub mod decoder;
pub mod encoder;
mod decoder_persistence_tests;
mod rfc_compliance_tests;
pub mod error;
pub mod options;
pub mod protocol;
pub mod raw_session;
pub mod state;
pub mod types;

pub use connection::TelnetConnection;
pub use cisco_conn::{CiscoConn, ConnectionType};
pub use cisco_telnet::CiscoTelnet;
pub use decoder::TelnetDecoder;
pub use encoder::TelnetEncoder;
pub use error::TelnetError;
pub use options::OptionNegotiator;
pub use protocol::*;
pub use raw_session::RawTelnetSession;
pub use state::StateManager;
pub use types::*;