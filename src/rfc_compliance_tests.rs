//! Comprehensive RFC compliance tests.
//!
//! Tests verify correct implementation of TELNET protocol per RFC 854, 855,
//! 856, 857, 858, 859, and 860.

#![allow(unused_must_use)]

#[cfg(test)]
mod tests {
    use crate::decoder::TelnetDecoder;
    use crate::encoder::TelnetEncoder;
    use crate::options::OptionNegotiator;
    use crate::protocol::*;
    use crate::types::{OptionState, TelnetCommand};

    // =========================================================================
    // 1. Protocol Constant Value Tests (RFC 854)
    // =========================================================================

    /// RFC 854: IAC (Interpret As Command) must be 255 (0xFF).
    #[test]
    fn test_iac_value() {
        assert_eq!(IAC, 255, "IAC must be 0xFF per RFC 854");
        assert_eq!(IAC, 0xFF);
    }

    /// RFC 854: WILL must be 251 (0xFB).
    #[test]
    fn test_will_value() {
        assert_eq!(WILL, 251, "WILL must be 0xFB per RFC 854");
        assert_eq!(WILL, 0xFB);
    }

    /// RFC 854: WONT must be 252 (0xFC).
    #[test]
    fn test_wont_value() {
        assert_eq!(WONT, 252, "WONT must be 0xFC per RFC 854");
        assert_eq!(WONT, 0xFC);
    }

    /// RFC 854: DO must be 253 (0xFD).
    #[test]
    fn test_do_value() {
        assert_eq!(DO, 253, "DO must be 0xFD per RFC 854");
        assert_eq!(DO, 0xFD);
    }

    /// RFC 854: DONT must be 254 (0xFE).
    #[test]
    fn test_dont_value() {
        assert_eq!(DONT, 254, "DONT must be 0xFE per RFC 854");
        assert_eq!(DONT, 0xFE);
    }

    /// RFC 854: SB (Subnegotiation Begin) must be 250 (0xFA).
    #[test]
    fn test_sb_value() {
        assert_eq!(SB, 250, "SB must be 0xFA per RFC 854");
        assert_eq!(SB, 0xFA);
    }

    /// RFC 854: SE (Subnegotiation End) must be 240 (0xF0).
    #[test]
    fn test_se_value() {
        assert_eq!(SE, 240, "SE must be 0xF0 per RFC 854");
        assert_eq!(SE, 0xF0);
    }

    /// RFC 854: NOP must be 241 (0xF1).
    #[test]
    fn test_nop_value() {
        assert_eq!(NOP, 241, "NOP must be 0xF1 per RFC 854");
    }

    /// RFC 854: DM (Data Mark) must be 242 (0xF2).
    #[test]
    fn test_dm_value() {
        assert_eq!(DM, 242, "DM must be 0xF2 per RFC 854");
    }

    /// RFC 854: BRK (Break) must be 243 (0xF3).
    #[test]
    fn test_brk_value() {
        assert_eq!(BRK, 243, "BRK must be 0xF3 per RFC 854");
    }

    /// RFC 854: IP (Interrupt Process) must be 244 (0xF4).
    #[test]
    fn test_ip_value() {
        assert_eq!(IP, 244, "IP must be 0xF4 per RFC 854");
    }

    /// RFC 854: AO (Abort Output) must be 245 (0xF5).
    #[test]
    fn test_ao_value() {
        assert_eq!(AO, 245, "AO must be 0xF5 per RFC 854");
    }

    /// RFC 854: AYT (Are You There) must be 246 (0xF6).
    #[test]
    fn test_ayt_value() {
        assert_eq!(AYT, 246, "AYT must be 0xF6 per RFC 854");
    }

    /// RFC 854: EC (Erase Character) must be 247 (0xF7).
    #[test]
    fn test_ec_value() {
        assert_eq!(EC, 247, "EC must be 0xF7 per RFC 854");
    }

    /// RFC 854: EL (Erase Line) must be 248 (0xF8).
    #[test]
    fn test_el_value() {
        assert_eq!(EL, 248, "EL must be 0xF8 per RFC 854");
    }

    /// RFC 854: GA (Go Ahead) must be 249 (0xF9).
    #[test]
    fn test_ga_value() {
        assert_eq!(GA, 249, "GA must be 0xF9 per RFC 854");
    }

    /// RFC 885: EOR (End of Record) must be 239 (0xEF).
    #[test]
    fn test_eor_value() {
        assert_eq!(EOR, 239, "EOR must be 0xEF per RFC 885");
    }

    /// EOF must be 236 (0xEC).
    #[test]
    fn test_eof_value() {
        assert_eq!(EOF, 236, "EOF must be 0xEC");
    }

    /// SUSP must be 237 (0xED).
    #[test]
    fn test_susp_value() {
        assert_eq!(SUSP, 237, "SUSP must be 0xED");
    }

    /// ABOR must be 238 (0xEE).
    #[test]
    fn test_abor_value() {
        assert_eq!(ABOR, 238, "ABOR must be 0xEE");
    }

    /// RFC 854: Command values must be in descending order from IAC.
    #[test]
    fn test_command_value_ordering() {
        assert!(IAC > DONT, "IAC > DONT");
        assert!(DONT > DO, "DONT > DO");
        assert!(DO > WONT, "DO > WONT");
        assert!(WONT > WILL, "WONT > WILL");
        assert!(WILL > SB, "WILL > SB");
        assert!(SB > GA, "SB > GA");
    }

    // =========================================================================
    // 2. Option Code Tests (RFC 855, 856, 857, 858, 859, 860)
    // =========================================================================

    /// RFC 856: TRANSMIT-BINARY option code must be 0.
    #[test]
    fn test_opt_binary_value() {
        assert_eq!(OPT_BINARY, 0, "TRANSMIT-BINARY must be 0 per RFC 856");
    }

    /// RFC 857: ECHO option code must be 1.
    #[test]
    fn test_opt_echo_value() {
        assert_eq!(OPT_ECHO, 1, "ECHO must be 1 per RFC 857");
    }

