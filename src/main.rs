use std::io;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::collections::{VecDeque, HashMap};
use std::process::Command;

use clap::Parser;
use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet_transport::{transport_channel, TransportChannelType};
use pnet_transport::TransportProtocol::Ipv4;
use pnet_transport::icmp_packet_iter;
use dns_lookup::lookup_host;
use rand::random;
use anyhow::{Context, Result, anyhow};
use crossbeam_channel::{select, tick};
use ctrlc;

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
    ip_addr: IpAddr,
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

// Statistics for the ping operation
struct PingStats {
    packets_sent: u64,
    packets_received: u64,
    rtt_sum: f64,
    rtt_min: f64,
    rtt_max: f64,
    rtt_history: VecDeque<f64>,
}

impl PingStats {
    fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            rtt_sum: 0.0,
            rtt_min: f64::MAX,
            rtt_max: 0.0,
            rtt_history: VecDeque::with_capacity(100),
        }
    }

    fn update(&mut self, rtt: f64) {
        self.packets_received += 1;
        self.rtt_sum += rtt;
        self.rtt_min = self.rtt_min.min(rtt);
        self.rtt_max = self.rtt_max.max(rtt);
        self.rtt_history.push_back(rtt);

        if self.rtt_history.len() > 100 {
            self.rtt_history.pop_front();
        }
    }

    fn avg_rtt(&self) -> f64 {
        if self.packets_received == 0 {
            0.0
        } else {
            self.rtt_sum / self.packets_received as f64
        }
    }

    fn packet_loss(&self) -> f64 {
        if self.packets_sent == 0 {
            0.0
        } else {
            (self.packets_sent - self.packets_received) as f64 / self.packets_sent as f64 * 100.0
        }
    }
}

// Create an ICMP echo request packet
fn create_icmp_packet(buffer: &mut [u8], sequence: u16, identifier: u16, size: usize) -> usize {
    // Clear the buffer first
    buffer.iter_mut().for_each(|b| *b = 0);
    
    // Create the ICMP packet
    let mut icmp_packet = MutableIcmpPacket::new(buffer).unwrap();
    
    // Set ICMP type to echo request
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode(0));
    
    // Set echo request data
    {
        let echo_request_buffer = icmp_packet.payload_mut();
        
        // Set identifier (byte 0-1)
        echo_request_buffer[0] = (identifier >> 8) as u8;
        echo_request_buffer[1] = (identifier & 0xFF) as u8;
        
        // Set sequence number (byte 2-3)
        echo_request_buffer[2] = (sequence >> 8) as u8;
        echo_request_buffer[3] = (sequence & 0xFF) as u8;
        
        // Fill the rest of the payload with pattern data
        for i in 4..echo_request_buffer.len() {
            echo_request_buffer[i] = (i % 256) as u8;
        }
    }
    
    // Calculate and set the checksum
    let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
    icmp_packet.set_checksum(checksum);
    
    println!("Created ICMP Echo Request: ID={}, Seq={}, Checksum={:x}", 
             identifier, sequence, checksum);
    
    size
}

// Parse command line arguments into a unified PingConfig
fn parse_args() -> Result<PingConfig> {
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
        ip_addr,
        count,
        packet_size,
        interval_ms,
        timeout_ms,
        ttl,
        quiet,
    })
}

// Execute ping with system command as fallback
fn execute_system_ping(config: &PingConfig) -> Result<()> {
    println!("Falling back to system ping command (raw sockets require root privileges)");
    
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
    
    let status = cmd.status()
        .context("Failed to execute ping command")?;
    
    if !status.success() {
        return Err(anyhow!("Ping command failed with exit code: {:?}", status.code()));
    }
    
    Ok(())
}

