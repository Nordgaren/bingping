use std::io;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::collections::HashMap;
use std::process::Command;

use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::Packet;
use pnet_transport::{transport_channel, TransportChannelType};
use pnet_transport::TransportProtocol::Ipv4;
use pnet_transport::icmp_packet_iter;
use anyhow::{Context, Result, anyhow};
use crossbeam_channel::{select, tick};
use rand::random;

use crate::config::PingConfig;
use crate::stats::PingStats;
use crate::packet::create_icmp_packet;
use crate::display::{rainbow_text, pink_text};

// Execute a system ping command (fallback if raw sockets not available)
pub fn execute_system_ping(config: &PingConfig) -> Result<()> {
    let mut cmd = Command::new("ping");
    
    // Add appropriate arguments based on operating system
    #[cfg(target_os = "linux")]
    {
        cmd.arg(config.ip_addr.to_string());
        
        if let Some(count) = config.count {
            cmd.args(["-c", &count.to_string()]);
        }
        
        let size = config.packet_size;
        cmd.args(["-s", &size.to_string()]);
        
        let interval = config.interval_ms as f64 / 1000.0;
        cmd.args(["-i", &interval.to_string()]);
        
        let timeout = config.timeout_ms as f64 / 1000.0;
        cmd.args(["-W", &timeout.to_string()]);
        
        cmd.args(["-t", &config.ttl.to_string()]);
    }
    
    #[cfg(target_os = "windows")]
    {
        cmd.arg(config.ip_addr.to_string());
        
        if let Some(count) = config.count {
            cmd.args(["-n", &count.to_string()]);
        }
        
        let size = config.packet_size;
        cmd.args(["-l", &size.to_string()]);
        
        let timeout = config.timeout_ms;
        cmd.args(["-w", &timeout.to_string()]);
        
        cmd.args(["-i", &config.ttl.to_string()]);
    }
    
    // Execute the ping command and stream output
    let mut child = cmd.spawn().context("Failed to execute ping command")?;
    let status = child.wait().context("Failed to wait for ping command to complete")?;
    
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Ping command failed with status: {}", status))
    }
}

// Ping implementation using raw sockets
pub fn ping_with_raw_sockets(config: &PingConfig) -> Result<()> {
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
    
    // Pass a copy of the config to the receiver thread
    let rainbow = config.rainbow;
    
    // Print header
    println!("PING {} ({}) {} bytes of data.", 
             destination, ip_addr, packet_size);
    
    // Launch receiver thread
    let receiver_thread = thread::spawn(move || {
        let mut icmp_iter = icmp_packet_iter(&mut rx);
        
        println!("Receiver thread started, waiting for packets...");
        
        while running_clone.load(Ordering::Relaxed) {
            // Create a ticker for heartbeat
            let ticker = tick(Duration::from_secs(1));
            
            select! {
                recv(ticker) -> _ => {
                    // Just a heartbeat to check for exit flag
                },
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
                                            
                                            // Check if we received ASCII art in the reply
                                            if payload.len() > 4 {
                                                // Try to extract ASCII art from the payload
                                                // Start after the ID and sequence bytes
                                                let art_data = &payload[4..];
                                                
                                                // Convert to string, ignoring non-printable characters
                                                let art_string = art_data.iter()
                                                    .filter(|&&b| b.is_ascii() && (b.is_ascii_graphic() || b == b' ' || b == b'\n'))
                                                    .map(|&b| b as char)
                                                    .collect::<String>();
                                                
                                                // If we got something back (some servers just send zeros)
                                                if !art_string.trim().is_empty() {
                                                    println!("Received ASCII art in reply:");
                                                    if rainbow {
                                                        // Rainbow colors
                                                        println!("{}", rainbow_text(&art_string));
                                                    } else {
                                                        // Pink color
                                                        println!("{}", pink_text(&art_string));
                                                    }
                                                }
                                            }
                                            
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
                        }
                    }
                }
            }
        }
        
        println!("Receiver thread shutting down");
    });
    
    // Start time for overall statistics
    let start_time = Instant::now();
    
    // Main send loop
    let mut total_sent = 0;
    let mut prev_send_time = Instant::now().checked_sub(Duration::from_millis(config.interval_ms)).unwrap_or_else(Instant::now);
    
    // Buffer for packet
    let mut packet_buffer = vec![0u8; packet_size + 8]; // 8 bytes for ICMP header
    
    while running.load(Ordering::Relaxed) {
        // Check for interval timing
        let now = Instant::now();
        let elapsed = now.duration_since(prev_send_time);
        
        if elapsed.as_millis() as u64 >= config.interval_ms {
            prev_send_time = now;
            
            // Increment sequence number
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
        
        // Break if running flag is false (e.g., due to CTRL+C)
        if !running.load(Ordering::Relaxed) {
            println!("Main thread detected exit signal");
            break;
        }
        
        // Small sleep to avoid burning CPU
        thread::sleep(Duration::from_millis(10));
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