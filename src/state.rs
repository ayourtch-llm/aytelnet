//! TELNET state manager.
//!
//! This module handles the overall client state including connection
//! status and option states.

use crate::types::{ClientState, ConnectionState, OptionState, TelnetCommand};

/// TELNET client state manager.
///
/// Manages the overall state of the TELNET client including
/// connection state and option negotiation state.
#[derive(Debug, Clone)]
pub struct StateManager {
    /// Current client state
    client_state: ClientState,
}

impl StateManager {
    /// Create a new state manager.
    pub fn new() -> Self {
        Self {
            client_state: ClientState::default(),
        }
    }

    /// Get the current connection state.
    pub fn connection_state(&self) -> ConnectionState {
        self.client_state.connection_state
    }

    /// Set the connection state.
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.client_state.connection_state = state;
    }

    /// Get the state of an option.
    pub fn get_option_state(&self, option: u8) -> OptionState {
        self.client_state.get_option(option)
    }

    /// Set the state of an option.
    pub fn set_option_state(&mut self, option: u8, state: OptionState) {
        self.client_state.set_option(option, state);
    }

    /// Check if an option is enabled.
    pub fn is_option_enabled(&self, option: u8) -> bool {
        self.client_state.is_option_enabled(option)
    }

    /// Process an incoming TELNET command and update state.
    ///
    /// Returns any commands that should be sent in response.
    pub fn process_command(&mut self, command: &TelnetCommand) -> Vec<TelnetCommand> {
        let mut responses = Vec::new();

        match command {
            TelnetCommand::Do(opt) => {
                // Remote wants us to enable an option
                let response = self.handle_do(*opt);
                responses.push(response);
            }
            TelnetCommand::Dont(opt) => {
                // Remote wants us to disable an option
                let response = self.handle_dont(*opt);
                responses.push(response);
            }
            TelnetCommand::Will(opt) => {
                // Remote will enable an option
                let response = self.handle_will(*opt);
                responses.push(response);
            }
            TelnetCommand::Wont(opt) => {
                // Remote wont enable an option
                let response = self.handle_wont(*opt);
                responses.push(response);
            }
            TelnetCommand::Subnegotiation { option, data } => {
                // Handle subnegotiation
                self.handle_subnegotiation(*option, data);
            }
            _ => {}
        }

        responses
    }

    /// Request to enable an option.
    pub fn request_enable(&mut self, option: u8) -> TelnetCommand {
        let _response = self.handle_will(option);
        TelnetCommand::Will(option)
    }

    /// Request to disable an option.
    pub fn request_disable(&mut self, option: u8) -> TelnetCommand {
        TelnetCommand::Wont(option)
    }

    /// Handle DO command.
    fn handle_do(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Closed => {
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Will(option)
            }
            OptionState::WantsDisable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::Enabled => {
                TelnetCommand::Will(option)
            }
            OptionState::RemoteWantsEnable => {
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Will(option)
            }
            OptionState::WantsEnable | OptionState::RemoteWantsDisable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Handle DONT command.
    fn handle_dont(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Enabled | OptionState::WantsEnable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::Closed => {
                TelnetCommand::Wont(option)
            }
            OptionState::RemoteWantsDisable | OptionState::WantsDisable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
            OptionState::RemoteWantsEnable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Wont(option)
            }
        }
    }

    /// Handle WILL command.
    fn handle_will(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Closed | OptionState::WantsDisable | OptionState::RemoteWantsDisable => {
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Do(option)
            }
            OptionState::Enabled => {
                TelnetCommand::Do(option)
            }
            OptionState::RemoteWantsEnable | OptionState::WantsEnable => {
                self.set_option_state(option, OptionState::Enabled);
                TelnetCommand::Do(option)
            }
        }
    }

    /// Handle WONT command.
    fn handle_wont(&mut self, option: u8) -> TelnetCommand {
        let current_state = self.get_option_state(option);
        
        match current_state {
            OptionState::Enabled | OptionState::WantsEnable | OptionState::RemoteWantsEnable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Dont(option)
            }
            OptionState::Closed => {
                TelnetCommand::Dont(option)
            }
            OptionState::RemoteWantsDisable | OptionState::WantsDisable => {
                self.set_option_state(option, OptionState::Closed);
                TelnetCommand::Dont(option)
            }
        }
    }

    /// Handle subnegotiation.
    fn handle_subnegotiation(&mut self, _option: u8, _data: &[u8]) {
        // Handle subnegotiation if needed
    }

    /// Get all option states.
    pub fn get_all_option_states(&self) -> std::collections::HashMap<u8, OptionState> {
        self.client_state.options.clone()
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.client_state = ClientState::default();
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TelnetCommand;

    #[test]
    fn test_initial_state() {
        let manager = StateManager::new();
        assert_eq!(manager.connection_state(), ConnectionState::Disconnected);
        assert_eq!(manager.get_option_state(1), OptionState::Closed);
    }

    #[test]
    fn test_process_do() {
        let mut manager = StateManager::new();
        let command = TelnetCommand::Do(1);
        let responses = manager.process_command(&command);
        
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], TelnetCommand::Will(1));
        assert!(manager.is_option_enabled(1));
    }

    #[test]
    fn test_process_will() {
        let mut manager = StateManager::new();
        let command = TelnetCommand::Will(1);
        let responses = manager.process_command(&command);
        
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], TelnetCommand::Do(1));
        assert!(manager.is_option_enabled(1));
    }

    #[test]
    fn test_process_dont() {
        let mut manager = StateManager::new();
        manager.set_option_state(1, OptionState::Enabled);
        let command = TelnetCommand::Dont(1);
        let responses = manager.process_command(&command);
        
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], TelnetCommand::Wont(1));
        assert!(!manager.is_option_enabled(1));
    }

    #[test]
    fn test_process_wont() {
        let mut manager = StateManager::new();
        manager.set_option_state(1, OptionState::Enabled);
        let command = TelnetCommand::Wont(1);
        let responses = manager.process_command(&command);
        
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], TelnetCommand::Dont(1));
        assert!(!manager.is_option_enabled(1));
    }

    #[test]
    fn test_process_subnegotiation() {
        let mut manager = StateManager::new();
        let command = TelnetCommand::Subnegotiation {
            option: 24,
            data: vec![0, 65, 78, 83, 73], // IS ANSI
        };
        let responses = manager.process_command(&command);
        
        assert!(responses.is_empty());
    }

    #[test]
    fn test_process_non_option_command() {
        let mut manager = StateManager::new();
        let command = TelnetCommand::Nop;
        let responses = manager.process_command(&command);
        
        assert!(responses.is_empty());
    }

    #[test]
    fn test_multiple_options() {
        let mut manager = StateManager::new();
        
        let responses1 = manager.process_command(&TelnetCommand::Do(1));
        let responses2 = manager.process_command(&TelnetCommand::Do(2));
        
        assert_eq!(responses1[0], TelnetCommand::Will(1));
        assert_eq!(responses2[0], TelnetCommand::Will(2));
        assert!(manager.is_option_enabled(1));
        assert!(manager.is_option_enabled(2));
    }

    #[test]
    fn test_reset() {
        let mut manager = StateManager::new();
        manager.set_option_state(1, OptionState::Enabled);
        manager.reset();
        
        assert_eq!(manager.get_option_state(1), OptionState::Closed);
    }
}