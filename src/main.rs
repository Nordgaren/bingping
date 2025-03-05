use std::process::Command;

use clap::Parser;
use dns_lookup::lookup_host;
use anyhow::{Context, Result, anyhow};

#[cfg(target_os = "linux")]
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Target host to ping
    destination: String,

    /// Number of packets to send
    #[clap(short = 'c', long = "count")]
    count: Option<u16>,

    /// Number of bytes to send
    #[clap(short = 's', long = "size")]
    size: Option<u16>,

    /// Interval between sending packets (in seconds)
    #[clap(short = 'i', long = "interval")]
    interval: Option<f64>,

    /// Timeout before giving up (in seconds)
    #[clap(short = 'W', long = "timeout")]
    timeout: Option<f64>,

    /// Time to live
    #[clap(short, long)]
    ttl: Option<u8>,

    /// Wait time between pings (in ms)
    #[clap(short = 'p', long = "pattern")]
    pattern: Option<String>,

    /// Wait time between pings (in ms)
    #[clap(short = 'q', long = "quiet")]
    quiet: bool,

    /// Verbose output
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,

    /// Audible ping
    #[clap(short = 'a', long = "audible")]
    audible: bool,

    /// Bypass route using socket options
    #[clap(short = 'b')]
    bypass_route: bool,

    /// Numeric output only (no DNS resolution)
    #[clap(short = 'n', long = "numeric")]
    numeric: bool,

    /// Timestamp display
    #[clap(short = 'D', long = "timestamp")]
    timestamp: bool,

    /// Flood ping
    #[clap(short = 'f', long = "flood")]
    flood: bool,
}

#[cfg(target_os = "windows")]
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Target host to ping
    destination: String,

    /// Number of packets to send
    #[clap(short = 'n', long = "count")]
    count: Option<u16>,

    /// Number of bytes to send
    #[clap(short = 'l', long = "size")]
    size: Option<u16>,

    /// Interval between sending packets (in seconds)
    #[clap(short = 'i', long = "interval")]
    interval: Option<f64>,

    /// Timeout before giving up (in seconds)
    #[clap(short = 'w', long = "timeout")]
    timeout: Option<f64>,

    /// Time to live
    #[clap(short = 'i', long = "ttl")]
    ttl: Option<u8>,

    /// Record route
    #[clap(short = 'r', long = "record-route")]
    record_route: bool,

    /// Timestamp route
    #[clap(short = 's', long = "timestamp")]
    timestamp: bool,

    /// Resolve addresses to hostnames
    #[clap(short = 'a', long = "resolve")]
    resolve: bool,
}

// Configuration for the ping operation
struct PingConfig {
    destination: String,
    count: Option<u16>,
    packet_size: usize,
    interval_ms: u64,
    timeout_ms: u64,
    ttl: u8,
    quiet: bool,
}

// Load ASCII art from file
fn load_ascii_art() -> String {
    include_str!("../ascii-art.txt").to_string()
}

// Parse command line arguments into a unified PingConfig
fn parse_args() -> Result<PingConfig> {
    let args = Args::parse();
    
    // Resolve hostname to IP address
    let host_addresses = lookup_host(&args.destination)
        .with_context(|| format!("Failed to resolve hostname: {}", args.destination))?;
    
    // First try to find an IPv4 address, then fall back to any IP address
    host_addresses.iter()
        .find(|ip| ip.is_ipv4())
        .copied()
        .or_else(|| host_addresses.get(0).copied())
        .ok_or_else(|| anyhow!("No IP addresses found for host: {}", args.destination))?;
    
    // Get configuration values, using defaults if not specified
    #[cfg(target_os = "linux")]
    let (count, packet_size, interval_ms, timeout_ms, ttl, quiet) = (
        args.count,
        args.size.map(|s| s as usize).unwrap_or(56),
        (args.interval.unwrap_or(1.0) * 1000.0) as u64,
        (args.timeout.unwrap_or(4.0) * 1000.0) as u64,
        args.ttl.unwrap_or(64),
        args.quiet,
    );
    
    #[cfg(target_os = "windows")]
    let (count, packet_size, interval_ms, timeout_ms, ttl, quiet) = (
        args.count,
        args.size.map(|s| s as usize).unwrap_or(32),
        (args.interval.unwrap_or(1.0) * 1000.0) as u64,
        (args.timeout.unwrap_or(4.0) * 1000.0) as u64,
        args.ttl.unwrap_or(128),
        false,
    );
    
    Ok(PingConfig {
        destination: args.destination,
        count,
        packet_size,
        interval_ms,
        timeout_ms,
        ttl,
        quiet,
    })
}

fn main() -> Result<()> {
    // Parse command-line arguments into PingConfig
    let config = parse_args()?;
    
    // Print ASCII art
    if !config.quiet {
        println!("{}", load_ascii_art());
    }
    
    // Generate the command based on the platform and arguments
    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("ping");
    #[cfg(target_os = "linux")]
    {
        cmd.arg("-c").arg(config.count.map(|c| c.to_string()).unwrap_or_else(|| "5".to_string()));
        cmd.arg("-s").arg(config.packet_size.to_string());
        cmd.arg("-i").arg((config.interval_ms as f64 / 1000.0).to_string());
        cmd.arg("-W").arg((config.timeout_ms as f64 / 1000.0).to_string());
        cmd.arg("-t").arg(config.ttl.to_string());
        if config.quiet {
            cmd.arg("-q");
        }
        cmd.arg(&config.destination);
    }
    
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("ping");
    #[cfg(target_os = "windows")]
    {
        cmd.arg("-n").arg(config.count.map(|c| c.to_string()).unwrap_or_else(|| "5".to_string()));
        cmd.arg("-l").arg(config.packet_size.to_string());
        cmd.arg("-w").arg(config.timeout_ms.to_string());
        cmd.arg("-i").arg(config.ttl.to_string());
        cmd.arg(&config.destination);
    }
    
    // Execute the ping command
    let status = cmd.status()
        .context("Failed to execute ping command")?;
    
    if !status.success() {
        return Err(anyhow!("Ping command failed with exit code: {:?}", status.code()));
    }
    
    Ok(())
}
