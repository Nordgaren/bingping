use pnet::packet::icmp::{IcmpTypes, MutableIcmpPacket};
use pnet::packet::MutablePacket;

use crate::display::load_ascii_art;

// Create an ICMP packet with ASCII art data
pub fn create_icmp_packet(buffer: &mut [u8], sequence: u16, identifier: u16, size: usize) -> usize {
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
        
        // Get ASCII art and use it as payload
        let ascii_art = load_ascii_art();
        let ascii_bytes = ascii_art.as_bytes();
        
        // Calculate how much of the ASCII art we can fit
        let max_payload_size = echo_request_buffer.len() - 4; // Reserve 4 bytes for ID and seq
        let art_size = ascii_bytes.len().min(max_payload_size);
        
        // Copy as much of the ASCII art as will fit in the payload
        echo_request_buffer[4..4+art_size].copy_from_slice(&ascii_bytes[0..art_size]);
        
        // Fill any remaining space with a pattern
        for i in (4 + art_size)..echo_request_buffer.len() {
            echo_request_buffer[i] = b'#';
        }
    }
    
    // Calculate and set the checksum
    let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
    icmp_packet.set_checksum(checksum);
    
    println!("Created ICMP Echo Request with ASCII art: ID={}, Seq={}, Checksum={:x}", 
             identifier, sequence, checksum);
    
    size
} 