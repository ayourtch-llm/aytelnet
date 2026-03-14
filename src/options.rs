//! TELNET option negotiation.
//!
//! This module handles TELNET option negotiation according to RFC 855.
//! It manages the state machine for WILL/WONT/DO/DONT exchanges.

use crate::types::{OptionState, TelnetCommand};

/// TELNET option negotiator.
///
/// Manages option negotiation state and generates appropriate
/// responses to incoming options.
#[derive(Debug, Clone, Default)]
pub struct OptionNegotiator {
    /// State of each option
    options: std::collections::HashMap<u8, OptionState>,
}

impl OptionNegotiator {
    /// Create a new negotiator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the state of an option.
    pub fn get_option_state(&self, option: u8) -> OptionState {
        *self.options.get(&option).unwrap_or(&OptionState::Closed)
    }

    /// Set the state of an option.
    pub fn set_option_state(&mut self, option: u8, state: OptionState) {
        self.options.insert(option, state);
    }

    /// Handle an incoming DO command.
    ///
    /// DO means the sender wants us to enable an option.
    /// Returns the response command (WILL or WONT).
    pub fn handle_do(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Closed => {
                // Accept the option
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Will(option)
            }
            OptionState::WantsDisable => {
                // We want to disable, refuse the DO
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::Enabled => {
                // Already enabled, just confirm
                TelnetCommand::Will(option)
            }
            OptionState::RemoteWantsEnable => {
                // Already waiting to enable, confirm
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Will(option)
            }
            OptionState::WantsEnable | OptionState::RemoteWantsDisable => {
                // Conflicting state, refuse
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Handle an incoming DONT command.
    ///
    /// DONT means the sender wants us to disable an option.
    /// Returns the response command (WILL or WONT).
    pub fn handle_dont(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Enabled | OptionState::WantsEnable => {
                // Disable the option
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::Closed => {
                // Already disabled, just confirm
                TelnetCommand::Wont(option)
            }
            OptionState::RemoteWantsDisable | OptionState::WantsDisable => {
                // Already waiting to disable, confirm
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::RemoteWantsEnable => {
                // Remote wants to enable, we want to disable - conflict
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Handle an incoming WILL command.
    ///
    /// WILL means the sender wants to enable an option on their side.
    /// Returns the response command (DO or DONT).
    pub fn handle_will(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Closed | OptionState::WantsDisable | OptionState::RemoteWantsDisable => {
                // Accept the option
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Do(option)
            }
            OptionState::Enabled => {
                // Already enabled, just confirm
                TelnetCommand::Do(option)
            }
            OptionState::RemoteWantsEnable | OptionState::WantsEnable => {
                // Already waiting to enable, confirm
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Do(option)
            }
        }
    }

    /// Handle an incoming WONT command.
    ///
    /// WONT means the sender wants to disable an option on their side.
    /// Returns the response command (DO or DONT).
    pub fn handle_wont(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Enabled | OptionState::WantsEnable | OptionState::RemoteWantsEnable => {
                // Disable the option
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Dont(option)
            }
            OptionState::Closed => {
                // Already disabled, just confirm
                TelnetCommand::Dont(option)
            }
            OptionState::RemoteWantsDisable | OptionState::WantsDisable => {
                // Already waiting to disable, confirm
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Dont(option)
            }
        }
    }

    /// Process an incoming TELNET command and generate response.
    pub fn process_command(&mut self, command: &TelnetCommand) -> Option<TelnetCommand> {
        match command {
            TelnetCommand::Do(opt) => Some(self.handle_do(*opt)),
            TelnetCommand::Dont(opt) => Some(self.handle_dont(*opt)),
            TelnetCommand::Will(opt) => Some(self.handle_will(*opt)),
            TelnetCommand::Wont(opt) => Some(self.handle_wont(*opt)),
            _ => None,
        }
    }

    /// Request to enable an option locally.
    pub fn request_enable(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Closed | OptionState::WantsDisable => {
                self.set_option_state(option, OptionState::WantsEnable);
                TelnetCommand::Will(option)
            }
            OptionState::Enabled => {
                // Already enabled
                TelnetCommand::Nop
            }
            OptionState::WantsEnable => {
                // Already requested
                TelnetCommand::Nop
            }
            OptionState::RemoteWantsEnable => {
                // Remote already wants to enable, just confirm
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Do(option)
            }
            OptionState::RemoteWantsDisable => {
                // Remote wants to disable, refuse
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Request to disable an option locally.
    pub fn request_disable(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Enabled | OptionState::WantsEnable => {
                self.set_option_state(option, OptionState::WantsDisable);
                TelnetCommand::Wont(option)
            }
            OptionState::Closed => {
                // Already disabled
                TelnetCommand::Nop
            }
            OptionState::WantsDisable => {
                // Already requested
                TelnetCommand::Nop
            }
            OptionState::RemoteWantsDisable => {
                // Remote already wants to disable, confirm
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Dont(option)
            }
            OptionState::RemoteWantsEnable => {
                // Remote wants to enable, refuse
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Get all option states.
    pub fn get_all_states(&self) -> std::collections::HashMap<u8, OptionState> {
        self.options.clone()
    }

    /// Check if an option is enabled.
    pub fn is_enabled(&self, option: u8) -> bool {
        self.get_option_state(option).is_enabled()
    }

    /// Reset all option states.
    pub fn reset(&mut self) {
        self.options.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TelnetCommand;

    #[test]
    fn test_initial_state() {
        let negotiator = OptionNegotiator::new();
        assert_eq!(negotiator.get_option_state(1), OptionState::Closed);
    }

    #[test]
    fn test_handle_do_accept() {
        let mut negotiator = OptionNegotiator::new();
        let response = negotiator.handle_do(1);
        
        assert_eq!(response, TelnetCommand::Will(1));
        assert!(negotiator.is_enabled(1));
    }

    #[test]
    fn test_handle_do_refuse() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::WantsDisable);
        let response = negotiator.handle_do(1);
        
        assert_eq!(response, TelnetCommand::Wont(1));
        assert!(!negotiator.is_enabled(1));
    }

    #[test]
    fn test_handle_dont_disable() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        let response = negotiator.handle_dont(1);
        
        assert_eq!(response, TelnetCommand::Wont(1));
        assert!(!negotiator.is_enabled(1));
    }

    #[test]
    fn test_handle_will_accept() {
        let mut negotiator = OptionNegotiator::new();
        let response = negotiator.handle_will(1);
        
        assert_eq!(response, TelnetCommand::Do(1));
        assert!(negotiator.is_enabled(1));
    }

    #[test]
    fn test_handle_wont_disable() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        let response = negotiator.handle_wont(1);
        
        assert_eq!(response, TelnetCommand::Dont(1));
        assert!(!negotiator.is_enabled(1));
    }

    #[test]
    fn test_request_enable() {
        let mut negotiator = OptionNegotiator::new();
        let request = negotiator.request_enable(1);
        
        assert_eq!(request, TelnetCommand::Will(1));
        assert!(negotiator.get_option_state(1).wants_enable());
    }

    #[test]
    fn test_request_disable() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        let request = negotiator.request_disable(1);
        
        assert_eq!(request, TelnetCommand::Wont(1));
        assert!(negotiator.get_option_state(1).wants_disable());
    }

    #[test]
    fn test_process_do() {
        let mut negotiator = OptionNegotiator::new();
        let command = TelnetCommand::Do(1);
        let response = negotiator.process_command(&command).unwrap();
        
        assert_eq!(response, TelnetCommand::Will(1));
    }

    #[test]
    fn test_process_will() {
        let mut negotiator = OptionNegotiator::new();
        let command = TelnetCommand::Will(1);
        let response = negotiator.process_command(&command).unwrap();
        
        assert_eq!(response, TelnetCommand::Do(1));
    }

    #[test]
    fn test_process_dont() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        let command = TelnetCommand::Dont(1);
        let response = negotiator.process_command(&command).unwrap();
        
        assert_eq!(response, TelnetCommand::Wont(1));
    }

    #[test]
    fn test_process_wont() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        let command = TelnetCommand::Wont(1);
        let response = negotiator.process_command(&command).unwrap();
        
        assert_eq!(response, TelnetCommand::Dont(1));
    }

    #[test]
    fn test_process_non_option_command() {
        let mut negotiator = OptionNegotiator::new();
        let command = TelnetCommand::Nop;
        let response = negotiator.process_command(&command);
        
        assert!(response.is_none());
    }

    #[test]
    fn test_reset() {
        let mut negotiator = OptionNegotiator::new();
        negotiator.set_option_state(1, OptionState::Enabled);
        negotiator.reset();
        
        assert_eq!(negotiator.get_option_state(1), OptionState::Closed);
    }

    #[test]
    fn test_multiple_options() {
        let mut negotiator = OptionNegotiator::new();
        
        negotiator.handle_do(1);
        negotiator.handle_do(2);
        negotiator.handle_do(3);
        
        assert!(negotiator.is_enabled(1));
        assert!(negotiator.is_enabled(2));
        assert!(negotiator.is_enabled(3));
    }
}