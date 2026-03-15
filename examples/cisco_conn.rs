//! CiscoConn - Simple command execution example
//!
//! This example demonstrates how to use the CiscoConn struct to execute
//! a single command on a Cisco device and get the output.

use aytelnet::cisco_conn::{CiscoConn, ConnectionType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic usage with default timeouts
    println!("=== Basic Usage ===");
    let conn = CiscoConn::new(
        "192.168.1.1",
        ConnectionType::CiscoTelnet,
        "admin",
        "password",
        "show version",
    )?;

    println!("Target: {}", conn.target());
    println!("Command: {}", conn.cmd());
    println!("Username: {}", conn.username());

    // Note: This would actually connect to the device and execute the command
    // For demonstration, we're not executing it as there's no actual device
    // let output = conn.execute().await?;
    // println!("Output: {}", output);

    // Example 2: With custom timeouts
    println!("\n=== With Custom Timeouts ===");
    let conn = CiscoConn::with_timeouts(
        "router.local",
        ConnectionType::CiscoTelnet,
        "cisco",
        "cisco123",
        "show running-config",
        Duration::from_secs(60),  // Connection timeout
        Duration::from_secs(30),  // Read timeout
    )?;

    println!("Target: {}", conn.target());
    println!("Command: {}", conn.cmd());

    // Example 3: IPv6 address
    println!("\n=== IPv6 Address ===");
    let conn = CiscoConn::new(
        "[::1]:2323",
        ConnectionType::CiscoTelnet,
        "admin",
        "password",
        "show interfaces",
    )?;

    println!("Target: {}", conn.target());
    println!("Command: {}", conn.cmd());

    // Example 4: Custom prompts
    println!("\n=== Custom Prompts ===");
    // Note: The CiscoConn struct doesn't expose custom prompts directly,
    // but they can be configured internally if needed
    println!("Custom prompts can be configured in CiscoConnConfig");

    println!("\n=== Example Complete ===");
    println!("To execute a real command, uncomment the conn.execute() call above");

    Ok(())
}
