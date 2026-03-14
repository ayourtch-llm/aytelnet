# TELNET Client Implementation Plan

## Overview

This document provides a comprehensive implementation plan for a TELNET client library using Rust with async/await and the tokio runtime.

## 1. Architecture

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      TELNET CLIENT                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Connection  │  │  Option      │  │    Data              │  │
│  │  Manager     │◄─┤  Negotiator  │  │    Encoder/Decoder   │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│         │                  │                    │               │
│         ▼                  ▼                    ▼               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    State Machine                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Application Layer                      │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Core Components

1. **Connection Manager**: Handles TCP connection lifecycle
2. **Option Negotiator**: Manages TELNET option negotiation
3. **Data Encoder/Decoder**: Handles IAC byte escaping and command parsing
4. **State Machine**: Manages connection states (connected, negotiating, etc.)
5. **Command Handler**: Processes incoming TELNET commands

## 2. Module Structure

```
aytelnet/
├── src/
│   ├── lib.rs                 # Public API
│   ├── connection.rs          # Connection Manager
│   ├── options.rs             # Option Negotiator
│   ├── protocol.rs            # Protocol constants and types
│   ├── encoder.rs             # Data encoder (IAC escaping)
│   ├── decoder.rs             # Data decoder (command parsing)
│   ├── state.rs               # State machine
│   ├── error.rs               # Error types
│   └── types.rs               # Shared types and enums
├── tests/
│   └── integration.rs         # Integration tests
├── Cargo.toml
└── README.md
```

## 3. Key Data Structures

### 3.1 Protocol Constants

```rust
pub const IAC: u8 = 255;      // Interpret As Command
pub const DO: u8 = 254;       // DO command
pub const DONT: u8 = 253;     // DON'T command
pub const WILL: u8 = 252;     // WILL command
pub const WONT: u8 = 251;     // WON'T command

// Common option codes
pub const OPT_ECHO: u8 = 1;
pub const OPT_SUPPRESS_GA: u8 = 3;
pub const OPT_BINARY: u8 = 8;
pub const OPT_STATUS: u8 = 5;
pub const OPT_TIMING_MARK: u8 = 6;
pub const OPT_TERMINAL_TYPE: u8 = 24;
pub const OPT_NAWS: u8 = 31;
pub const OPT_LINEMODE: u8 = 34;
```

### 3.2 TELNET Commands

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetCommand {
    Do(u8),
    Dont(u8),
    Will(u8),
    Wont(u8),
    Subnegotiation { option: u8, data: Vec<u8> },
    Iac,                    // IAC followed by non-command
    Data(u8),               // Regular data byte
}
```

### 3.3 Option State

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionState {
    Closed,           // Option not enabled
    Enabled,          // Option enabled
    WantsEnable,      // We want to enable this option
    WantsDisable,     // We want to disable this option
    RemoteWantsEnable, // Remote wants to enable this option
    RemoteWantsDisable, // Remote wants to disable this option
}
```