// Ping implementation using raw sockets
fn ping_with_raw_sockets(config: &PingConfig) -> Result<()> {
    let ip_addr = config.ip_addr;
    let packet_size = config.packet_size;
    let ttl = config.ttl;
    let destination = config.destination.clone();
    
    // Generate random identifier
    let identifier = (random::<u16>() % 65535) as u16;
    
    println!("Using ICMP identifier: {}", identifier);
    
    // Create transport channel for ICMP
    // Protocol 1 is ICMP for IPv4
    let protocol = Ipv4(pnet::packet::ip::IpNextHeaderProtocol(1));
    println!("Creating transport channel for ICMP (protocol number: 1)");
    
    let (mut tx, mut rx) = match transport_channel(4096, TransportChannelType::Layer4(protocol)) {
        Ok((tx, rx)) => {
            println!("Successfully created transport channel");
            (tx, rx)
        },
        Err(e) => {
            eprintln!("Error creating transport channel: {}", e);
            eprintln!("This is likely due to permissions - try running with sudo or as administrator");
            return Err(anyhow!("Failed to create ICMP socket: {}", e));
        }
    };
    
    // Statistics
    let stats = Arc::new(Mutex::new(PingStats::new()));
    let stats_clone = Arc::clone(&stats);
    
    // Running flag
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);
    
    // Set up CTRL+C handler
    let running_ctrlc = Arc::clone(&running);
    ctrlc::set_handler(move || {
        running_ctrlc.store(false, Ordering::SeqCst);
    }).context("Failed to set CTRL+C handler")?;
    
    // Create sequence number counter
    let sequence = Arc::new(AtomicU64::new(0));
    
    // Map to store send timestamps for each sequence number
    let send_times = Arc::new(Mutex::new(HashMap::new()));
    let send_times_clone = Arc::clone(&send_times);
    
    // Print header
    println!("PING {} ({}) {} bytes of data.", 
             destination, ip_addr, packet_size);
    
    // Launch receiver thread
    let receiver_thread = thread::spawn(move || {
        let mut icmp_iter = icmp_packet_iter(&mut rx);
        
        println!("Receiver thread started, waiting for packets...");
        
        // Create a timeout channel for periodic wakeups
        let timeout = tick(Duration::from_millis(100));
        
        while running_clone.load(Ordering::Relaxed) {
            // Use select to either process a packet or timeout
            select! {
                recv(timeout) -> _ => {
                    // Just a timeout tick to check if we should exit
                    if !running_clone.load(Ordering::Relaxed) {
                        println!("Receiver thread detected exit signal");
                        break;
                    }
                }
                default => {
                    // Try to receive a packet with a non-blocking approach
                    match icmp_iter.next() {
                        Ok((packet, addr)) => {
                            let recv_time = Instant::now();
                            
                            // Print packet type for debugging
                            println!("Received ICMP packet type: {:?} from {}", packet.get_icmp_type(), addr);
                            
                            if packet.get_icmp_type() == IcmpTypes::EchoReply {
                                // Extract the payload which should contain our identifier and sequence
                                let payload = packet.payload();
                                if payload.len() >= 4 { // Need at least 4 bytes for ID and seq
                                    let reply_id = ((payload[0] as u16) << 8) | (payload[1] as u16);
                                    let reply_seq = ((payload[2] as u16) << 8) | (payload[3] as u16);
                                    
                                    println!("  - Packet ID: {}, Sequence: {}, Expected ID: {}", 
                                             reply_id, reply_seq, identifier);
                                    
                                    if reply_id == identifier {
                                        // Calculate round-trip time
                                        let mut send_times = send_times_clone.lock().unwrap();
                                        if let Some(send_time) = send_times.remove(&(reply_seq as u64)) {
                                            let rtt = recv_time.duration_since(send_time).as_secs_f64() * 1000.0;
                                            
                                            println!("{} bytes from {}: icmp_seq={} ttl={} time={:.1} ms",
                                                    packet_size, addr, reply_seq, ttl, rtt);
                                            
                                            // Update statistics
                                            let mut stats = stats_clone.lock().unwrap();
                                            stats.update(rtt);
                                        } else {
                                            println!("  - No send time found for sequence {}", reply_seq);
                                        }
                                    } else {
                                        println!("  - Ignoring packet with wrong identifier");
                                    }
                                } else {
                                    println!("  - Packet payload too short: {} bytes", payload.len());
                                }
                            }
                        },
                        Err(e) => {
                            if e.kind() != io::ErrorKind::TimedOut && e.kind() != io::ErrorKind::WouldBlock {
                                eprintln!("Error receiving packet: {}", e);
                            }
                            // Short sleep to prevent CPU spinning on repeated errors
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
            }
        }
        
        println!("Receiver thread shutting down");
    });
    
    // Set up timer for sending packets
    let ticker = tick(Duration::from_millis(config.interval_ms));
    
    // Prepare ICMP packet buffer
    let mut packet_buffer = vec![0u8; packet_size + 8]; // 8 bytes for ICMP header
    
    // Main loop: send packets and print statistics
    let start_time = Instant::now();
    let mut total_sent = 0;
    
    loop {
        select! {
            recv(ticker) -> _ => {
                // Increment sequence number and wrap around at 65535
                let seq = (sequence.fetch_add(1, Ordering::SeqCst) % 65535) as u16;
                
                // Store send time
                {
                    let mut send_times = send_times.lock().unwrap();
                    send_times.insert(seq as u64, Instant::now());
                }
                
                // Create ICMP packet
                create_icmp_packet(&mut packet_buffer, seq, identifier, packet_size);
                
                // Send the packet
                println!("Sending ICMP packet with seq={}", seq);
                match tx.send_to(MutableIcmpPacket::new(&mut packet_buffer).unwrap(), ip_addr) {
                    Ok(bytes_sent) => {
                        println!("Sent {} bytes to {}", bytes_sent, ip_addr);
                        // Update statistics
                        let mut stats = stats.lock().unwrap();
                        stats.packets_sent += 1;
                        total_sent += 1;
                    },
                    Err(e) => {
                        eprintln!("Error sending packet: {}", e);
                    }
                }
                
                // Check if we've sent enough packets
                if let Some(count) = config.count {
                    if total_sent >= count as u64 {
                        println!("Sent requested number of packets ({}), signaling exit", count);
                        running.store(false, Ordering::SeqCst);
                        // Allow some time for the last packet's reply to be received
                        thread::sleep(Duration::from_millis(500));
                        break;
                    }
                }
            }
        }
        
        // Break if running flag is false (e.g., due to CTRL+C)
        if !running.load(Ordering::Relaxed) {
            println!("Main thread detected exit signal");
            break;
        }
    }
    
    // Signal the receiver thread to exit
    running.store(false, Ordering::SeqCst);
    println!("Waiting for receiver thread to finish...");
    
    // Wait for receiver thread to finish
    match receiver_thread.join() {
        Ok(_) => println!("Receiver thread joined successfully"),
        Err(e) => eprintln!("Error joining receiver thread: {:?}", e),
    }
    
    // Print statistics
    let stats = stats.lock().unwrap();
    let elapsed = start_time.elapsed().as_secs_f64();
    
    println!("\n--- {} ping statistics ---", destination);
    println!("{} packets transmitted, {} received, {:.1}% packet loss, time {:.0}ms",
             stats.packets_sent, stats.packets_received, stats.packet_loss(), elapsed * 1000.0);
    
    if stats.packets_received > 0 {
        println!("rtt min/avg/max = {:.3}/{:.3}/{:.3} ms",
                 stats.rtt_min, stats.avg_rtt(), stats.rtt_max);
    }
    
    Ok(())
}

fn main() -> Result<()> {
    // Parse command-line arguments into PingConfig
    let config = parse_args()?;
    
    // Print ASCII art
    if !config.quiet {
        println!("{}", load_ascii_art());
    }
    
    // Try to use raw sockets for custom ping implementation
    match ping_with_raw_sockets(&config) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If the error is due to permissions (usually for raw sockets), fall back to system ping
            if let Some(io_err) = e.root_cause().downcast_ref::<std::io::Error>() {
                if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                    // Permission denied, likely due to not running as root
                    execute_system_ping(&config)
                } else {
                    // Some other error occurred
                    Err(e)
                }
            } else {
                // Not an IO error
                Err(e)
            }
        }
    }
}
