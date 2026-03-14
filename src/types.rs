//! Shared types for TELNET client.

use std::fmt;

use crate::error::TelnetError;

/// TELNET command types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelnetCommand {
    /// DO option - request remote to enable option
    Do(u8),
    
    /// DON'T option - request remote to disable option
    Dont(u8),
    
    /// WILL option - declare local will enable option
    Will(u8),
    
    /// WON'T option - declare local will not enable option
    Wont(u8),
    
    /// Subnegotiation - option with parameters
    Subnegotiation {
        /// Option code
        option: u8,
        /// Subnegotiation data
        data: Vec<u8>,
    },
    
    /// NOP - no operation
    Nop,
    
    /// GA - go ahead
    GoAhead,
    
    /// AO - abort output
    AbortOutput,
    
    /// AYT - are you there
    AreYouThere,
    
    /// EC - erase character
    EraseCharacter,
    
    /// EL - erase line
    EraseLine,
    
    /// IP - interrupt process
    InterruptProcess,
    
    /// BRK - break
    Break,
    
    /// DM - data mark
    DataMark,
    
    /// EOR - end of record
    EndOfRecord,
    
    /// EOF - end of file
    EndOfFile,
    
    /// SUSP - suspend
    Suspend,
    
    /// ABOR - abort
    Abort,
    
    /// Data byte (not a command)
    Data(u8),
}

impl fmt::Display for TelnetCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelnetCommand::Do(opt) => write!(f, "DO {}", opt),
            TelnetCommand::Dont(opt) => write!(f, "DONT {}", opt),
            TelnetCommand::Will(opt) => write!(f, "WILL {}", opt),
            TelnetCommand::Wont(opt) => write!(f, "WONT {}", opt),
            TelnetCommand::Subnegotiation { option, data } => {
                write!(f, "SB {} ({} bytes)", option, data.len())
            }
            TelnetCommand::Nop => write!(f, "NOP"),
            TelnetCommand::GoAhead => write!(f, "GA"),
            TelnetCommand::AbortOutput => write!(f, "AO"),
            TelnetCommand::AreYouThere => write!(f, "AYT"),
            TelnetCommand::EraseCharacter => write!(f, "EC"),
            TelnetCommand::EraseLine => write!(f, "EL"),
            TelnetCommand::InterruptProcess => write!(f, "IP"),
            TelnetCommand::Break => write!(f, "BRK"),
            TelnetCommand::DataMark => write!(f, "DM"),
            TelnetCommand::EndOfRecord => write!(f, "EOR"),
            TelnetCommand::EndOfFile => write!(f, "EOF"),
            TelnetCommand::Suspend => write!(f, "SUSP"),
            TelnetCommand::Abort => write!(f, "ABOR"),
            TelnetCommand::Data(byte) => write!(f, "DATA({})", byte),
        }
    }
}

/// Option states for TELNET option negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionState {
    /// Option is closed/disabled
    Closed,
    
    /// Option is enabled
    Enabled,
    
    /// We want to enable this option
    WantsEnable,
    
    /// We want to disable this option
    WantsDisable,
    
    /// Remote wants to enable this option
    RemoteWantsEnable,
    
    /// Remote wants to disable this option
    RemoteWantsDisable,
}

impl OptionState {
    /// Returns true if the option is currently enabled.
    pub fn is_enabled(&self) -> bool {
        matches!(self, OptionState::Enabled)
    }
    
    /// Returns true if we want to enable the option.
    pub fn wants_enable(&self) -> bool {
        matches!(self, OptionState::WantsEnable)
    }
    
    /// Returns true if we want to disable the option.
    pub fn wants_disable(&self) -> bool {
        matches!(self, OptionState::WantsDisable)
    }
}

/// Client connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    
    /// Connecting to server
    Connecting,
    
    /// Connected to server
    Connected,
    
    /// Negotiating options
    Negotiating,
    
    /// Closing connection
    Closing,
}

/// TELNET client configuration.
#[derive(Debug, Clone)]
pub struct TelnetConfig {
    /// Server hostname or IP
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Connection timeout
    pub timeout: std::time::Duration,
    
    /// Request ECHO option
    pub enable_echo: bool,
    
    /// Request BINARY option
    pub enable_binary: bool,
    
    /// Request SUPPRESS-GO-AHEAD option
    pub enable_suppress_ga: bool,
    
    /// Terminal type to advertise
    pub terminal_type: Option<String>,
    
    /// Client window size (for NAWS)
    pub window_width: u16,
    pub window_height: u16,
}

impl Default for TelnetConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 23,
            timeout: std::time::Duration::from_secs(30),
            enable_echo: true,
            enable_binary: true,
            enable_suppress_ga: true,
            terminal_type: Some(String::from("ANSI")),
            window_width: 80,
            window_height: 24,
        }
    }
}

impl TelnetConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the host.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }
    
    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Set the timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Request ECHO option.
    pub fn with_echo(mut self, enable: bool) -> Self {
        self.enable_echo = enable;
        self
    }
    
    /// Request BINARY option.
    pub fn with_binary(mut self, enable: bool) -> Self {
        self.enable_binary = enable;
        self
    }
    
    /// Request SUPPRESS-GO-AHEAD option.
    pub fn with_suppress_ga(mut self, enable: bool) -> Self {
        self.enable_suppress_ga = enable;
        self
    }
    
    /// Set terminal type.
    pub fn with_terminal_type(mut self, ty: &str) -> Self {
        self.terminal_type = Some(ty.to_string());
        self
    }
}

/// Events emitted by the TELNET client.
#[derive(Debug)]
pub enum TelnetEvent {
    /// Received data from server
    Data(Vec<u8>),
    
    /// Received TELNET command
    Command(TelnetCommand),
    
    /// Option was negotiated
    OptionNegotiated {
        /// Option code
        option: u8,
        /// Whether option was enabled
        enabled: bool,
    },
    
    /// Connection closed
    Closed,
    
    /// Error occurred
    Error(crate::error::TelnetError),
}

/// Client state tracking current connection status.
#[derive(Debug, Clone)]
pub struct ClientState {
    /// Current connection state
    pub connection_state: ConnectionState,
    
    /// State of each negotiated option
    pub options: std::collections::HashMap<u8, OptionState>,
    
    /// Whether local echo is enabled
    pub local_echo: bool,
    
    /// Whether remote echo is enabled
    pub remote_echo: bool,
    
    /// Whether binary mode is enabled
    pub binary_mode: bool,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            options: std::collections::HashMap::new(),
            local_echo: false,
            remote_echo: false,
            binary_mode: false,
        }
    }
}

impl ClientState {
    /// Get the state of a specific option.
    pub fn get_option(&self, option: u8) -> OptionState {
        *self.options.get(&option).unwrap_or(&OptionState::Closed)
    }
    
    /// Set the state of a specific option.
    pub fn set_option(&mut self, option: u8, state: OptionState) {
        self.options.insert(option, state);
    }
    
    /// Check if an option is enabled.
    pub fn is_option_enabled(&self, option: u8) -> bool {
        self.get_option(option).is_enabled()
    }
}