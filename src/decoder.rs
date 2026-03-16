//! TELNET data decoder.
//!
//! This module handles decoding of TELNET data streams,
//! parsing commands and extracting application data.

#![deny(unused_must_use)]

use crate::protocol::*;
use crate::types::TelnetCommand;
use tracing::debug;

/// States for the TELNET decoder state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecodeState {
    /// Normal data receiving state
    Normal,
    
    /// Received IAC, expecting command
    Iac,
    
    /// Received DO, expecting option code
    Do,
    
    /// Received DONT, expecting option code
    Dont,
    
    /// Received WILL, expecting option code
    Will,
    
    /// Received WONT, expecting option code
    Wont,
    
    /// Received SB, expecting option code and data
    Sb,
    
    /// Collecting subnegotiation data
    SbData,
    
    /// Received SE within subnegotiation
    SbSe,
}

impl Default for DecodeState {
    fn default() -> Self {
        DecodeState::Normal
    }
}

/// TELNET data decoder.
///
/// Decodes a stream of bytes into TELNET commands.
/// Handles IAC byte escaping and command parsing.
#[derive(Debug, Clone, Default)]
pub struct TelnetDecoder {
    state: DecodeState,
    pending_option: Option<u8>,
    sb_data: Vec<u8>,
}

impl TelnetDecoder {
    /// Create a new decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode a single byte, returning any completed command.
    pub fn decode_byte(&mut self, byte: u8) -> Option<TelnetCommand> {
        // Create human-readable representation
        let readable = if byte >= 32 && byte <= 126 {
            // Printable ASCII character
            format!("'{}' (0x{:02x})", byte as char, byte)
        } else if byte == 10 {
            "LF (\\n, 0x0a)".to_string()
        } else if byte == 13 {
            "CR (\\r, 0x0d)".to_string()
        } else if byte == 0 {
            "NUL (0x00)".to_string()
        } else {
            // Non-printable control character
            format!("0x{:02x} (control)", byte)
        };
        
        debug!("Decoding byte: {} [state: {:?}]", readable, self.state);
        
        match self.state {
            DecodeState::Normal => {
                if byte == IAC {
                    self.state = DecodeState::Iac;
                    debug!("  -> IAC received, transitioning to Iac state");
                    None
                } else {
                    debug!("  -> Data byte: {}", readable);
                    Some(TelnetCommand::Data(byte))
                }
            }
            
            DecodeState::Iac => {
                self.state = DecodeState::Normal;
                let cmd = match byte {
                    DO => {
                        debug!("  -> DO command received, waiting for option code");
                        self.state = DecodeState::Do;
                        None
                    }
                    DONT => {
                        debug!("  -> DONT command received, waiting for option code");
                        self.state = DecodeState::Dont;
                        None
                    }
                    WILL => {
                        debug!("  -> WILL command received, waiting for option code");
                        self.state = DecodeState::Will;
                        None
                    }
                    WONT => {
                        debug!("  -> WONT command received, waiting for option code");
                        self.state = DecodeState::Wont;
                        None
                    }
                    SB => {
                        debug!("  -> SB (subnegotiation) command received, waiting for option code");
                        self.state = DecodeState::Sb;
                        None
                    }
                    SE => {
                        // Unexpected SE, treat as NOP
                        debug!("  -> SE (unexpected), treating as NOP");
                        Some(TelnetCommand::Nop)
                    }
                    NOP => {
                        debug!("  -> NOP command");
                        Some(TelnetCommand::Nop)
                    }
                    GA => {
                        debug!("  -> GA (Go Ahead) command");
                        Some(TelnetCommand::GoAhead)
                    }
                    AO => {
                        debug!("  -> AO (Abort Output) command");
                        Some(TelnetCommand::AbortOutput)
                    }
                    AYT => {
                        debug!("  -> AYT (Are You There) command");
                        Some(TelnetCommand::AreYouThere)
                    }
                    EC => {
                        debug!("  -> EC (Erase Character) command");
                        Some(TelnetCommand::EraseCharacter)
                    }
                    EL => {
                        debug!("  -> EL (Erase Line) command");
                        Some(TelnetCommand::EraseLine)
                    }
                    IP => {
                        debug!("  -> IP (Interrupt Process) command");
                        Some(TelnetCommand::InterruptProcess)
                    }
                    BRK => {
                        debug!("  -> BRK (Break) command");
                        Some(TelnetCommand::Break)
                    }
                    DM => {
                        debug!("  -> DM (Data Mark) command");
                        Some(TelnetCommand::DataMark)
                    }
                    EOF => {
                        debug!("  -> EOF (End of File) command");
                        Some(TelnetCommand::EndOfFile)
                    }
                    SUSP => {
                        debug!("  -> SUSP (Suspend) command");
                        Some(TelnetCommand::Suspend)
                    }
                    ABOR => {
                        debug!("  -> ABOR (Abort) command");
                        Some(TelnetCommand::Abort)
                    }
                    EOR => {
                        debug!("  -> EOR (End of Record) command");
                        Some(TelnetCommand::EndOfRecord)
                    }
                    IAC => {
                        // IAC IAC represents a literal IAC byte
                        debug!("  -> IAC IAC (literal IAC byte)");
                        Some(TelnetCommand::Data(IAC))
                    }
                    _ => {
                        // Unknown command, treat as data
                        debug!("  -> Unknown command 0x{:02x}, treating as data", byte);
                        Some(TelnetCommand::Data(byte))
                    }
                };
                cmd
            }
            
            DecodeState::Do => {
                let cmd = TelnetCommand::Do(byte);
                debug!("  -> DO command completed: option 0x{:02x}", byte);
                self.state = DecodeState::Normal;
                Some(cmd)
            }
            
            DecodeState::Dont => {
                let cmd = TelnetCommand::Dont(byte);
                debug!("  -> DONT command completed: option 0x{:02x}", byte);
                self.state = DecodeState::Normal;
                Some(cmd)
            }
            
            DecodeState::Will => {
                let cmd = TelnetCommand::Will(byte);
                debug!("  -> WILL command completed: option 0x{:02x}", byte);
                self.state = DecodeState::Normal;
                Some(cmd)
            }
            
            DecodeState::Wont => {
                let cmd = TelnetCommand::Wont(byte);
                debug!("  -> WONT command completed: option 0x{:02x}", byte);
                self.state = DecodeState::Normal;
                Some(cmd)
            }
            
            DecodeState::Sb => {
                debug!("  -> SB command completed: option 0x{:02x}, starting data collection", byte);
                self.state = DecodeState::SbData;
                self.sb_data.clear();
                self.sb_data.push(byte);
                None
            }
            
            DecodeState::SbData => {
                if byte == IAC {
                    // Could be start of SE or IAC escape
                    debug!("  -> SB data: IAC received, waiting for SE or IAC escape");
                    self.state = DecodeState::SbSe;
                    None
                } else if byte == SE {
                    // Unexpected SE without IAC, treat as data
                    debug!("  -> SB data: unexpected SE without IAC");
                    self.sb_data.push(byte);
                    None
                } else {
                    debug!("  -> SB data byte: {}", readable);
                    self.sb_data.push(byte);
                    None
                }
            }
            
            DecodeState::SbSe => {
                if byte == SE {
                    // End of subnegotiation
                    debug!("  -> SB completed: option 0x{:02x}, {} bytes of data", 
                           self.sb_data[0], self.sb_data.len() - 1);
                    let cmd = TelnetCommand::Subnegotiation {
                        option: self.sb_data[0],
                        data: self.sb_data[1..].to_vec(),
                    };
                    self.state = DecodeState::Normal;
                    self.sb_data.clear();
                    Some(cmd)
                } else if byte == IAC {
                    // IAC IAC in subnegotiation represents a literal IAC byte
                    debug!("  -> SB data: IAC IAC (literal IAC byte)");
                    self.sb_data.push(IAC);
                    // Return to SbData since the IAC escape is complete
                    self.state = DecodeState::SbData;
                    None
                } else {
                    // Unexpected byte after IAC, treat as data
                    debug!("  -> SB data: unexpected byte after IAC, treating as data");
                    self.sb_data.push(byte);
                    self.state = DecodeState::SbData;
                    None
                }
            }
        }
    }

