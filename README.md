# aytelnet

> ⚠️ **Work in Progress** — This library is under active development. APIs may change.

An async TELNET client library for Rust, built on [Tokio](https://tokio.rs/). Implements the TELNET protocol per RFC 854 and related specifications (RFC 855–860).

## Features

- **Fully async** — Built on Tokio for non-blocking I/O
- **RFC-compliant** — Implements RFC 854 (Telnet Protocol), RFC 855 (Option Specifications), RFC 856 (Binary Transmission), RFC 857 (Echo), RFC 858 (Suppress Go-Ahead), RFC 859 (Status), and RFC 860 (Timing Mark)
- **Layered API** — Low-level `TelnetConnection`, mid-level `CiscoTelnet`, and high-level `CiscoConn`
- **IAC escaping** — Correct handling of IAC (0xFF) in data streams and subnegotiation
- **Option negotiation** — Full WILL/WONT/DO/DONT state machine with loop prevention

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
aytelnet = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Execute a command on a Cisco device

```rust
use aytelnet::cisco_conn::{CiscoConn, ConnectionType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = CiscoConn::new(
        "192.168.1.1",
        ConnectionType::CiscoTelnet,
        "admin",
        "password",
    ).await?;

    let output = conn.run_cmd("show version").await?;
    println!("{}", output);
    Ok(())
}
```

## API Reference

### `TelnetConnection` — Low-Level Async Telnet Client

The core connection type providing direct TELNET protocol access over TCP.

```rust
use aytelnet::TelnetConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a telnet server
    let mut client = TelnetConnection::connect("example.com", 23).await?;

    // Negotiate options
    client.negotiate_option(aytelnet::OPT_ECHO, true).await?;
    client.negotiate_option(aytelnet::OPT_BINARY, true).await?;
    client.negotiate_option(aytelnet::OPT_SUPPRESS_GA, true).await?;

    // Send data (IAC bytes are automatically escaped)
    client.send(b"show version\n").await?;

    // Receive response
    let event = client.receive().await?;
    println!("Received: {:?}", event);

    client.disconnect().await?;
    Ok(())
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `TelnetConnection::connect(host, port)` | Connect to a TELNET server via TCP |
| `TelnetConnection::start_with_config(host, port, echo, binary, suppress_ga)` | Connect and negotiate options |
| `negotiate_option(option, enable)` | Request to enable/disable a TELNET option |
| `send_command(command)` | Send a raw TELNET command |
| `send(data)` | Send data with IAC escaping |
| `receive()` | Receive a `TelnetEvent` from the server |
| `disconnect()` | Close the connection |
| `is_connected()` | Check connection status |
| `state()` | Access the `StateManager` |

#### Events

`receive()` returns a `TelnetEvent`:

- `TelnetEvent::Data(Vec<u8>)` — Application data from the server
- `TelnetEvent::Command(TelnetCommand)` — A TELNET protocol command
- `TelnetEvent::OptionNegotiated { option, enabled }` — Option state change
- `TelnetEvent::Closed` — Connection closed by server
- `TelnetEvent::Error(TelnetError)` — An error occurred

### `CiscoTelnet` — Mid-Level Cisco Device Client

Provides automated login and stateful communication with Cisco devices.

```rust
use aytelnet::CiscoTelnet;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CiscoTelnet::new("192.168.1.1", "admin", "password")
        .with_timeout(Duration::from_secs(60))
        .with_read_timeout(Duration::from_secs(5))
        .with_prompt("Router#")
        .with_prompt("Switch#");

    client.connect().await?;

    client.send(b"show version\n").await?;
    let output = client.receive_until(b"#", Duration::from_secs(10)).await?;
    println!("{}", output);

    client.disconnect().await?;
    Ok(())
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `CiscoTelnet::new(address, username, password)` | Create a new client |
| `with_timeout(duration)` | Set connection timeout (default: 30s) |
| `with_read_timeout(duration)` | Set read timeout (default: 10s) |
| `with_prompt(pattern)` | Add a prompt pattern to detect login completion |
| `with_prompts(patterns)` | Add multiple prompt patterns |
| `connect()` | Connect, authenticate, and wait for prompt |
| `send(data)` | Send data to the device |
| `receive_until(pattern, timeout)` | Read until pattern is found |
| `receive_line()` | Read a single line |
| `disconnect()` | Close the connection |
| `is_logged_in()` | Check authentication status |
| `state()` | Get the current `CiscoTelnetState` |

#### Address Format

Supports IPv4, IPv6, and hostnames with optional port:

- `"192.168.1.1"` or `"192.168.1.1:23"`
- `"[::1]"` or `"[::1]:23"`
- `"router.local"` or `"router.local:2323"`

Default port is 23 if not specified.

### `CiscoConn` — High-Level One-Shot Command Executor

The simplest API for executing a single command on a Cisco device. Handles the full lifecycle: connect → authenticate → execute → disconnect.

```rust
use aytelnet::cisco_conn::{CiscoConn, ConnectionType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // With default timeouts (30s connection, 30s read)
    let conn = CiscoConn::new(
        "192.168.1.1",
        ConnectionType::CiscoTelnet,
        "admin",
        "password",
    ).await?;

    let output = conn.run_cmd("show interfaces").await?;
    println!("{}", output);
    Ok(())
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `CiscoConn::new(target, conntype, username, password)` | Create with default timeouts |
| `CiscoConn::with_timeouts(target, conntype, user, pass, timeout, read_timeout)` | Create with custom timeouts |
| `run_cmd(command)` | Execute a command and return the output |
| `target()` | Get the target address |
| `username()` | Get the username |
| `conntype()` | Get the connection type |

#### Default Prompts

CiscoConn recognizes these prompts to detect successful login:
- `"Router#"`, `"Switch#"`, `"config#"`, `"cli#"`

### Protocol Constants

```rust
use aytelnet::*;

// Option codes
const BINARY: u8     = OPT_BINARY;      // 0  (RFC 856)
const ECHO: u8       = OPT_ECHO;        // 1  (RFC 857)
const SUPPRESS: u8   = OPT_SUPPRESS_GA; // 3  (RFC 858)
const STATUS: u8     = OPT_STATUS;      // 5  (RFC 859)
const TIMING: u8     = OPT_TIMING_MARK; // 6  (RFC 860)
const TERM_TYPE: u8  = OPT_TERMINAL_TYPE; // 24
const WINDOW: u8     = OPT_NAWS;        // 31
const LINEMODE: u8   = OPT_LINEMODE;    // 34

// Command bytes (RFC 854)
const _IAC: u8  = IAC;  // 255  Interpret As Command
const _WILL: u8 = WILL; // 251
const _WONT: u8 = WONT; // 252
const _DO: u8   = DO;   // 253
const _DONT: u8 = DONT; // 254
const _SB: u8   = SB;   // 250  Subnegotiation Begin
const _SE: u8   = SE;   // 240  Subnegotiation End
```

### Error Handling

All async methods return `Result<T, TelnetError>`. Error variants:

| Variant | Description |
|---------|-------------|
| `TelnetError::Io` | Underlying I/O error |
| `TelnetError::Protocol` | Invalid protocol command or state |
| `TelnetError::Negotiation` | Option negotiation failure |
| `TelnetError::Timeout` | Connection or operation timeout |
| `TelnetError::Disconnected` | Connection was closed |
| `TelnetError::InvalidState` | Operation not valid in current state |

## Examples

Run the included examples:

```bash
# One-shot command execution
cargo run --example cisco_conn <host> <username> <password> <command>

# Interactive telnet client (supports Ctrl-] escape)
cargo run --example telnet_client <host> [port]

# Cisco telnet session
cargo run --example cisco_telnet
```

## Testing

```bash
# Run all library tests
cargo test --lib

# Run specific test module
cargo test --lib rfc_compliance_tests
```

## RFC Compliance

| RFC | Title | Status |
|-----|-------|--------|
| RFC 854 | Telnet Protocol Specification | ✅ Implemented |
| RFC 855 | Telnet Option Specifications | ✅ Implemented |
| RFC 856 | Telnet Binary Transmission | ✅ Implemented |
| RFC 857 | Telnet Echo Option | ✅ Implemented |
| RFC 858 | Telnet Suppress Go-Ahead | ✅ Implemented |
| RFC 859 | Telnet Status Option | ⚠️ Basic support |
| RFC 860 | Telnet Timing Mark | ⚠️ Basic support |
| RFC 1184 | Telnet Linemode Option | 🔲 Not yet implemented |

See [`docs/spec-coverage.md`](docs/spec-coverage.md) for detailed coverage.

## License

See [LICENSE](LICENSE) for details.
