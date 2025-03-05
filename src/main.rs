use anyhow::Result;

mod args;
mod config;
mod stats;
mod display;
mod packet;
mod ping;

use crate::config::parse_args;
use crate::display::{load_ascii_art, pink_text, rainbow_text};
use crate::ping::{ping_with_raw_sockets, execute_system_ping};

fn main() -> Result<()> {
    // Parse command-line arguments into PingConfig
    let config = parse_args()?;
    
    // Print ASCII art
    if !config.quiet {
        let ascii_art = load_ascii_art();
        if config.rainbow {
            // Use rainbow colors
            println!("{}", rainbow_text(&ascii_art));
        } else {
            // Use pink color
            println!("{}", pink_text(&ascii_art));
        }
    }
    
    // Try to use raw sockets for custom ping implementation
    match ping_with_raw_sockets(&config) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If the error is due to permissions (usually for raw sockets), fall back to system ping
            if let Some(io_err) = e.root_cause().downcast_ref::<std::io::Error>() {
                eprintln!("Failed to use raw sockets: {}", io_err);
                eprintln!("Falling back to system ping command");
                execute_system_ping(&config)
            } else {
                // For other errors, just return the error
                Err(e)
            }
        }
    }
}
