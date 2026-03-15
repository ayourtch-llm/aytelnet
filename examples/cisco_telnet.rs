//! CiscoTelnet Interactive CLI Example
//!
//! A CLI example for connecting to Cisco devices with automated login.
//!
//! Usage:
//!   cargo run --example cisco_telnet <address> <username> <password>
//!
//! Examples:
//!   cargo run --example cisco_telnet 192.168.1.1 admin secret
//!   cargo run --example cisco_telnet [::1] admin secret
//!   cargo run --example cisco_telnet router.local admin secret
//!
//! Features:
//!   - Automatic username/password authentication
//!   - Interactive shell after login
//!   - Escape mode with Ctrl-]
//!   - Custom prompt detection

use aytelnet::cisco_telnet::CiscoTelnet;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <address> <username> <password>", args[0]);
        eprintln!("Example: {} 192.168.1.1 admin secret", args[0]);
        eprintln!("         {} router.local admin secret", args[0]);
        eprintln!("         {} [::1] admin secret", args[0]);
        std::process::exit(1);
    }
    
    let address = &args[1];
    let username = &args[2];
    let password = &args[3];
    
    println!("Connecting to {} as {}...", address, username);
    
    // Create CiscoTelnet client
    let mut client = CiscoTelnet::new(address, username, password)
        .with_timeout(Duration::from_secs(30))
        .with_read_timeout(Duration::from_secs(10))
        .with_prompts(&["Router*", "Switch*", "ASA*", "Firepower*", "config"]);
    
    // Connect and authenticate
    client.connect().await?;
    println!("Connected and authenticated!");
    
    // Enter interactive mode
    println!("\n--- Cisco Device Shell ---");
    println!("Type Ctrl-] to enter escape mode");
    println!("Commands in escape mode: help, quit, status, escape");
    println!("============================\n");
    
    // Escape mode state machine
    let mut escape_mode = false;
    let mut escape_buffer = String::new();
    let mut escape_data_buffer = Vec::new();
    
    // Set up terminal for raw mode
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    
    loop {
        // Use select to handle both keyboard input and server events
        tokio::select! {
            // Handle incoming data
            event = client.receive_line() => {
                match event {
                    Ok(line) => {
                        if escape_mode {
                            // Collect data in escape mode
                            escape_data_buffer.extend_from_slice(line.as_bytes());
                            // Check if we have a complete command
                            if escape_data_buffer.ends_with(b"\n") || escape_data_buffer.ends_with(b"\r") {
                                if let Ok(text) = String::from_utf8(escape_data_buffer.clone()) {
                                    let cmd = text.trim();
                                    handle_escape_command(cmd, &mut io::stdout(), &mut escape_buffer, &mut client).await?;
                                }
                                escape_data_buffer.clear();
                                escape_buffer.clear();
                            }
                        } else {
                            // Print received line normally
                            print!("{}", line);
                            io::stdout().flush().unwrap();
                        }
                    }
                    Err(e) => {
                        if !escape_mode {
                            println!("\n[Error: {}]", e);
                        }
                        break;
                    }
                }
            }
            
            // Handle keyboard input (non-blocking)
            result = tokio::task::spawn_blocking(|| {
                // Try to read an event with a timeout
                if event::poll(std::time::Duration::from_millis(10)).unwrap() {
                    Some(event::read().unwrap())
                } else {
                    None
                }
            }) => {
                if let Some(event) = result.ok().flatten() {
                    match event {
                        Event::Key(key) => {
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }
                            
                            // Handle escape mode
                            if escape_mode {
                                match key.code {
                                    KeyCode::Esc => {
                                        // Escape from escape mode
                                        escape_mode = false;
                                        escape_buffer.clear();
                                        let _ = execute!(io::stdout(), Print("\n[Normal mode resumed]\n"));
                                        let _ = io::stdout().flush();
                                    }
                                    KeyCode::Enter => {
                                        // Execute escape command
                                        let cmd = escape_buffer.trim().to_string();
                                        handle_escape_command(&cmd, &mut io::stdout(), &mut escape_buffer, &mut client).await?;
                                    }
                                    KeyCode::Backspace | KeyCode::Delete => {
                                        if !escape_buffer.is_empty() {
                                            escape_buffer.pop();
                                            let _ = execute!(io::stdout(), Clear(ClearType::UntilNewLine));
                                            let _ = execute!(io::stdout(), cursor::MoveToColumn(0));
                                            if !escape_buffer.is_empty() {
                                                let _ = execute!(io::stdout(), Print(escape_buffer.as_str()));
                                            }
                                            let _ = io::stdout().flush();
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        escape_buffer.push(c);
                                        let _ = execute!(io::stdout(), Print(c));
                                        let _ = io::stdout().flush();
                                    }
                                    _ => {}
                                }
                                continue;
                            }
                            
                            // Normal mode: check for escape character (Ctrl-])
                            if key.code == KeyCode::Char(']') {
                                escape_mode = true;
                                escape_buffer.clear();
                                let _ = execute!(io::stdout(), Print("\n[Escape mode - type 'help' for commands, 'quit' to exit]\n"));
                                let _ = io::stdout().flush();
                                continue;
                            }
                            
                            // Normal mode: send character immediately with local echo
                            match key.code {
                                KeyCode::Enter => {
                                    let _ = execute!(io::stdout(), Print("\n"));
                                    let _ = io::stdout().flush();
                                    // Send newline to server
                                    if let Err(e) = client.send(&[b'\n']).await {
                                        eprintln!("Send error: {}", e);
                                    }
                                }
                                KeyCode::Backspace | KeyCode::Delete => {
                                    // Show backspace effect locally
                                    let _ = execute!(io::stdout(), cursor::MoveLeft(1));
                                    let _ = execute!(io::stdout(), Print(" "));
                                    let _ = execute!(io::stdout(), cursor::MoveLeft(1));
                                    let _ = io::stdout().flush();
                                    // Send backspace character (0x7f) to server
                                    if let Err(e) = client.send(&[0x7f]).await {
                                        eprintln!("Send error: {}", e);
                                    }
                                }
                                KeyCode::Char(c) => {
                                    // Show character locally (echo)
                                    let _ = execute!(io::stdout(), Print(c));
                                    let _ = io::stdout().flush();
                                    // Send character to server
                                    if let Err(e) = client.send(c.to_string().as_bytes()).await {
                                        eprintln!("Send error: {}", e);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Clean up terminal
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    // Disconnect
    client.disconnect().await?;
    println!("Disconnected from device.");
    
    Ok(())
}

// Handle escape mode commands
async fn handle_escape_command(
    cmd: &str,
    stdout: &mut io::Stdout,
    escape_buffer: &mut String,
    client: &mut CiscoTelnet,
) -> Result<(), Box<dyn Error>> {
    let cmd_lower = cmd.to_lowercase();
    
    match cmd_lower.as_str() {
        "quit" | "exit" => {
            println!("\nDisconnecting...");
            client.disconnect().await?;
            println!("Disconnected from device.");
            std::process::exit(0);
        }
        "help" | "?" => {
            println!("\n=== Cisco Telnet Commands ===");
            println!("  quit    - Disconnect and exit");
            println!("  help    - Show this help message");
            println!("  status  - Show connection status");
            println!("  escape  - Return to normal mode");
            println!("================================\n");
            escape_buffer.clear();
        }
        "status" => {
            println!("\n[Connection Status]");
            println!("  Connected to Cisco device");
            println!("  Logged in: {}", client.is_logged_in());
            println!("  Use 'quit' to disconnect");
            println!("  Use 'escape' to return to normal mode");
            println!("================================\n");
            escape_buffer.clear();
        }
        "escape" => {
            println!("\n[Normal mode resumed]\n");
            escape_buffer.clear();
        }
        _ => {
            println!("\n[Unknown command: {}]", cmd);
            println!("Type 'help' for available commands\n");
            escape_buffer.clear();
        }
    }
    
    let _ = stdout.flush();
    Ok(())
}
