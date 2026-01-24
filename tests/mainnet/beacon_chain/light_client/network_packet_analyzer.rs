use std::net::UdpSocket;
use std::time::Duration;

#[derive(Debug)]
struct PacketHeader {
    source_port: u16,
    destination_port: u16,
    length: u16,
    checksum: u16,
}

impl PacketHeader {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        
        Some(PacketHeader {
            source_port: u16::from_be_bytes([data[0], data[1]]),
            destination_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        })
    }
    
    fn validate_checksum(&self, payload: &[u8]) -> bool {
        let mut sum: u32 = 0;
        
        sum += self.source_port as u32;
        sum += self.destination_port as u32;
        sum += self.length as u32;
        
        for chunk in payload.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]]) as u32
            } else {
                (chunk[0] as u32) << 8
            };
            sum += word;
        }
        
        while sum > 0xFFFF {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        let computed_checksum = !(sum as u16);
        computed_checksum == self.checksum
    }
}

fn analyze_packet(data: &[u8]) {
    match PacketHeader::from_bytes(data) {
        Some(header) => {
            println!("Packet Analysis:");
            println!("  Source Port: {}", header.source_port);
            println!("  Destination Port: {}", header.destination_port);
            println!("  Length: {} bytes", header.length);
            println!("  Checksum: 0x{:04X}", header.checksum);
            
            let payload = &data[8..];
            if header.validate_checksum(payload) {
                println!("  Checksum: VALID");
            } else {
                println!("  Checksum: INVALID");
            }
            
            println!("  Payload Size: {} bytes", payload.len());
            if !payload.is_empty() {
                println!("  First 16 bytes of payload: {:02X?}", &payload[..payload.len().min(16)]);
            }
        }
        None => println!("Invalid packet: insufficient data"),
    }
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    
    println!("Listening for UDP packets on {}", socket.local_addr()?);
    
    let mut buffer = [0u8; 1024];
    
    match socket.recv_from(&mut buffer) {
        Ok((size, source)) => {
            println!("Received {} bytes from {}", size, source);
            analyze_packet(&buffer[..size]);
        }
        Err(e) => {
            println!("No packets received: {}", e);
        }
    }
    
    Ok(())
}