//! Tests for decoder state persistence across multiple decode calls.
//!
//! This module verifies that the decoder correctly handles TELNET commands
//! that span multiple input chunks, ensuring proper state machine behavior.

#![deny(unused_must_use)]

#[cfg(test)]
mod tests {
    use crate::decoder::TelnetDecoder;
    use crate::protocol::*;
    use crate::types::TelnetCommand;

    /// Test that decoder maintains state when command spans multiple decode calls
    #[test]
    fn test_decode_state_persistence() {
        let mut decoder = TelnetDecoder::new();
        
        // Simulate receiving IAC and DO separately
        assert_eq!(decoder.decode_byte(IAC), None); // State: Iac
        assert_eq!(decoder.decode_byte(DO), None);  // State: Do
        
        // Now receive the option byte
        assert_eq!(decoder.decode_byte(OPT_ECHO), Some(TelnetCommand::Do(OPT_ECHO)));
        
        // Verify decoder is back to normal state
        assert_eq!(decoder.decode_byte(65), Some(TelnetCommand::Data(65)));
    }

    /// Test command split across multiple decode calls
    #[test]
    fn test_command_split_across_calls() {
        let mut decoder = TelnetDecoder::new();
        
        // First chunk: IAC DO
        decoder.decode_byte(IAC);
        decoder.decode_byte(DO);
        
        // Second chunk: option byte
        let commands = decoder.decode(&[OPT_ECHO]);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], TelnetCommand::Do(OPT_ECHO));
        
        // Third chunk: more data
        let commands = decoder.decode(&[65, 66]);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], TelnetCommand::Data(65));
        assert_eq!(commands[1], TelnetCommand::Data(66));
    }

    /// Test subnegotiation spanning multiple chunks
    #[test]
    fn test_subnegotiation_split() {
        let mut decoder = TelnetDecoder::new();
        
        // First chunk: IAC SB TERMINAL-TYPE
        // IAC -> state=Iac
        // SB -> state=Sb
        // TERMINAL-TYPE -> state=SbData, sb_data=[TERMINAL-TYPE]
        decoder.decode_byte(IAC);
        decoder.decode_byte(SB);
        decoder.decode_byte(OPT_TERMINAL_TYPE);
        
        // Second chunk: some data
        let commands = decoder.decode(&[1, 2, 3]);
        assert_eq!(commands.len(), 0); // Still collecting data
        
        // Third chunk: more data + IAC (start of escape)
        let commands = decoder.decode(&[4, IAC]);
        assert_eq!(commands.len(), 0); // Still waiting for SE or IAC
        
        // Fourth chunk: IAC (literal escape) + IAC SE (end marker)
        // IAC IAC = literal IAC byte, then IAC SE ends the subnegotiation
        let commands = decoder.decode(&[IAC, IAC, SE]);
        assert_eq!(commands.len(), 1);
        // sb_data contains: [TERMINAL-TYPE, 1, 2, 3, 4, IAC]
        // After extracting option (first byte), data is [1, 2, 3, 4, IAC]
        assert_eq!(commands[0], TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![1, 2, 3, 4, IAC],
        });
    }

    /// Test command split at different points
    #[test]
    fn test_multiple_split_scenarios() {
        // Scenario 1: Split after IAC
        let mut decoder1 = TelnetDecoder::new();
        decoder1.decode_byte(IAC);
        let result = decoder1.decode(&[DO, OPT_ECHO]);
        assert_eq!(result, vec![TelnetCommand::Do(OPT_ECHO)]);
        
        // Scenario 2: Split after IAC DO
        let mut decoder2 = TelnetDecoder::new();
        decoder2.decode_byte(IAC);
        decoder2.decode_byte(DO);
        let result = decoder2.decode(&[OPT_ECHO]);
        assert_eq!(result, vec![TelnetCommand::Do(OPT_ECHO)]);
        
        // Scenario 3: Split in middle of subnegotiation
        let mut decoder3 = TelnetDecoder::new();
        decoder3.decode_byte(IAC);
        decoder3.decode_byte(SB);
        decoder3.decode_byte(OPT_TERMINAL_TYPE);
        decoder3.decode_byte(1);
        decoder3.decode_byte(2);
        
        let result = decoder3.decode(&[3, 4, IAC, SE]);
        assert_eq!(result.len(), 1);
        if let TelnetCommand::Subnegotiation { option, data } = &result[0] {
            assert_eq!(*option, OPT_TERMINAL_TYPE);
            assert_eq!(*data, vec![1, 2, 3, 4]);
        } else {
            panic!("Expected Subnegotiation command");
        }
    }

    /// Test that decoder state is preserved through multiple read operations
    #[tokio::test]
    async fn test_decoder_state_across_multiple_reads() {
        let mut decoder = TelnetDecoder::new();
        
        // Simulate first read: IAC DO
        decoder.decode_byte(IAC);
        decoder.decode_byte(DO);
        
        // Simulate second read: option byte
        let result = decoder.decode(&[OPT_ECHO]);
        assert_eq!(result.len(), 1);
        
        // Simulate third read: data
        let result = decoder.decode(&[65, 66, 67]);
        assert_eq!(result.len(), 3);
        
        // Simulate fourth read: IAC WILL
        decoder.decode_byte(IAC);
        decoder.decode_byte(WILL);
        
        // Simulate fifth read: option byte
        let result = decoder.decode(&[OPT_BINARY]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TelnetCommand::Will(OPT_BINARY));
    }

    /// Test complex multi-command stream split across reads
    #[tokio::test]
    async fn test_complex_split_stream() {
        let mut decoder = TelnetDecoder::new();
        
        // First read: partial commands
        let chunk1 = vec![IAC, DO, OPT_ECHO, 65, IAC];
        let result1 = decoder.decode(&chunk1);
        assert_eq!(result1.len(), 2); // Do(ECHO), Data(65)
        
        // Second read: continues IAC
        let chunk2 = vec![WILL, OPT_BINARY];
        let result2 = decoder.decode(&chunk2);
        assert_eq!(result2.len(), 1); // Will(BINARY)
        
        // Third read: complete command
        let chunk3 = vec![66, 67];
        let result3 = decoder.decode(&chunk3);
        assert_eq!(result3.len(), 2); // Data(66), Data(67)
    }
}