    /// Decode a stream of bytes, returning all completed commands.
    pub fn decode(&mut self, bytes: &[u8]) -> Vec<TelnetCommand> {
        let mut commands = Vec::new();
        for &byte in bytes {
            if let Some(cmd) = self.decode_byte(byte) {
                commands.push(cmd);
            }
        }
        commands
    }

    /// Reset the decoder state.
    pub fn reset(&mut self) {
        self.state = DecodeState::Normal;
        self.pending_option = None;
        self.sb_data.clear();
    }

    /// Get the current decoder state.
    #[allow(dead_code)]
    pub(crate) fn state(&self) -> DecodeState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_normal_data() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(65), Some(TelnetCommand::Data(65)));
        assert_eq!(decoder.decode_byte(0), Some(TelnetCommand::Data(0)));
        assert_eq!(decoder.decode_byte(254), Some(TelnetCommand::Data(254)));
    }

    #[test]
    fn test_decode_iac_byte() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(65), Some(TelnetCommand::Data(65)));
    }

    #[test]
    fn test_decode_iac_iac() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(IAC), Some(TelnetCommand::Data(IAC)));
    }

    #[test]
    fn test_decode_do() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(DO), None);
        assert_eq!(decoder.decode_byte(OPT_ECHO), Some(TelnetCommand::Do(OPT_ECHO)));
    }

    #[test]
    fn test_decode_dont() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(DONT), None);
        assert_eq!(decoder.decode_byte(OPT_ECHO), Some(TelnetCommand::Dont(OPT_ECHO)));
    }

    #[test]
    fn test_decode_will() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(WILL), None);
        assert_eq!(decoder.decode_byte(OPT_ECHO), Some(TelnetCommand::Will(OPT_ECHO)));
    }

    #[test]
    fn test_decode_wont() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(WONT), None);
        assert_eq!(decoder.decode_byte(OPT_ECHO), Some(TelnetCommand::Wont(OPT_ECHO)));
    }

    #[test]
    fn test_decode_nop() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(NOP), Some(TelnetCommand::Nop));
    }

    #[test]
    fn test_decode_ga() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(GA), Some(TelnetCommand::GoAhead));
    }

    #[test]
    fn test_decode_subnegotiation() {
        let mut decoder = TelnetDecoder::new();
        
        // SB TERMINAL-TYPE
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(SB), None);
        assert_eq!(decoder.decode_byte(OPT_TERMINAL_TYPE), None);
        
        // Data
        assert_eq!(decoder.decode_byte(1), None);
        assert_eq!(decoder.decode_byte(2), None);
        assert_eq!(decoder.decode_byte(3), None);
        
        // SE
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(SE), Some(TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![1, 2, 3],
        }));
    }

    #[test]
    fn test_decode_subnegotiation_with_iac_in_data() {
        let mut decoder = TelnetDecoder::new();
        
        // SB TERMINAL-TYPE
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(SB), None);
        assert_eq!(decoder.decode_byte(OPT_TERMINAL_TYPE), None);
        
        // Data with IAC - IAC IAC in subnegotiation represents literal IAC byte
        assert_eq!(decoder.decode_byte(1), None);
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(IAC), None); // IAC IAC = literal IAC
        assert_eq!(decoder.decode_byte(2), None);
        assert_eq!(decoder.decode_byte(3), None);
        
        // SE
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(SE), Some(TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![1, IAC, 2, 3],
        }));
    }

    #[test]
    fn test_decode_full_stream() {
        let mut decoder = TelnetDecoder::new();
        let bytes = vec![
            IAC, DO, OPT_ECHO,
            65, 66, 67,
            IAC, WILL, OPT_ECHO,
            IAC, IAC,
        ];
        
        let commands = decoder.decode(&bytes);
        
        // IAC IAC at end is a data byte, so we get 6 commands
        assert_eq!(commands.len(), 6);
        assert_eq!(commands[0], TelnetCommand::Do(OPT_ECHO));
        assert_eq!(commands[1], TelnetCommand::Data(65));
        assert_eq!(commands[2], TelnetCommand::Data(66));
        assert_eq!(commands[3], TelnetCommand::Data(67));
        assert_eq!(commands[4], TelnetCommand::Will(OPT_ECHO));
        assert_eq!(commands[5], TelnetCommand::Data(IAC));
    }

    #[test]
    fn test_decode_reset() {
        let mut decoder = TelnetDecoder::new();
        decoder.decode_byte(IAC);
        assert_eq!(decoder.state(), DecodeState::Iac);
        
        decoder.reset();
        assert_eq!(decoder.state(), DecodeState::Normal);
    }

    #[test]
    fn test_decode_ao() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(AO), Some(TelnetCommand::AbortOutput));
    }

    #[test]
    fn test_decode_ayt() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(AYT), Some(TelnetCommand::AreYouThere));
    }

    #[test]
    fn test_decode_unknown_command() {
        let mut decoder = TelnetDecoder::new();
        assert_eq!(decoder.decode_byte(IAC), None);
        assert_eq!(decoder.decode_byte(200), Some(TelnetCommand::Data(200)));
    }
}