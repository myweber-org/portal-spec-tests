use std::net::UdpSocket;
use std::str;

struct PacketInfo {
    source_port: u16,
    dest_port: u16,
    payload: Vec<u8>,
    protocol_type: Protocol,
}

#[derive(Debug, PartialEq)]
enum Protocol {
    HTTP,
    DNS,
    UNKNOWN,
}

impl PacketInfo {
    fn new(source_port: u16, dest_port: u16, payload: Vec<u8>) -> Self {
        let protocol_type = PacketInfo::detect_protocol(source_port, dest_port, &payload);
        PacketInfo {
            source_port,
            dest_port,
            payload,
            protocol_type,
        }
    }

    fn detect_protocol(source_port: u16, dest_port: u16, payload: &[u8]) -> Protocol {
        match (source_port, dest_port) {
            (53, _) | (_, 53) => Protocol::DNS,
            (80, _) | (_, 80) | (443, _) | (_, 443) => Protocol::HTTP,
            _ => {
                if PacketInfo::contains_http_keywords(payload) {
                    Protocol::HTTP
                } else {
                    Protocol::UNKNOWN
                }
            }
        }
    }

    fn contains_http_keywords(payload: &[u8]) -> bool {
        let keywords = ["GET", "POST", "HTTP", "Host:"];
        if let Ok(payload_str) = str::from_utf8(payload) {
            keywords.iter().any(|&kw| payload_str.contains(kw))
        } else {
            false
        }
    }

    fn display(&self) {
        println!("Packet Analysis:");
        println!("  Source Port: {}", self.source_port);
        println!("  Destination Port: {}", self.dest_port);
        println!("  Protocol: {:?}", self.protocol_type);
        println!("  Payload Length: {} bytes", self.payload.len());
        
        if let Ok(payload_str) = str::from_utf8(&self.payload) {
            if !payload_str.is_empty() {
                println!("  Payload Preview: {}", 
                    payload_str.chars().take(50).collect::<String>());
            }
        }
    }
}

fn capture_packets(interface: &str, port: u16) -> std::io::Result<()> {
    let bind_addr = format!("{}:{}", interface, port);
    let socket = UdpSocket::bind(bind_addr)?;
    println!("Listening on {}:{}", interface, port);
    
    let mut buffer = [0; 1024];
    
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, src_addr)) => {
                let payload = buffer[..size].to_vec();
                let packet = PacketInfo::new(
                    src_addr.port(),
                    port,
                    payload
                );
                packet.display();
                println!("  From: {}", src_addr);
                println!("{}", "-".repeat(40));
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn main() {
    let interface = "127.0.0.1";
    let port = 8080;
    
    if let Err(e) = capture_packets(interface, port) {
        eprintln!("Failed to capture packets: {}", e);
    }
}