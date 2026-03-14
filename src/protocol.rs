//! TELNET protocol constants and types.
//!
//! This module defines all the constants and basic types needed for
//! TELNET protocol implementation according to RFC 854 and related RFCs.

/// IAC (Interpret As Command) byte - 255 (0xFF)
///
/// When the IAC byte is received, the next byte is interpreted as a
/// TELNET command or option code, not as data.
pub const IAC: u8 = 255;

/// DO command - 254 (0xFE)
///
/// The sender of this command REQUESTS that the receiver enable an option.
pub const DO: u8 = 254;

/// DON'T command - 253 (0xFD)
///
/// The sender of this command DEMANDS that the receiver disable an option.
pub const DONT: u8 = 253;

/// WILL command - 252 (0xFC)
///
/// The sender of this command DECLARES that it will enable an option.
pub const WILL: u8 = 252;

/// WON'T command - 251 (0xFB)
///
/// The sender of this command DECLARES that it will not enable an option.
pub const WONT: u8 = 251;

/// SB (Subnegotiation Begin) command - 250 (0xFA)
///
/// Marks the beginning of subnegotiation parameters.
pub const SB: u8 = 250;

/// SE (Subnegotiation End) command - 240 (0xF0)
///
/// Marks the end of subnegotiation parameters.
pub const SE: u8 = 240;

/// NOP (No Operation) command - 241 (0xF1)
///
/// No operation. Used to maintain synchronization.
pub const NOP: u8 = 241;

/// GA (Go Ahead) command - 242 (0xF2)
///
/// Indicates that the receiver may transmit.
pub const GA: u8 = 242;

/// AO (Abort Output) command - 243 (0xF3)
///
/// Indicates that the previous output should be discarded.
pub const AO: u8 = 243;

/// AYT (Are You There) command - 244 (0xF4)
///
/// Asks if the receiver is still present.
pub const AYT: u8 = 244;

/// EC (Erase Character) command - 245 (0xF5)
///
/// Request to erase the last character.
pub const EC: u8 = 245;

/// EL (Erase Line) command - 246 (0xF6)
///
/// Request to erase the current line.
pub const EL: u8 = 246;

/// IP (Interrupt Process) command - 247 (0xF7)
///
/// Request to interrupt the current process.
pub const IP: u8 = 247;

/// BRK (Break) command - 248 (0xF8)
///
/// Indicates a break condition.
pub const BRK: u8 = 248;

/// DM (Data Mark) command - 249 (0xF9)
///
/// Marks significant data.
pub const DM: u8 = 249;

/// EOR (End of Record) command - 240 (0xF0)
///
/// Marks end of record.
pub const EOR: u8 = 240;

/// EOF (End of File) command - 236 (0xEC)
///
/// Marks end of file.
pub const EOF: u8 = 236;

/// SUSP (Suspend) command - 237 (0xED)
///
/// Suspend the current process.
pub const SUSP: u8 = 237;

/// ABOR (Abort) command - 238 (0xEE)
///
/// Abort the current operation.
pub const ABOR: u8 = 238;

// Common option codes

/// ECHO option - 1
///
/// Echo received data back to the sender.
pub const OPT_ECHO: u8 = 1;

/// SUPPRESS GO AHEAD option - 3
///
/// Suppress transmission of GA (Go Ahead) characters.
pub const OPT_SUPPRESS_GA: u8 = 3;

/// STATUS option - 5
///
/// Query current option status.
pub const OPT_STATUS: u8 = 5;

/// TIMING MARK option - 6
///
/// Synchronization mechanism.
pub const OPT_TIMING_MARK: u8 = 6;

/// BINARY option - 8
///
/// Enable 8-bit binary data transmission.
pub const OPT_BINARY: u8 = 8;

/// TERMINAL-TYPE option - 24
///
/// Identify terminal type.
pub const OPT_TERMINAL_TYPE: u8 = 24;

/// NAWS (Negotiate About Window Size) option - 31
///
/// Negotiate terminal window size.
pub const OPT_NAWS: u8 = 31;

/// LINEMODE option - 34
///
/// Enable local line editing.
pub const OPT_LINEMODE: u8 = 34;

/// New ENVIRON option - 39
///
/// Pass environment variables.
pub const OPT_NEW_ENVIRON: u8 = 39;

/// AUTHENTICATION option - 29
///
/// Authentication framework.
pub const OPT_AUTHENTICATION: u8 = 29;

/// ENCRYPTION option - 35
///
/// Encryption support.
pub const OPT_ENCRYPTION: u8 = 35;

/// RFC 1184 - Telnet Linemode Option
pub const OPT_LINEMODE_MODE: u8 = 1;
pub const OPT_LINEMODE_FORWARDMASK: u8 = 2;
pub const OPT_LINEMODE_SLC: u8 = 3;

/// RFC 1184 - Linemode mode bits
pub const MODE_EDIT: u8 = 1;
pub const MODE_TRAPSIG: u8 = 2;
pub const MODE_ACK: u8 = 4;
pub const MODE_SOFT_TAB: u8 = 8;
pub const MODE_LIT_ECHO: u8 = 16;

/// RFC 1184 - SLC codes
pub const SLC_ABORT: u8 = 1;
pub const SLC_AOTERMIN: u8 = 2;
pub const SLC_AOTO: u8 = 3;
pub const SLC_AOWT: u8 = 4;
pub const SLC_BRK: u8 = 5;
pub const SLC_IP: u8 = 6;
pub const SLC_AO: u8 = 7;
pub const SLC_EC: u8 = 8;
pub const SLC_EL: u8 = 9;
pub const SLC_GA: u8 = 10;
pub const SLC_EOR: u8 = 11;
pub const SLC_EOF: u8 = 12;
pub const SLC_SUSP: u8 = 13;
pub const SLC_XON: u8 = 14;
pub const SLC_XOFF: u8 = 15;
pub const SLC_FORW1: u8 = 16;
pub const SLC_FORW2: u8 = 17;