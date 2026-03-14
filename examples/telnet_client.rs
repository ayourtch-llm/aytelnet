//! TELNET Client Example
//!
//! A simple async TELNET client that connects to a TELNET server.
//!
//! Usage:
//!   cargo run --example telnet_client <host> [port]
//!
//! Examples:
//!   cargo run --example telnet_client example.com 23
//!   cargo run --example telnet_client 192.168.1.1
//!   cargo run --example telnet_client [::1] 23

use aytelnet::{TelnetConnection, TelnetEvent, OPT_ECHO, OPT_BINARY};
use std::env;
use std::error::Error;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

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
    
    // Request to suppress echo locally
    client.negotiate_option(OPT_ECHO, false).await?;
    
    println!("Options negotiated!");
    
    // Main event loop - handle user input and received events
    println!("\n--- TELNET Session ---");
    println!("Type 'quit' to disconnect");
    println!("Press Enter to send a line");
    println!("=====================\n");
    
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    
    loop {
        // Use select to handle both input and server events concurrently
        tokio::select! {
            // Handle incoming TELNET events
            event = client.receive() => {
                match event {
                    Ok(TelnetEvent::Data(data)) => {
                        // Print received data
                        if let Ok(text) = String::from_utf8(data.clone()) {
                            print!("{}", text);
                            std::io::stdout().flush().unwrap();
                        } else {
                            // Print binary data as hex
                            print!("[Binary: ");
                            for byte in data {
                                print!("{:02x} ", byte);
                            }
                            println!("]");
                        }
                    }
                    Ok(TelnetEvent::Command(cmd)) => {
                        println!("\n[Command: {:?}]\n", cmd);
                    }
                    Ok(TelnetEvent::OptionNegotiated { option, enabled }) => {
                        println!("[Option {:02x?}: {}]\n", option, if enabled { "enabled" } else { "disabled" });
                    }
                    Ok(TelnetEvent::Closed) => {
                        println!("\n[Connection closed]");
                        break;
                    }
                    Ok(TelnetEvent::Error(e)) => {
                        println!("\n[Error: {}]", e);
                        break;
                    }
                    Err(e) => {
                        println!("\n[Error: {}]", e);
                        break;
                    }
                }
            }
            
            // Handle user input
            result = read_line(&mut stdin_reader) => {
                match result {
                    Ok(input) => {
                        // Check for quit command
                        if input.trim().to_lowercase() == "quit" {
                            println!("\nDisconnecting...");
                            break;
                        }
                        
                        // Send the input (without the newline for cleaner output)
                        let send_data = if input.ends_with('\n') {
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

// Helper function to read a line from stdin
async fn read_line(reader: &mut BufReader<tokio::io::Stdin>) -> Result<String, std::io::Error> {
    let mut input = String::new();
    reader.read_line(&mut input).await?;
    Ok(input)
}