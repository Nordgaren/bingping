use std::net::IpAddr;
use anyhow::{Context, Result, anyhow};
use dns_lookup::lookup_host;
use clap::Parser;

use crate::args::Args;

// Configuration for the ping operation
pub struct PingConfig {
    pub destination: String,
    pub ip_addr: IpAddr,
    pub count: Option<u16>,
    pub packet_size: usize,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub ttl: u8,
    pub quiet: bool,
    pub rainbow: bool,
}

// Parse command line arguments into a unified PingConfig
pub fn parse_args() -> Result<PingConfig> {
    let args = Args::parse();
    
    // Resolve hostname to IP address
    let host_addresses = lookup_host(&args.destination)
        .with_context(|| format!("Failed to resolve hostname: {}", args.destination))?;
    
    // First try to find an IPv4 address, then fall back to any IP address
    let ip_addr = host_addresses.iter()
        .find(|ip| ip.is_ipv4())
        .copied()
        .or_else(|| host_addresses.get(0).copied())
        .ok_or_else(|| anyhow!("No IP addresses found for host: {}", args.destination))?;
    
    // Get configuration values, using defaults if not specified
    #[cfg(target_os = "linux")]
    let (count, packet_size, interval_ms, timeout_ms, ttl, quiet, rainbow) = (
        args.count,
        args.size.map(|s| s as usize).unwrap_or(4096),
        (args.interval.unwrap_or(1.0) * 1000.0) as u64,
        (args.timeout.unwrap_or(4.0) * 1000.0) as u64,
        args.ttl.unwrap_or(64),
        args.quiet,
        args.rainbow,
    );
    
    #[cfg(target_os = "windows")]
    let (count, packet_size, interval_ms, timeout_ms, ttl, quiet, rainbow) = (
        args.count,
        args.size.map(|s| s as usize).unwrap_or(4096),
        (args.interval.unwrap_or(1.0) * 1000.0) as u64,
        (args.timeout.unwrap_or(4.0) * 1000.0) as u64,
        args.ttl.unwrap_or(128),
        false,
        args.rainbow,
    );
    
    Ok(PingConfig {
        destination: args.destination,
        ip_addr,
        count,
        packet_size,
        interval_ms,
        timeout_ms,
        ttl,
        quiet,
        rainbow,
    })
} 