### 3.4 Connection State

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Negotiating,
    Closing,
}
```

### 3.5 Configuration

```rust
#[derive(Debug, Clone)]
pub struct TelnetConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub enable_echo: bool,
    pub enable_binary: bool,
    pub enable_suppress_ga: bool,
    pub terminal_type: Option<String>,
}
```

### 3.6 Client State

```rust
#[derive(Debug, Clone)]
pub struct ClientState {
    pub connection_state: ConnectionState,
    pub options: HashMap<u8, OptionState>,
    pub local_echo: bool,
    pub remote_echo: bool,
    pub binary_mode: bool,
}
```

## 4. Core Algorithms

### 4.1 TELNET Command Encoding

```rust
pub fn encode_command(command: &TelnetCommand) -> Vec<u8> {
    match command {
        TelnetCommand::Do(opt) => vec![IAC, DO, *opt],
        TelnetCommand::Dont(opt) => vec![IAC, DONT, *opt],
        TelnetCommand::Will(opt) => vec![IAC, WILL, *opt],
        TelnetCommand::Wont(opt) => vec![IAC, WONT, *opt],
        TelnetCommand::Subnegotiation { option, data } => {
            let mut result = vec![IAC, SB, *option];
            result.extend_from_slice(data);
            result.push(IAC);
            result.push(SE);
            result
        }
        TelnetCommand::Iac => vec![IAC, IAC],
        TelnetCommand::Data(byte) => {
            if *byte == IAC {
                vec![IAC, IAC]
            } else {
                vec![*byte]
            }
        }
    }
}
```

### 4.2 TELNET Command Decoding

```rust
pub fn decode_byte(byte: u8, state: &DecodeState) -> Option<TelnetCommand> {
    match state {
        DecodeState::Normal => {
            if byte == IAC {
                DecodeState::Iac
            } else {
                Some(TelnetCommand::Data(byte))
            }
        }
        DecodeState::Iac => {
            match byte {
                DO => Some(TelnetCommand::Do(0)), // Will get option in next step
                DONT => Some(TelnetCommand::Dont(0)),
                WILL => Some(TelnetCommand::Will(0)),
                WONT => Some(TelnetCommand::Wont(0)),
                SB => Some(TelnetCommand::Subnegotiation { option: 0, data: Vec::new() }),
                SE => None, // Unexpected SE
                IAC => Some(TelnetCommand::Iac),
                _ => Some(TelnetCommand::Data(byte)),
            }
        }
    }
}
```

### 4.3 Option Negotiation State Machine

```rust
pub fn handle_option_negotiation(
    state: &mut OptionState,
    command: TelnetCommand,
) -> Option<TelnetCommand> {
    match (state, command) {
        // We want to enable option, remote accepts
        (OptionState::WantsEnable, TelnetCommand::Will(opt)) => {
            *state = OptionState::Enabled;
            Some(TelnetCommand::Will(opt))
        }
        // We want to enable option, remote refuses
        (OptionState::WantsEnable, TelnetCommand::Wont(opt)) => {
            *state = OptionState::Closed;
            None
        }
        // Remote wants to enable option, we accept
        (OptionState::Closed, TelnetCommand::Do(opt)) => {
            *state = OptionState::Enabled;
            Some(TelnetCommand::Will(opt))
        }
        // Remote wants to enable option, we refuse
        (OptionState::Closed, TelnetCommand::Do(opt)) => {
            *state = OptionState::Closed;
            Some(TelnetCommand::Wont(opt))
        }
        // ... more states
        _ => None
    }
}
```

## 5. Async/Tokio Integration

### 5.1 Connection Manager

```rust
pub struct TelnetClient {
    config: TelnetConfig,
    state: Arc<Mutex<ClientState>>,
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<TelnetEvent>,
}

impl TelnetClient {
    pub async fn connect(config: TelnetConfig) -> Result<Self> {
        // Create TCP connection
        let socket = TcpStream::connect((config.host.as_str(), config.port)).await?;
        
        // Split into read/write halves
        let (read, write) = socket.into_split();
        
        // Spawn read/write tasks
        let (tx, rx) = mpsc::channel(64);
        
        let read_task = tokio::spawn(read_task(read, tx.clone()));
        let write_task = tokio::spawn(write_task(write, rx));
        
        Ok(Self { config, state: Arc::new(Mutex::new(ClientState::default())), tx, rx })
    }
    
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        // Encode data with IAC escaping
        let encoded = self.encode_data(data);
        // Send through channel
        self.tx.send(encoded).await?;
        Ok(())
    }
    
    pub async fn receive(&mut self) -> Result<TelnetEvent> {
        self.rx.recv().await.ok_or(Error::Disconnected)
    }
}
```

### 5.2 Read Task

```rust
async fn read_task(
    mut read: ReadHalf<TcpStream>,
    tx: mpsc::Sender<TelnetEvent>,
) {
    let mut decoder = TelnetDecoder::new();
    let mut buf = [0u8; 4096];
    
    loop {
        match read.read(&mut buf).await {
            Ok(0) => {
                // Connection closed
                tx.send(TelnetEvent::Closed).await.ok();
                break;
            }
            Ok(n) => {
                // Decode bytes
                for &byte in &buf[..n] {
                    if let Some(command) = decoder.decode(byte) {
                        tx.send(TelnetEvent::Command(command)).await.ok();
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                tx.send(TelnetEvent::Error(e)).await.ok();
                break;
            }
        }
    }
}
```

### 5.3 Write Task

```rust
async fn write_task(
    mut write: WriteHalf<TcpStream>,
    mut rx: mpsc::Receiver<Vec<u8>>,
) {
    while let Some(data) = rx.recv().await {
        match write.write_all(&data).await {
            Ok(_) => {}
            Err(e) => {
                // Handle write error
                break;
            }
        }
    }
}
```

## 6. Error Handling Strategy

### 6.1 Error Types

```rust
#[derive(Debug)]
pub enum TelnetError {
    Io(std::io::Error),
    Protocol(String),
    Negotiation(String),
    Timeout,
    Disconnected,
    InvalidState(String),
}

impl std::fmt::Display for TelnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TelnetError::Io(e) => write!(f, "I/O error: {}", e),
            TelnetError::Protocol(e) => write!(f, "Protocol error: {}", e),
            TelnetError::Negotiation(e) => write!(f, "Negotiation error: {}", e),
            TelnetError::Timeout => write!(f, "Connection timeout"),
            TelnetError::Disconnected => write!(f, "Connection disconnected"),
            TelnetError::InvalidState(e) => write!(f, "Invalid state: {}", e),
        }
    }
}
```

### 6.2 Result Type

```rust
pub type Result<T> = std::result::Result<T, TelnetError>;
```

## 7. Testing Strategy

### 7.1 Unit Tests

Test each component independently:

1. **Encoder Tests**
   - IAC byte escaping
   - Command encoding
   - Subnegotiation encoding

2. **Decoder Tests**
   - Command parsing
   - IAC byte handling
   - Subnegotiation parsing

3. **Option Negotiation Tests**
   - WILL/WONT handling
   - DO/DONT handling
   - State transitions

4. **State Machine Tests**
   - State transitions
   - Invalid state handling

### 7.2 Integration Tests

1. **Connection Tests**
   - Connect to mock server
   - Disconnect handling
   - Reconnection

2. **Option Negotiation Tests**
   - Echo negotiation
   - Binary mode negotiation
   - Multiple option negotiation

3. **Protocol Conformance Tests**
   - RFC 854 compliance
   - RFC 855 compliance

### 7.3 Mock Server

Create a mock TELNET server for testing:

```rust
pub struct MockTelnetServer {
    listener: TcpListener,
    handlers: Arc<Mutex<Vec<Box<dyn Fn(TelnetCommand) + Send + Sync>>>>,
}