    /// RFC 858: SUPPRESS-GO-AHEAD option code must be 3.
    #[test]
    fn test_opt_suppress_ga_value() {
        assert_eq!(OPT_SUPPRESS_GA, 3, "SUPPRESS-GO-AHEAD must be 3 per RFC 858");
    }

    /// RFC 859: STATUS option code must be 5.
    #[test]
    fn test_opt_status_value() {
        assert_eq!(OPT_STATUS, 5, "STATUS must be 5 per RFC 859");
    }

    /// RFC 860: TIMING-MARK option code must be 6.
    #[test]
    fn test_opt_timing_mark_value() {
        assert_eq!(OPT_TIMING_MARK, 6, "TIMING-MARK must be 6 per RFC 860");
    }

    /// TERMINAL-TYPE option code must be 24.
    #[test]
    fn test_opt_terminal_type_value() {
        assert_eq!(OPT_TERMINAL_TYPE, 24, "TERMINAL-TYPE must be 24");
    }

    /// NAWS (Negotiate About Window Size) option code must be 31.
    #[test]
    fn test_opt_naws_value() {
        assert_eq!(OPT_NAWS, 31, "NAWS must be 31");
    }

    /// LINEMODE option code must be 34.
    #[test]
    fn test_opt_linemode_value() {
        assert_eq!(OPT_LINEMODE, 34, "LINEMODE must be 34");
    }

    /// Option codes must be distinct.
    #[test]
    fn test_option_codes_are_distinct() {
        let codes = [
            OPT_BINARY,
            OPT_ECHO,
            OPT_SUPPRESS_GA,
            OPT_STATUS,
            OPT_TIMING_MARK,
            OPT_TERMINAL_TYPE,
            OPT_NAWS,
            OPT_LINEMODE,
        ];
        for i in 0..codes.len() {
            for j in (i + 1)..codes.len() {
                assert_ne!(
                    codes[i], codes[j],
                    "Option codes at index {} and {} must be distinct",
                    i, j
                );
            }
        }
    }

    // =========================================================================
    // 3. Decoder Tests - IAC Escaping (RFC 854)
    // =========================================================================

