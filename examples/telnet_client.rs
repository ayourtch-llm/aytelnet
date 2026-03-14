//! TELNET Client Example
//!
//! A TELNET client with escape mode (Ctrl-]) for commands.
//!
//! Usage:
//!   cargo run --example telnet_client <host> [port]
//!
//! Examples:
//!   cargo run --example telnet_client example.com 23
//!   cargo run --example telnet_client 192.168.1.1
//!   cargo run --example telnet_client [::1] 23
//!
//! Features:
//!   - Immediate character echo (sends characters as typed)
//!   - Escape mode with Ctrl-]
//!   - Commands: quit, help, status

use aytelnet::{TelnetConnection, TelnetEvent, OPT_BINARY};
use std::env;
use std::error::Error;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const ESCAPE_CHAR: u8 = 0x1d; // Ctrl-]

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <host> [port]", args[0]);
        eprintln!("Example: {} example.com 23", args[0]);
        std::process::exit(1);
    }
    
    let host = &args[1];
    let port: u16 = args.get(2)
        .map(|s| s.parse().unwrap_or(23))
        .unwrap_or(23);
    
    println!("Connecting to {}:{}...", host, port);
    
    // Connect to the TELNET server
    let mut client = TelnetConnection::connect(host, port).await?;
    println!("Connected!");
    
    // Negotiate common options
    println!("Negotiating options...");
    
    // Request to suppress GA (Go Ahead)
    client.negotiate_option(aytelnet::OPT_SUPPRESS_GA, true).await?;
    
    // Request binary mode (disable character interpretation)
    client.negotiate_option(OPT_BINARY, true).await?;
    
    println!("Options negotiated!");
    
    // Main event loop
    println!("\n--- TELNET Session ---");
    println!("Type Ctrl-] to enter escape mode");
    println!("Commands in escape mode: help, quit, status");
    println!("========================\n");
    
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    
    // Escape mode state machine
    let mut escape_mode = false;
    let mut escape_buffer = String::new();
    let mut escape_data_buffer = Vec::new();
    
    loop {
        tokio::select! {
            // Handle incoming TELNET events
            event = client.receive() => {
                match event {
                    Ok(TelnetEvent::Data(data)) => {
                        if escape_mode {
                            // Collect data in escape mode
                            escape_data_buffer.extend_from_slice(&data);
                            // Check if we have a complete command
                            if escape_data_buffer.ends_with(b"\n") || escape_data_buffer.ends_with(b"\r") {
                                if let Ok(text) = String::from_utf8(escape_data_buffer.clone()) {
                                    let cmd = text.trim();
                                    handle_escape_command(cmd).await?;
                                }
                                escape_data_buffer.clear();
                                escape_buffer.clear();
                            }
                        } else {
                            // Print received data normally
                            if let Ok(text) = String::from_utf8(data.clone()) {
                                print!("{}", text);
                                std::io::stdout().flush().unwrap();
                            }
                        }
                    }
                    Ok(TelnetEvent::Command(cmd)) => {
                        if !escape_mode {
                            println!("[Command: {:?}]", cmd);
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    Ok(TelnetEvent::OptionNegotiated { option, enabled }) => {
                        if !escape_mode {
                            println!("[Option {:02x?}: {}]", option, if enabled { "enabled" } else { "disabled" });
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    Ok(TelnetEvent::Closed) => {
                        if !escape_mode {
                            println!("\n[Connection closed]");
                        }
                        break;
                    }
                    Ok(TelnetEvent::Error(e)) => {
                        if !escape_mode {
                            println!("\n[Error: {}]", e);
                        }
                        break;
                    }
                    Err(e) => {
                        if !escape_mode {
                            println!("\n[Error: {}]", e);
                        }
                        break;
                    }
                }
            }
            
            // Handle user input
            result = stdin_reader.read_line(&mut escape_buffer) => {
                match result {
                    Ok(0) => {
                        // EOF
                        break;
                    }
                    Ok(_) => {
                        if escape_mode {
                            // Escape mode: we handle commands via Data events
                            continue;
                        }
                        
                        let input = escape_buffer.clone();
                        escape_buffer.clear();
                        
                        // Check for escape character (Ctrl-])
                        if input.as_bytes().last() == Some(&ESCAPE_CHAR) {
                            println!("\n[Escape mode - type 'help' for commands, 'quit' to exit]\n");
                            std::io::stdout().flush().unwrap();
                            escape_mode = true;
                            escape_buffer.clear();
                            continue;
                        }
                        
                        // Send input to server (without the newline)
                        let send_data = if input.ends_with('\n') || input.ends_with('\r') {
                            &input[..input.len()-1]
                        } else {
                            &input
                        };
                        
                        if !send_data.is_empty() {
                            if let Err(e) = client.send(send_data.as_bytes()).await {
                                eprintln!("Send error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        }
    }
    
    // Disconnect
    client.disconnect().await?;
    println!("Disconnected from server.");
    
    Ok(())
}

// Handle escape mode commands
async fn handle_escape_command(cmd: &str) -> Result<(), Box<dyn Error>> {
    let cmd_lower = cmd.to_lowercase();
    
    match cmd_lower.as_str() {
        "quit" | "exit" => {
            println!("\nDisconnecting...");
            std::process::exit(0);
        }
        "help" | "?" => {
            println!("\n=== TELNET Commands ===");
            println!("  quit    - Disconnect and exit");
            println!("  help    - Show this help message");
            println!("  status  - Show connection status");
            println!("  escape  - Return to normal mode");
            println!("========================\n");
        }
        "status" => {
            println!("\n[Connection Status]");
            println!("  Connected to TELNET server");
            println!("  Use 'quit' to disconnect");
            println!("  Use 'escape' to return to normal mode");
            println!("========================\n");
        }
        "escape" => {
            println!("\n[Normal mode resumed]\n");
        }
        _ => {
            println!("\n[Unknown command: {}]", cmd);
            println!("Type 'help' for available commands\n");
        }
    }
    
    Ok(())
}