impl MockTelnetServer {
    pub async fn bind(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        Ok(Self {
            listener,
            handlers: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub fn on_command(&self, handler: impl Fn(TelnetCommand) + Send + Sync + 'static) {
        self.handlers.lock().unwrap().push(Box::new(handler));
    }
}
```

## 8. Implementation Phases

### Phase 1: Core Protocol (Week 1-2)
- [ ] Define types and constants
- [ ] Implement encoder/decoder
- [ ] Implement basic connection manager
- [ ] Write unit tests for encoder/decoder

### Phase 2: Option Negotiation (Week 3-4)
- [ ] Implement option state machine
- [ ] Implement basic option negotiation (ECHO, SUPPRESS-GA, BINARY)
- [ ] Write integration tests

### Phase 3: Enhanced Features (Week 5-6)
- [ ] Implement TERMINAL-TYPE option
- [ ] Implement NAWS option
- [ ] Add configuration options

### Phase 4: Optional Extensions (Week 7-8)
- [ ] Implement LINEMODE option
- [ ] Implement STATUS option
- [ ] Implement TIMING-MARK option

### Phase 5: Polish and Documentation (Week 9-10)
- [ ] Write documentation
- [ ] Create examples
- [ ] Performance optimization
- [ ] Final testing

## 9. Public API Design

```rust
pub struct TelnetClient {
    // Internal state
}

impl TelnetClient {
    // Connection
    pub async fn connect(host: &str, port: u16) -> Result<Self>;
    pub async fn disconnect(&mut self);
    
    // Configuration
    pub fn with_echo(mut self, enable: bool) -> Self;
    pub fn with_binary(mut self, enable: bool) -> Self;
    pub fn with_terminal_type(mut self, ty: &str) -> Self;
    
    // Option Negotiation
    pub async fn negotiate_echo(&mut self, enable: bool) -> Result<()>;
    pub async fn negotiate_binary(&mut self, enable: bool) -> Result<()>;
    pub async fn negotiate_suppress_ga(&mut self, enable: bool) -> Result<()>;
    
    // Data Transfer
    pub async fn send(&self, data: &[u8]) -> Result<()>;
    pub async fn receive(&mut self) -> Result<TelnetEvent>;
    
    // Events
    pub fn events(&mut self) -> &mut mpsc::Receiver<TelnetEvent>;
}

#[derive(Debug)]
pub enum TelnetEvent {
    Data(Vec<u8>),
    Command(TelnetCommand),
    OptionNegotiated(u8, bool),
    Closed,
    Error(TelnetError),
}
```

## 10. Dependencies (Cargo.toml)

```toml
[package]
name = "aytelnet"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
bytes = "1"
thiserror = "1"

[dev-dependencies]
tokio-test = "0.4"
```

## 11. RFC Compliance Checklist

- [ ] RFC 854: Telnet Protocol Specification (MUST)
- [ ] RFC 855: Telnet Option Specifications (MUST)
- [ ] RFC 856: Telnet BINARY Transmission (SHOULD)
- [ ] RFC 857: Telnet ECHO Option (SHOULD)
- [ ] RFC 858: Telnet SUPPRESS-GO-AHEAD Option (SHOULD)
- [ ] RFC 859: Telnet STATUS Option (MAY)
- [ ] RFC 860: Telnet TIMING-MARK Option (MAY)
- [ ] RFC 1184: Telnet LINEMODE Option (MAY)

## 12. Next Steps

1. Create project structure
2. Implement Phase 1 (Core Protocol)
3. Write tests for each component
4. Proceed to Phase 2 (Option Negotiation)
5. Continue through remaining phases