    /// RFC 854: IAC IAC must produce a literal 0xFF data byte.
    #[test]
    fn test_decoder_iac_iac_produces_literal_0xff() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, IAC]);
        assert_eq!(cmds.len(), 1, "IAC IAC should produce exactly one command");
        assert_eq!(
            cmds[0],
            TelnetCommand::Data(0xFF),
            "IAC IAC must produce Data(0xFF)"
        );
    }

    /// RFC 854: IAC followed by each valid command byte must decode correctly.
    #[test]
    fn test_decoder_iac_followed_by_valid_commands() {
        let cases: Vec<(u8, TelnetCommand)> = vec![
            (NOP, TelnetCommand::Nop),
            (GA, TelnetCommand::GoAhead),
            (AO, TelnetCommand::AbortOutput),
            (AYT, TelnetCommand::AreYouThere),
            (EC, TelnetCommand::EraseCharacter),
            (EL, TelnetCommand::EraseLine),
            (IP, TelnetCommand::InterruptProcess),
            (BRK, TelnetCommand::Break),
            (DM, TelnetCommand::DataMark),
            (EOR, TelnetCommand::EndOfRecord),
            (EOF, TelnetCommand::EndOfFile),
            (SUSP, TelnetCommand::Suspend),
            (ABOR, TelnetCommand::Abort),
        ];

        for (byte, expected) in cases {
            let mut decoder = TelnetDecoder::new();
            let cmds = decoder.decode(&[IAC, byte]);
            assert_eq!(
                cmds.len(),
                1,
                "IAC followed by 0x{:02X} should produce one command",
                byte
            );
            assert_eq!(
                cmds[0], expected,
                "IAC 0x{:02X} should decode to {:?}",
                byte, expected
            );
        }
    }

    /// RFC 854: All data bytes 0-254 must pass through as Data.
    #[test]
    fn test_decoder_all_data_bytes_pass_through() {
        for byte in 0u8..=254 {
            let mut decoder = TelnetDecoder::new();
            let result = decoder.decode_byte(byte);
            assert_eq!(
                result,
                Some(TelnetCommand::Data(byte)),
                "Byte {} (0x{:02X}) should pass through as Data",
                byte,
                byte
            );
        }
    }

    /// RFC 854: Byte 255 (IAC) must trigger IAC state, not produce data.
    #[test]
    fn test_decoder_byte_255_triggers_iac_state() {
        let mut decoder = TelnetDecoder::new();
        let result = decoder.decode_byte(255);
        assert_eq!(
            result, None,
            "Byte 255 must not produce a command; it enters IAC state"
        );
        assert_eq!(
            decoder.state(),
            crate::decoder::DecodeState::Iac,
            "After receiving IAC, decoder must be in Iac state"
        );
    }

    // =========================================================================
    // 4. Decoder Tests - Command Parsing
    // =========================================================================

    /// RFC 854: Each single-byte command type must decode correctly.
    #[test]
    fn test_decoder_all_single_byte_commands() {
        let commands: Vec<(u8, TelnetCommand)> = vec![
            (NOP, TelnetCommand::Nop),
            (GA, TelnetCommand::GoAhead),
            (AO, TelnetCommand::AbortOutput),
            (AYT, TelnetCommand::AreYouThere),
            (EC, TelnetCommand::EraseCharacter),
            (EL, TelnetCommand::EraseLine),
            (IP, TelnetCommand::InterruptProcess),
            (BRK, TelnetCommand::Break),
            (DM, TelnetCommand::DataMark),
            (EOR, TelnetCommand::EndOfRecord),
            (EOF, TelnetCommand::EndOfFile),
            (SUSP, TelnetCommand::Suspend),
            (ABOR, TelnetCommand::Abort),
        ];

        for (cmd_byte, expected) in commands {
            let mut decoder = TelnetDecoder::new();
            assert_eq!(decoder.decode_byte(IAC), None);
            assert_eq!(
                decoder.decode_byte(cmd_byte),
                Some(expected.clone()),
                "Command byte 0x{:02X} should decode to {:?}",
                cmd_byte,
                expected
            );
        }
    }

    /// RFC 854: DO with option code must parse correctly.
    #[test]
    fn test_decoder_do_with_option() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, DO, OPT_ECHO]);
        assert_eq!(cmds, vec![TelnetCommand::Do(OPT_ECHO)]);
    }

    /// RFC 854: DONT with option code must parse correctly.
    #[test]
    fn test_decoder_dont_with_option() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, DONT, OPT_ECHO]);
        assert_eq!(cmds, vec![TelnetCommand::Dont(OPT_ECHO)]);
    }

    /// RFC 854: WILL with option code must parse correctly.
    #[test]
    fn test_decoder_will_with_option() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, WILL, OPT_SUPPRESS_GA]);
        assert_eq!(cmds, vec![TelnetCommand::Will(OPT_SUPPRESS_GA)]);
    }

    /// RFC 854: WONT with option code must parse correctly.
    #[test]
    fn test_decoder_wont_with_option() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, WONT, OPT_SUPPRESS_GA]);
        assert_eq!(cmds, vec![TelnetCommand::Wont(OPT_SUPPRESS_GA)]);
    }

    /// RFC 854: DO/DONT/WILL/WONT must each correctly parse with various option codes.
    #[test]
    fn test_decoder_negotiation_commands_with_various_options() {
        let options = [OPT_BINARY, OPT_ECHO, OPT_SUPPRESS_GA, OPT_TERMINAL_TYPE, OPT_NAWS];
        let negotiation_bytes = [(DO, "Do"), (DONT, "Dont"), (WILL, "Will"), (WONT, "Wont")];

        for &opt in &options {
            for &(cmd_byte, name) in &negotiation_bytes {
                let mut decoder = TelnetDecoder::new();
                let cmds = decoder.decode(&[IAC, cmd_byte, opt]);
                assert_eq!(
                    cmds.len(),
                    1,
                    "{} with option {} should produce one command",
                    name,
                    opt
                );
                let expected = match cmd_byte {
                    DO => TelnetCommand::Do(opt),
                    DONT => TelnetCommand::Dont(opt),
                    WILL => TelnetCommand::Will(opt),
                    WONT => TelnetCommand::Wont(opt),
                    _ => unreachable!(),
                };
                assert_eq!(cmds[0], expected, "{} option {} mismatch", name, opt);
            }
        }
    }

    // =========================================================================
    // 5. Decoder Tests - Subnegotiation (RFC 855)
    // =========================================================================

    /// RFC 855: Basic subnegotiation IAC SB option data IAC SE.
    #[test]
    fn test_decoder_basic_subnegotiation() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, SB, OPT_TERMINAL_TYPE, 0, b'V', b'T', b'1', b'0', b'0', IAC, SE]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_TERMINAL_TYPE,
                data: vec![0, b'V', b'T', b'1', b'0', b'0'],
            }
        );
    }

    /// RFC 855: Subneg with IAC in data — IAC IAC produces literal 0xFF.
    #[test]
    fn test_decoder_subneg_iac_in_data() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[
            IAC, SB, OPT_NAWS, 0x01, IAC, IAC, 0x02, IAC, SE,
        ]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_NAWS,
                data: vec![0x01, 0xFF, 0x02],
            },
            "IAC IAC in subneg data must produce literal 0xFF"
        );
    }

    /// RFC 855: Empty subnegotiation — IAC SB option IAC SE (no data).
    #[test]
    fn test_decoder_empty_subnegotiation() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, SB, OPT_STATUS, IAC, SE]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_STATUS,
                data: vec![],
            },
            "Empty subneg should have empty data vec"
        );
    }

    /// RFC 855: After IAC IAC in subneg, SE byte (0xF0) is treated as data,
    /// not as end-of-subnegotiation. The IAC escape returns to SbData state,
    /// so a bare SE is just a data byte.
    #[test]
    fn test_decoder_subneg_se_after_iac_escape_is_data() {
        let mut decoder = TelnetDecoder::new();
        // Sequence: IAC SB opt <data> IAC IAC SE(0xF0) <more> IAC SE
        // The IAC IAC produces literal 0xFF, then SE(0xF0) is data,
        // then IAC SE ends the subneg.
        let cmds = decoder.decode(&[
            IAC, SB, OPT_NAWS, 0x42, IAC, IAC, SE, 0x43, IAC, SE,
        ]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_NAWS,
                data: vec![0x42, 0xFF, SE, 0x43],
            },
            "After IAC IAC escape, bare SE (0xF0) must be treated as data"
        );
    }

    /// RFC 855: Correct sequence to end subneg after IAC escape data:
    /// IAC IAC (literal 0xFF), then IAC SE (end subneg).
    #[test]
    fn test_decoder_subneg_end_after_iac_data() {
        let mut decoder = TelnetDecoder::new();
        // IAC SB opt IAC IAC IAC SE
        // = subneg with data containing literal 0xFF, then end
        let cmds = decoder.decode(&[IAC, SB, OPT_BINARY, IAC, IAC, IAC, SE]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_BINARY,
                data: vec![0xFF],
            },
            "IAC IAC followed by IAC SE should give data=[0xFF] then end subneg"
        );
    }

    /// RFC 855: Multiple IAC escapes in subneg data.
    #[test]
    fn test_decoder_subneg_multiple_iac_escapes() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[
            IAC, SB, OPT_NAWS, IAC, IAC, 0x01, IAC, IAC, IAC, SE,
        ]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_NAWS,
                data: vec![0xFF, 0x01, 0xFF],
            },
            "Multiple IAC IAC in subneg must each produce 0xFF"
        );
    }

    // =========================================================================
    // 6. Encoder Tests - IAC Escaping (RFC 854)
    // =========================================================================

    /// RFC 854: Encoding data byte 0xFF must produce IAC IAC.
    #[test]
    fn test_encoder_iac_byte_doubled() {
        let result = TelnetEncoder::encode_byte(0xFF);
        assert_eq!(result, vec![IAC, IAC], "0xFF must be encoded as IAC IAC");
    }

    /// RFC 854: Encoding data without 0xFF must remain unchanged.
    #[test]
    fn test_encoder_data_without_iac_unchanged() {
        let data = b"Hello, World!";
        let result = TelnetEncoder::encode_data(data);
        assert_eq!(result, data.to_vec(), "Data without IAC should be unchanged");
    }

    /// RFC 854: All data bytes 0-254 must encode unchanged (single byte).
    #[test]
    fn test_encoder_all_non_iac_bytes_unchanged() {
        for byte in 0u8..=254 {
            let result = TelnetEncoder::encode_byte(byte);
            assert_eq!(
                result,
                vec![byte],
                "Byte {} (0x{:02X}) should encode as itself",
                byte,
                byte
            );
        }
    }

    /// RFC 854: encode_data with embedded IAC bytes doubles each one.
    #[test]
    fn test_encoder_data_with_embedded_iac() {
        let data = [0x41, 0xFF, 0x42, 0xFF, 0x43];
        let result = TelnetEncoder::encode_data(&data);
        assert_eq!(
            result,
            vec![0x41, 0xFF, 0xFF, 0x42, 0xFF, 0xFF, 0x43],
            "Each IAC in data must be doubled"
        );
    }

    /// RFC 854: encode_data with all IAC bytes.
    #[test]
    fn test_encoder_data_all_iac() {
        let data = [0xFF, 0xFF, 0xFF];
        let result = TelnetEncoder::encode_data(&data);
        assert_eq!(
            result,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            "Three IAC bytes should become six bytes"
        );
    }

    /// RFC 854: Empty data encodes to empty output.
    #[test]
    fn test_encoder_empty_data() {
        let result = TelnetEncoder::encode_data(&[]);
        assert!(result.is_empty(), "Empty data should encode to empty output");
    }

    // =========================================================================
    // 7. Encoder Tests - Subnegotiation IAC Escaping (RFC 855)
    // =========================================================================

    /// RFC 855: Subneg data containing IAC byte must be doubled in output.
    #[test]
    fn test_encoder_subneg_iac_in_data_doubled() {
        let cmd = TelnetCommand::Subnegotiation {
            option: OPT_NAWS,
            data: vec![0x01, 0xFF, 0x02],
        };
        let encoded = TelnetEncoder::encode_command(&cmd);
        assert_eq!(
            encoded,
            vec![IAC, SB, OPT_NAWS, 0x01, 0xFF, 0xFF, 0x02, IAC, SE],
            "IAC in subneg data must be doubled"
        );
    }

    /// RFC 855: Subneg data without IAC encodes normally.
    #[test]
    fn test_encoder_subneg_no_iac_normal() {
        let cmd = TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![0, b'V', b'T', b'1', b'0', b'0'],
        };
        let encoded = TelnetEncoder::encode_command(&cmd);
        assert_eq!(
            encoded,
            vec![IAC, SB, OPT_TERMINAL_TYPE, 0, b'V', b'T', b'1', b'0', b'0', IAC, SE]
        );
    }

    /// RFC 855: Multiple IAC bytes in subneg data are each doubled.
    #[test]
    fn test_encoder_subneg_multiple_iac_bytes() {
        let cmd = TelnetCommand::Subnegotiation {
            option: OPT_NAWS,
            data: vec![0xFF, 0xFF],
        };
        let encoded = TelnetEncoder::encode_command(&cmd);
        assert_eq!(
            encoded,
            vec![IAC, SB, OPT_NAWS, 0xFF, 0xFF, 0xFF, 0xFF, IAC, SE],
            "Each IAC byte in subneg data must be doubled"
        );
    }

    /// RFC 855: Empty subneg data encodes with just IAC SB option IAC SE.
    #[test]
    fn test_encoder_subneg_empty_data() {
        let cmd = TelnetCommand::Subnegotiation {
            option: OPT_STATUS,
            data: vec![],
        };
        let encoded = TelnetEncoder::encode_command(&cmd);
        assert_eq!(encoded, vec![IAC, SB, OPT_STATUS, IAC, SE]);
    }

    // =========================================================================
    // 8. Round-Trip Tests - Encode then Decode
    // =========================================================================

    /// RFC 854: Encoding then decoding a DO command must return the original.
    #[test]
    fn test_roundtrip_do() {
        let original = TelnetCommand::Do(OPT_ECHO);
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// RFC 854: Encoding then decoding a DONT command must return the original.
    #[test]
    fn test_roundtrip_dont() {
        let original = TelnetCommand::Dont(OPT_BINARY);
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// RFC 854: Encoding then decoding a WILL command must return the original.
    #[test]
    fn test_roundtrip_will() {
        let original = TelnetCommand::Will(OPT_SUPPRESS_GA);
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// RFC 854: Encoding then decoding a WONT command must return the original.
    #[test]
    fn test_roundtrip_wont() {
        let original = TelnetCommand::Wont(OPT_NAWS);
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// Round-trip all single-byte command types.
    #[test]
    fn test_roundtrip_all_single_byte_commands() {
        let commands = vec![
            TelnetCommand::Nop,
            TelnetCommand::GoAhead,
            TelnetCommand::AbortOutput,
            TelnetCommand::AreYouThere,
            TelnetCommand::EraseCharacter,
            TelnetCommand::EraseLine,
            TelnetCommand::InterruptProcess,
            TelnetCommand::Break,
            TelnetCommand::DataMark,
            TelnetCommand::EndOfRecord,
            TelnetCommand::EndOfFile,
            TelnetCommand::Suspend,
            TelnetCommand::Abort,
        ];

        for cmd in commands {
            let encoded = TelnetEncoder::encode_command(&cmd);
            let mut decoder = TelnetDecoder::new();
            let decoded = decoder.decode(&encoded);
            assert_eq!(
                decoded,
                vec![cmd.clone()],
                "Round-trip failed for {:?}",
                cmd
            );
        }
    }

    /// Round-trip Data bytes including IAC.
    #[test]
    fn test_roundtrip_data_bytes() {
        for byte in [0u8, 1, 127, 254, 255] {
            let cmd = TelnetCommand::Data(byte);
            let encoded = TelnetEncoder::encode_command(&cmd);
            let mut decoder = TelnetDecoder::new();
            let decoded = decoder.decode(&encoded);
            assert_eq!(
                decoded,
                vec![TelnetCommand::Data(byte)],
                "Round-trip failed for Data({})",
                byte
            );
        }
    }

    /// RFC 855: Subneg with IAC in data must round-trip correctly.
    #[test]
    fn test_roundtrip_subneg_with_iac_data() {
        let original = TelnetCommand::Subnegotiation {
            option: OPT_NAWS,
            data: vec![0x00, 0xFF, 0x50, 0xFF],
        };
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(
            decoded,
            vec![original],
            "Subneg with IAC data must round-trip correctly"
        );
    }

    /// Round-trip subneg with no IAC in data.
    #[test]
    fn test_roundtrip_subneg_no_iac() {
        let original = TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: vec![0, b'x', b't', b'e', b'r', b'm'],
        };
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// Round-trip multiple commands at once.
    #[test]
    fn test_roundtrip_multiple_commands() {
        let commands = vec![
            TelnetCommand::Will(OPT_ECHO),
            TelnetCommand::Do(OPT_BINARY),
            TelnetCommand::Nop,
            TelnetCommand::GoAhead,
        ];
        let encoded = TelnetEncoder::encode_commands(&commands);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, commands);
    }

    // =========================================================================
    // 9. Option Negotiation Tests (RFC 855, 854)
    // =========================================================================

    /// RFC 855: Default state for all options must be Closed.
    #[test]
    fn test_negotiator_default_state_is_closed() {
        let negotiator = OptionNegotiator::new();
        for opt in [OPT_BINARY, OPT_ECHO, OPT_SUPPRESS_GA, OPT_TERMINAL_TYPE, OPT_NAWS, 100, 255]
        {
            assert_eq!(
                negotiator.get_option_state(opt),
                OptionState::Closed,
                "Option {} should default to Closed",
                opt
            );
        }
    }

    /// RFC 855: WILL/DO exchange enables an option.
    #[test]
    fn test_negotiator_will_do_enables_option() {
        let mut negotiator = OptionNegotiator::new();
        let response = negotiator.handle_will(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Do(OPT_ECHO),
            "Response to WILL should be DO"
        );
        assert!(
            negotiator.is_enabled(OPT_ECHO),
            "Option should be enabled after WILL/DO exchange"
        );
    }

    /// RFC 855: DO/WILL exchange enables an option (from our side).
    #[test]
    fn test_negotiator_do_will_enables_option() {
        let mut negotiator = OptionNegotiator::new();
        let response = negotiator.handle_do(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Will(OPT_ECHO),
            "Response to DO should be WILL"
        );
        assert!(
            negotiator.is_enabled(OPT_ECHO),
            "Option should be enabled after DO/WILL exchange"
        );
    }

    /// RFC 855: WONT/DONT exchange disables an option.
    #[test]
    fn test_negotiator_wont_dont_disables_option() {
        let mut negotiator = OptionNegotiator::new();
        // First enable the option
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);
        assert!(negotiator.is_enabled(OPT_ECHO));

        let response = negotiator.handle_wont(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Dont(OPT_ECHO),
            "Response to WONT should be DONT"
        );
        assert!(
            !negotiator.is_enabled(OPT_ECHO),
            "Option should be disabled after WONT/DONT exchange"
        );
    }

    /// RFC 855: DONT/WONT exchange disables an option (from our side).
    #[test]
    fn test_negotiator_dont_wont_disables_option() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);

        let response = negotiator.handle_dont(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Wont(OPT_ECHO),
            "Response to DONT should be WONT"
        );
        assert!(
            !negotiator.is_enabled(OPT_ECHO),
            "Option should be disabled after DONT/WONT exchange"
        );
    }

    /// RFC 855: Loop prevention — DO when already enabled returns WILL without state change.
    #[test]
    fn test_negotiator_do_when_already_enabled() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);

        let response = negotiator.handle_do(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Will(OPT_ECHO),
            "DO when already enabled should still return WILL"
        );
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::Enabled,
            "State should remain Enabled"
        );
    }

    /// RFC 855: Loop prevention — DONT when already Closed returns WONT.
    #[test]
    fn test_negotiator_dont_when_already_closed() {
        let mut negotiator = OptionNegotiator::new();
        let response = negotiator.handle_dont(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Wont(OPT_ECHO),
            "DONT when already Closed should return WONT"
        );
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::Closed,
            "State should remain Closed"
        );
    }

    /// RFC 855: WONT received for a requested option disables it.
    #[test]
    fn test_negotiator_wont_received_for_requested_option() {
        let mut negotiator = OptionNegotiator::new();
        // Request to enable (sets WantsEnable)
        let _request = negotiator.request_enable(OPT_ECHO);
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::WantsEnable
        );

        // Remote refuses with WONT
        let response = negotiator.handle_wont(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Dont(OPT_ECHO),
            "Response to WONT should be DONT"
        );
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::Closed,
            "Option should be Closed after refused WONT"
        );
    }

    /// RFC 855: WILL when already enabled returns DO (no state change).
    #[test]
    fn test_negotiator_will_when_already_enabled() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);

        let response = negotiator.handle_will(OPT_ECHO);
        assert_eq!(
            response,
            TelnetCommand::Do(OPT_ECHO),
            "WILL when already enabled should return DO"
        );
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::Enabled,
            "State should remain Enabled"
        );
    }

    /// RFC 855: process_command routes DO/DONT/WILL/WONT correctly.
    #[test]
    fn test_negotiator_process_command_routing() {
        let mut neg = OptionNegotiator::new();
        assert!(neg.process_command(&TelnetCommand::Do(OPT_ECHO)).is_some());

        let mut neg = OptionNegotiator::new();
        assert!(neg.process_command(&TelnetCommand::Will(OPT_ECHO)).is_some());

        let mut neg = OptionNegotiator::new();
        assert!(neg.process_command(&TelnetCommand::Dont(OPT_ECHO)).is_some());

        let mut neg = OptionNegotiator::new();
        assert!(neg.process_command(&TelnetCommand::Wont(OPT_ECHO)).is_some());
    }

    /// RFC 855: process_command returns None for non-negotiation commands.
    #[test]
    fn test_negotiator_process_command_non_negotiation() {
        let mut negotiator = OptionNegotiator::new();
        assert_eq!(negotiator.process_command(&TelnetCommand::Nop), None);
        assert_eq!(negotiator.process_command(&TelnetCommand::GoAhead), None);
        assert_eq!(negotiator.process_command(&TelnetCommand::Data(65)), None);
        assert_eq!(
            negotiator.process_command(&TelnetCommand::Subnegotiation {
                option: OPT_ECHO,
                data: vec![]
            }),
            None
        );
    }

    /// RFC 855: request_enable sets WantsEnable and returns WILL.
    #[test]
    fn test_negotiator_request_enable() {
        let mut negotiator = OptionNegotiator::new();
        let cmd = negotiator.request_enable(OPT_ECHO);
        assert_eq!(cmd, TelnetCommand::Will(OPT_ECHO));
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::WantsEnable
        );
    }

    /// RFC 855: request_enable when already enabled returns NOP.
    #[test]
    fn test_negotiator_request_enable_already_enabled() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);
        let cmd = negotiator.request_enable(OPT_ECHO);
        assert_eq!(cmd, TelnetCommand::Nop, "Should return NOP if already enabled");
    }

    /// RFC 855: request_disable sets WantsDisable and returns WONT.
    #[test]
    fn test_negotiator_request_disable() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);
        let cmd = negotiator.request_disable(OPT_ECHO);
        assert_eq!(cmd, TelnetCommand::Wont(OPT_ECHO));
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::WantsDisable
        );
    }

    /// RFC 855: request_disable when already closed returns NOP.
    #[test]
    fn test_negotiator_request_disable_already_closed() {
        let mut negotiator = OptionNegotiator::new();
        let cmd = negotiator.request_disable(OPT_ECHO);
        assert_eq!(cmd, TelnetCommand::Nop, "Should return NOP if already closed");
    }

    /// RFC 855: Negotiator reset clears all option states.
    #[test]
    fn test_negotiator_reset_clears_all() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(OPT_ECHO, OptionState::Enabled);
        negotiator.set_option_state(OPT_BINARY, OptionState::WantsEnable);
        negotiator.set_option_state(OPT_SUPPRESS_GA, OptionState::WantsDisable);

        negotiator.reset();

        assert_eq!(negotiator.get_option_state(OPT_ECHO), OptionState::Closed);
        assert_eq!(negotiator.get_option_state(OPT_BINARY), OptionState::Closed);
        assert_eq!(
            negotiator.get_option_state(OPT_SUPPRESS_GA),
            OptionState::Closed
        );
    }

    // =========================================================================
    // 10. Boundary Tests
    // =========================================================================

    /// Boundary: Option code 0 (minimum) works in negotiation commands.
    #[test]
    fn test_boundary_option_code_zero() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, DO, 0]);
        assert_eq!(cmds, vec![TelnetCommand::Do(0)]);

        let encoded = TelnetEncoder::encode_command(&TelnetCommand::Do(0));
        assert_eq!(encoded, vec![IAC, DO, 0]);
    }

    /// Boundary: Option code 255 (maximum) works in negotiation commands.
    #[test]
    fn test_boundary_option_code_255() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, DO, 255]);
        assert_eq!(cmds, vec![TelnetCommand::Do(255)]);

        let encoded = TelnetEncoder::encode_command(&TelnetCommand::Do(255));
        assert_eq!(encoded, vec![IAC, DO, 255]);
    }

    /// Boundary: Option code 255 round-trips through WILL/WONT/DO/DONT.
    #[test]
    fn test_boundary_option_255_all_negotiation_types() {
        for cmd in [
            TelnetCommand::Do(255),
            TelnetCommand::Dont(255),
            TelnetCommand::Will(255),
            TelnetCommand::Wont(255),
        ] {
            let encoded = TelnetEncoder::encode_command(&cmd);
            let mut decoder = TelnetDecoder::new();
            let decoded = decoder.decode(&encoded);
            assert_eq!(decoded, vec![cmd.clone()], "Option 255 round-trip failed for {:?}", cmd);
        }
    }

    /// Boundary: Empty subneg data round-trips.
    #[test]
    fn test_boundary_empty_subneg_roundtrip() {
        let original = TelnetCommand::Subnegotiation {
            option: OPT_ECHO,
            data: vec![],
        };
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original]);
    }

    /// Boundary: Large subneg data (1024 bytes) round-trips.
    #[test]
    fn test_boundary_large_subneg_data() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let original = TelnetCommand::Subnegotiation {
            option: OPT_NAWS,
            data: data.clone(),
        };
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded.len(), 1);
        if let TelnetCommand::Subnegotiation {
            option,
            data: decoded_data,
        } = &decoded[0]
        {
            assert_eq!(*option, OPT_NAWS);
            assert_eq!(decoded_data.len(), data.len(), "Large subneg data length mismatch");
            assert_eq!(*decoded_data, data, "Large subneg data content mismatch");
        } else {
            panic!("Expected Subnegotiation, got {:?}", decoded[0]);
        }
    }

    /// Boundary: Large subneg data with IAC bytes round-trips.
    #[test]
    fn test_boundary_large_subneg_data_with_iac() {
        // Include multiple 0xFF bytes in the data
        let mut data = Vec::with_capacity(512);
        for i in 0..512 {
            data.push(if i % 64 == 0 { 0xFF } else { (i % 254) as u8 });
        }
        let original = TelnetCommand::Subnegotiation {
            option: OPT_TERMINAL_TYPE,
            data: data.clone(),
        };
        let encoded = TelnetEncoder::encode_command(&original);
        let mut decoder = TelnetDecoder::new();
        let decoded = decoder.decode(&encoded);
        assert_eq!(decoded, vec![original], "Large subneg with IAC bytes must round-trip");
    }

    /// Boundary: All 256 possible data bytes through decoder.
    #[test]
    fn test_boundary_all_256_data_bytes_through_decoder() {
        let mut input = Vec::with_capacity(512);
        for byte in 0u8..=254 {
            input.push(byte);
        }
        // For 0xFF, use IAC IAC
        input.push(IAC);
        input.push(IAC);

        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&input);

        assert_eq!(cmds.len(), 256, "Should decode 256 data commands");
        for byte in 0u8..=254 {
            assert_eq!(
                cmds[byte as usize],
                TelnetCommand::Data(byte),
                "Byte {} mismatch",
                byte
            );
        }
        assert_eq!(
            cmds[255],
            TelnetCommand::Data(0xFF),
            "IAC IAC should produce Data(0xFF)"
        );
    }

    // =========================================================================
    // 11. Negative Tests / Error Handling
    // =========================================================================

    /// Unknown command byte after IAC is treated as data.
    #[test]
    fn test_negative_unknown_command_byte_after_iac() {
        // Bytes that aren't valid TELNET commands (not in the defined set)
        let unknown_bytes: Vec<u8> = (128u8..236).collect();
        for &byte in &unknown_bytes {
            // Skip bytes that are valid commands
            if [
                IAC, DO, DONT, WILL, WONT, SB, SE, NOP, GA, AO, AYT, EC, EL, IP, BRK, DM, EOR,
                EOF, SUSP, ABOR,
            ]
            .contains(&byte)
            {
                continue;
            }
            let mut decoder = TelnetDecoder::new();
            let cmds = decoder.decode(&[IAC, byte]);
            assert_eq!(
                cmds.len(),
                1,
                "Unknown byte 0x{:02X} after IAC should produce one command",
                byte
            );
            assert_eq!(
                cmds[0],
                TelnetCommand::Data(byte),
                "Unknown byte 0x{:02X} after IAC should be treated as Data",
                byte
            );
        }
    }

    /// Unexpected SE without prior SB in IAC state is treated as NOP.
    #[test]
    fn test_negative_unexpected_se_in_iac_state() {
        let mut decoder = TelnetDecoder::new();
        let cmds = decoder.decode(&[IAC, SE]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Nop,
            "Unexpected SE in IAC state should produce NOP"
        );
    }

    /// Decoder reset clears state back to Normal.
    #[test]
    fn test_negative_decoder_reset_clears_state() {
        let mut decoder = TelnetDecoder::new();

        // Put decoder in IAC state
        decoder.decode_byte(IAC);
        assert_eq!(decoder.state(), crate::decoder::DecodeState::Iac);

        decoder.reset();
        assert_eq!(
            decoder.state(),
            crate::decoder::DecodeState::Normal,
            "Reset must return to Normal state"
        );
    }

    /// Decoder reset during subnegotiation clears state.
    #[test]
    fn test_negative_decoder_reset_during_subneg() {
        let mut decoder = TelnetDecoder::new();

        // Start a subnegotiation
        decoder.decode(&[IAC, SB, OPT_ECHO, 0x01, 0x02]);
        assert_eq!(decoder.state(), crate::decoder::DecodeState::SbData);

        decoder.reset();
        assert_eq!(
            decoder.state(),
            crate::decoder::DecodeState::Normal,
            "Reset during subneg must return to Normal"
        );

        // Verify decoder works normally after reset
        let cmds = decoder.decode(&[65]);
        assert_eq!(cmds, vec![TelnetCommand::Data(65)]);
    }

    /// Decoder handles split commands across multiple decode calls.
    #[test]
    fn test_negative_split_command_across_calls() {
        let mut decoder = TelnetDecoder::new();

        // Split IAC DO OPT_ECHO across three calls
        let cmds1 = decoder.decode(&[IAC]);
        assert!(cmds1.is_empty(), "IAC alone should produce no commands");

        let cmds2 = decoder.decode(&[DO]);
        assert!(cmds2.is_empty(), "DO after IAC should produce no commands yet");

        let cmds3 = decoder.decode(&[OPT_ECHO]);
        assert_eq!(
            cmds3,
            vec![TelnetCommand::Do(OPT_ECHO)],
            "Option byte should complete the command"
        );
    }

    /// Split subnegotiation across multiple calls.
    #[test]
    fn test_negative_split_subneg_across_calls() {
        let mut decoder = TelnetDecoder::new();

        let cmds1 = decoder.decode(&[IAC, SB]);
        assert!(cmds1.is_empty());

        let cmds2 = decoder.decode(&[OPT_TERMINAL_TYPE, 0, b'V']);
        assert!(cmds2.is_empty());

        let cmds3 = decoder.decode(&[b'T', b'1', b'0', b'0']);
        assert!(cmds3.is_empty());

        let cmds4 = decoder.decode(&[IAC, SE]);
        assert_eq!(cmds4.len(), 1);
        assert_eq!(
            cmds4[0],
            TelnetCommand::Subnegotiation {
                option: OPT_TERMINAL_TYPE,
                data: vec![0, b'V', b'T', b'1', b'0', b'0'],
            }
        );
    }

    /// Interleaved data and commands decode correctly.
    #[test]
    fn test_negative_interleaved_data_and_commands() {
        let mut decoder = TelnetDecoder::new();
        let input = [
            b'H', b'i', IAC, NOP, b'!', IAC, GA, b'\n',
        ];
        let cmds = decoder.decode(&input);
        assert_eq!(
            cmds,
            vec![
                TelnetCommand::Data(b'H'),
                TelnetCommand::Data(b'i'),
                TelnetCommand::Nop,
                TelnetCommand::Data(b'!'),
                TelnetCommand::GoAhead,
                TelnetCommand::Data(b'\n'),
            ]
        );
    }

    /// SE byte (0xF0) in SbData state without prior IAC is treated as data.
    #[test]
    fn test_negative_bare_se_in_subneg_data() {
        let mut decoder = TelnetDecoder::new();
        // IAC SB opt SE(bare) data IAC SE
        let cmds = decoder.decode(&[IAC, SB, OPT_NAWS, SE, 0x42, IAC, SE]);
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            TelnetCommand::Subnegotiation {
                option: OPT_NAWS,
                data: vec![SE, 0x42],
            },
            "Bare SE (0xF0) in subneg data should be treated as data"
        );
    }

    // =========================================================================
    // 12. Default State Tests (RFC 856, 857, 858)
    // =========================================================================

    /// RFC 856: Binary mode defaults to off (Closed). Per RFC 856, the default
    /// mode of operation is ASCII with TRANSMIT-BINARY disabled.
    #[test]
    fn test_default_binary_mode_off() {
        let negotiator = OptionNegotiator::new();
        assert_eq!(
            negotiator.get_option_state(OPT_BINARY),
            OptionState::Closed,
            "RFC 856: Binary mode must default to off (Closed)"
        );
        assert!(
            !negotiator.is_enabled(OPT_BINARY),
            "RFC 856: Binary mode must not be enabled by default"
        );
    }

    /// RFC 857: Echo defaults to off (Closed). Per RFC 857, the existing
    /// condition of not echoing must be preserved.
    #[test]
    fn test_default_echo_off() {
        let negotiator = OptionNegotiator::new();
        assert_eq!(
            negotiator.get_option_state(OPT_ECHO),
            OptionState::Closed,
            "RFC 857: Echo must default to off (Closed)"
        );
        assert!(
            !negotiator.is_enabled(OPT_ECHO),
            "RFC 857: Echo must not be enabled by default"
        );
    }

    /// RFC 858: Suppress Go-Ahead defaults to off (Closed). Per RFC 858,
    /// the existing condition of sending Go-Ahead signals must be preserved.
    #[test]
    fn test_default_suppress_ga_off() {
        let negotiator = OptionNegotiator::new();
        assert_eq!(
            negotiator.get_option_state(OPT_SUPPRESS_GA),
            OptionState::Closed,
            "RFC 858: Suppress-GA must default to off (Closed)"
        );
        assert!(
            !negotiator.is_enabled(OPT_SUPPRESS_GA),
            "RFC 858: Suppress-GA must not be enabled by default"
        );
    }

    /// All standard options default to Closed (WONT/DONT).
    #[test]
    fn test_default_all_standard_options_closed() {
        let negotiator = OptionNegotiator::new();
        let standard_options = [
            (OPT_BINARY, "BINARY"),
            (OPT_ECHO, "ECHO"),
            (OPT_SUPPRESS_GA, "SUPPRESS-GA"),
            (OPT_STATUS, "STATUS"),
            (OPT_TIMING_MARK, "TIMING-MARK"),
            (OPT_TERMINAL_TYPE, "TERMINAL-TYPE"),
            (OPT_NAWS, "NAWS"),
            (OPT_LINEMODE, "LINEMODE"),
        ];
        for (opt, name) in standard_options {
            assert_eq!(
                negotiator.get_option_state(opt),
                OptionState::Closed,
                "{} (option {}) must default to Closed",
                name,
                opt
            );
        }
    }

    /// OptionState::is_enabled returns correct values.
    #[test]
    fn test_option_state_is_enabled() {
        assert!(OptionState::Enabled.is_enabled());
        assert!(!OptionState::Closed.is_enabled());
        assert!(!OptionState::WantsEnable.is_enabled());
        assert!(!OptionState::WantsDisable.is_enabled());
        assert!(!OptionState::RemoteWantsEnable.is_enabled());
        assert!(!OptionState::RemoteWantsDisable.is_enabled());
    }

    /// Full negotiation lifecycle: enable via remote DO → disable via remote DONT → re-enable.
    #[test]
    fn test_full_negotiation_lifecycle() {
        let mut negotiator = OptionNegotiator::new();

        // 1. Start: Closed
        assert_eq!(negotiator.get_option_state(OPT_ECHO), OptionState::Closed);

        // 2. Remote sends DO → we accept with WILL, state becomes Enabled
        let resp = negotiator.handle_do(OPT_ECHO);
        assert_eq!(resp, TelnetCommand::Will(OPT_ECHO));
        assert!(negotiator.is_enabled(OPT_ECHO));

        // 3. Remote sends DONT → we confirm with WONT, state becomes Closed
        let resp = negotiator.handle_dont(OPT_ECHO);
        assert_eq!(resp, TelnetCommand::Wont(OPT_ECHO));
        assert!(!negotiator.is_enabled(OPT_ECHO));
        assert_eq!(negotiator.get_option_state(OPT_ECHO), OptionState::Closed);

        // 4. Remote sends DO again → re-enable
        let resp = negotiator.handle_do(OPT_ECHO);
        assert_eq!(resp, TelnetCommand::Will(OPT_ECHO));
        assert!(negotiator.is_enabled(OPT_ECHO));
    }

    /// Negotiation: remote initiates with WILL, we respond with DO, option is enabled.
    #[test]
    fn test_negotiation_remote_initiates_will() {
        let mut negotiator = OptionNegotiator::new();

        // Remote sends WILL
        let resp = negotiator.handle_will(OPT_SUPPRESS_GA);
        assert_eq!(resp, TelnetCommand::Do(OPT_SUPPRESS_GA));
        assert!(negotiator.is_enabled(OPT_SUPPRESS_GA));

        // Remote sends WONT to disable
        let resp2 = negotiator.handle_wont(OPT_SUPPRESS_GA);
        assert_eq!(resp2, TelnetCommand::Dont(OPT_SUPPRESS_GA));
        assert!(!negotiator.is_enabled(OPT_SUPPRESS_GA));
    }

    /// Negotiation: remote initiates with DO, we respond with WILL, option is enabled.
    #[test]
    fn test_negotiation_remote_initiates_do() {
        let mut negotiator = OptionNegotiator::new();

        // Remote sends DO
        let resp = negotiator.handle_do(OPT_BINARY);
        assert_eq!(resp, TelnetCommand::Will(OPT_BINARY));
        assert!(negotiator.is_enabled(OPT_BINARY));

        // Remote sends DONT to disable
        let resp2 = negotiator.handle_dont(OPT_BINARY);
        assert_eq!(resp2, TelnetCommand::Wont(OPT_BINARY));
        assert!(!negotiator.is_enabled(OPT_BINARY));
    }

    /// Multiple options can be negotiated independently.
    #[test]
    fn test_negotiation_multiple_independent_options() {
        let mut negotiator = OptionNegotiator::new();

        negotiator.handle_do(OPT_ECHO);
        negotiator.handle_will(OPT_BINARY);
        negotiator.handle_do(OPT_SUPPRESS_GA);

        assert!(negotiator.is_enabled(OPT_ECHO));
        assert!(negotiator.is_enabled(OPT_BINARY));
        assert!(negotiator.is_enabled(OPT_SUPPRESS_GA));
        assert!(!negotiator.is_enabled(OPT_TERMINAL_TYPE));
    }
}
