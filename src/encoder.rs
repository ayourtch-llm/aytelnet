//! TELNET data encoder.
//!
//! This module handles encoding of TELNET data and commands,
//! including IAC byte escaping.

use crate::protocol::*;
use crate::types::TelnetCommand;

/// TELNET data encoder.
///
/// Handles encoding of data bytes and TELNET commands with proper
/// IAC byte escaping.
#[derive(Debug, Clone, Default)]
pub struct TelnetEncoder;

impl TelnetEncoder {
    /// Create a new encoder.
    pub fn new() -> Self {
        Self
    }

    /// Encode a single data byte, escaping IAC if necessary.
    ///
    /// If the byte is IAC (255), it is doubled to IAC IAC.
    /// Otherwise, the byte is returned as-is.
    pub fn encode_byte(byte: u8) -> Vec<u8> {
        if byte == IAC {
            vec![IAC, IAC]
        } else {
            vec![byte]
        }
    }

    /// Encode a buffer of data bytes, escaping all IAC bytes.
    pub fn encode_data(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len() * 2);
        for &byte in data {
            result.extend_from_slice(&Self::encode_byte(byte));
        }
        result
    }

    /// Encode a TELNET command.
    pub fn encode_command(command: &TelnetCommand) -> Vec<u8> {
        match command {
            TelnetCommand::Do(opt) => vec![IAC, DO, *opt],
            TelnetCommand::Dont(opt) => vec![IAC, DONT, *opt],
            TelnetCommand::Will(opt) => vec![IAC, WILL, *opt],
            TelnetCommand::Wont(opt) => vec![IAC, WONT, *opt],
            TelnetCommand::Subnegotiation { option, data } => {
                let mut result = Vec::with_capacity(3 + data.len() + 2);
                result.push(IAC);
                result.push(SB);
                result.push(*option);
                result.extend_from_slice(data);
                result.push(IAC);
                result.push(SE);
                result
            }
            TelnetCommand::Nop => vec![IAC, NOP],
            TelnetCommand::GoAhead => vec![IAC, GA],
            TelnetCommand::AbortOutput => vec![IAC, AO],
            TelnetCommand::AreYouThere => vec![IAC, AYT],
            TelnetCommand::EraseCharacter => vec![IAC, EC],
            TelnetCommand::EraseLine => vec![IAC, EL],
            TelnetCommand::InterruptProcess => vec![IAC, IP],
            TelnetCommand::Break => vec![IAC, BRK],
            TelnetCommand::DataMark => vec![IAC, DM],
            TelnetCommand::EndOfRecord => vec![IAC, EOR],
            TelnetCommand::EndOfFile => vec![IAC, EOF],
            TelnetCommand::Suspend => vec![IAC, SUSP],
            TelnetCommand::Abort => vec![IAC, ABOR],
            TelnetCommand::Data(byte) => Self::encode_byte(*byte),
        }
    }

    /// Encode a list of commands into a single buffer.
    pub fn encode_commands(commands: &[TelnetCommand]) -> Vec<u8> {
        let mut result = Vec::new();
        for command in commands {
            result.extend_from_slice(&Self::encode_command(command));
        }
        result
    }

    /// Encode data with TELNET escaping.
    ///
    /// This is used when sending application data that might contain
    /// IAC bytes.
    pub fn encode_with_telnet_escaping(data: &[u8]) -> Vec<u8> {
        Self::encode_data(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_byte_normal() {
        assert_eq!(TelnetEncoder::encode_byte(65), vec![65]);
        assert_eq!(TelnetEncoder::encode_byte(0), vec![0]);
        assert_eq!(TelnetEncoder::encode_byte(254), vec![254]);
    }

    #[test]
    fn test_encode_byte_iac() {
        assert_eq!(TelnetEncoder::encode_byte(IAC), vec![IAC, IAC]);
    }

    #[test]
    fn test_encode_data_no_iac() {
        let data = vec![65, 66, 67];
        assert_eq!(TelnetEncoder::encode_data(&data), vec![65, 66, 67]);
    }

    #[test]
    fn test_encode_data_with_iac() {
        let data = vec![65, IAC, 67];
        assert_eq!(TelnetEncoder::encode_data(&data), vec![65, IAC, IAC, 67]);
    }

    #[test]
    fn test_encode_data_multiple_iac() {
        let data = vec![IAC, IAC, IAC];
        assert_eq!(TelnetEncoder::encode_data(&data), vec![IAC, IAC, IAC, IAC, IAC, IAC]);
    }

    #[test]
    fn test_encode_command_do() {
        let cmd = TelnetCommand::Do(OPT_ECHO);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, DO, OPT_ECHO]);
    }

    #[test]
    fn test_encode_command_dont() {
        let cmd = TelnetCommand::Dont(OPT_ECHO);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, DONT, OPT_ECHO]);
    }

    #[test]
    fn test_encode_command_will() {
        let cmd = TelnetCommand::Will(OPT_ECHO);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, WILL, OPT_ECHO]);
    }

    #[test]
    fn test_encode_command_wont() {
        let cmd = TelnetCommand::Wont(OPT_ECHO);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, WONT, OPT_ECHO]);
    }

    #[test]
    fn test_encode_command_subnegotiation() {
        let cmd = TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![1, 2, 3],
        };
        let expected = vec![IAC, SB, OPT_TERMINAL_TYPE, 1, 2, 3, IAC, SE];
        assert_eq!(TelnetEncoder::encode_command(&cmd), expected);
    }

    #[test]
    fn test_encode_command_nop() {
        let cmd = TelnetCommand::Nop;
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, NOP]);
    }

    #[test]
    fn test_encode_command_data() {
        let cmd = TelnetCommand::Data(65);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![65]);
    }

    #[test]
    fn test_encode_command_data_iac() {
        let cmd = TelnetCommand::Data(IAC);
        assert_eq!(TelnetEncoder::encode_command(&cmd), vec![IAC, IAC]);
    }

    #[test]
    fn test_encode_commands() {
        let commands = vec![
            TelnetCommand::Will(OPT_ECHO),
            TelnetCommand::Do(OPT_ECHO),
        ];
        let expected = vec![IAC, WILL, OPT_ECHO, IAC, DO, OPT_ECHO];
        assert_eq!(TelnetEncoder::encode_commands(&commands), expected);
    }
}