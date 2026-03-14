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
    
    // Request ECHO from server (let server handle echo)
    client.negotiate_option(aytelnet::OPT_ECHO, true).await?;
    
    println!("Options negotiated!");
    
    // Setup terminal in raw mode
    terminal::enable_raw_mode()?;
    
    // Enter alternate screen
    execute!(io::stdout(), EnterAlternateScreen)?;
    
    // Main event loop
    println!("\n--- TELNET Session ---");
    println!("Type Ctrl-] to enter escape mode");
    println!("Commands in escape mode: help, quit, status, escape");
    println!("========================\n");
    
    // Escape mode state machine
    let mut escape_mode = false;
    let mut escape_buffer = String::new();
    let mut escape_data_buffer = Vec::new();
    
    loop {
        // Use select to handle both keyboard input and server events
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
                                    handle_escape_command(cmd, &mut io::stdout(), &mut escape_buffer, &mut client).await?;
                                }
                                escape_data_buffer.clear();
                                escape_buffer.clear();
                            }
                        } else {
                            // Print received data normally
                            if let Ok(text) = String::from_utf8(data.clone()) {
                                // Normalize CRLF to LF for display
                                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                                print!("{}", normalized);
                                io::stdout().flush().unwrap();
                            }
                        }
                    }
                    Ok(TelnetEvent::Command(cmd)) => {
                        if !escape_mode {
                            println!("[Command: {:?}]", cmd);
                            io::stdout().flush().unwrap();
                        }
                    }
                    Ok(TelnetEvent::OptionNegotiated { option, enabled }) => {
                        if !escape_mode {
                            println!("[Option {:02x?}: {}]", option, if enabled { "enabled" } else { "disabled" });
                            io::stdout().flush().unwrap();
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
                                    KeyCode::Backspace => {
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
                            
                            // Normal mode: send character immediately
                            match key.code {
                                KeyCode::Enter => {
                                    let _ = execute!(io::stdout(), Print("\n"));
                                    let _ = io::stdout().flush();
                                    // Send newline to server
                                    if let Err(e) = client.send(&[b'\n']).await {
                                        eprintln!("Send error: {}", e);
                                    }
                                }
                                KeyCode::Backspace => {
                                    // Send backspace character (0x7f)
                                    let _ = execute!(io::stdout(), Print("\x08 \x08"));
                                    let _ = io::stdout().flush();
                                    if let Err(e) = client.send(&[0x7f]).await {
                                        eprintln!("Send error: {}", e);
                                    }
                                }
                                KeyCode::Char(c) => {
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
    
    // Cleanup
    // Leave alternate screen
    execute!(io::stdout(), LeaveAlternateScreen)?;
    
    // Restore terminal
    terminal::disable_raw_mode()?;
    
    // Disconnect
    client.disconnect().await?;
    println!("Disconnected from server.");
    
    Ok(())
}

// Handle escape mode commands
async fn handle_escape_command(
    cmd: &str,
    stdout: &mut io::Stdout,
    escape_buffer: &mut String,
    client: &mut TelnetConnection,
) -> Result<(), Box<dyn Error>> {
    let cmd_lower = cmd.to_lowercase();
    
    match cmd_lower.as_str() {
        "quit" | "exit" => {
            println!("\nDisconnecting...");
            client.disconnect().await?;
            println!("Disconnected from server.");
            std::process::exit(0);
        }
        "help" | "?" => {
            println!("\n=== TELNET Commands ===");
            println!("  quit    - Disconnect and exit");
            println!("  help    - Show this help message");
            println!("  status  - Show connection status");
            println!("  escape  - Return to normal mode");
            println!("========================\n");
            escape_buffer.clear();
        }
        "status" => {
            println!("\n[Connection Status]");
            println!("  Connected to TELNET server");
            println!("  Use 'quit' to disconnect");
            println!("  Use 'escape' to return to normal mode");
            println!("========================\n");
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