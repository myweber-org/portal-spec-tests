use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

impl NetworkPacket {
    pub fn new(raw_data: &[u8]) -> Option<Self> {
        if raw_data.len() < 20 {
            return None;
        }

        let version = (raw_data[0] >> 4) & 0x0F;
        
        match version {
            4 => Self::parse_ipv4_packet(raw_data),
            6 => Self::parse_ipv6_packet(raw_data),
            _ => None,
        }
    }

    fn parse_ipv4_packet(data: &[u8]) -> Option<Self> {
        let header_length = ((data[0] & 0x0F) as usize) * 4;
        if data.len() < header_length {
            return None;
        }

        let source_ip = IpAddr::V4(Ipv4Addr::new(
            data[12], data[13], data[14], data[15]
        ));
        
        let destination_ip = IpAddr::V4(Ipv4Addr::new(
            data[16], data[17], data[18], data[19]
        ));

        let protocol = match data[9] {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            p => Protocol::Unknown(p),
        };

        let payload = data[header_length..].to_vec();
        
        Some(Self {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn parse_ipv6_packet(data: &[u8]) -> Option<Self> {
        if data.len() < 40 {
            return None;
        }

        let source_bytes: [u8; 16] = data[8..24].try_into().ok()?;
        let destination_bytes: [u8; 16] = data[24..40].try_into().ok()?;
        
        let source_ip = IpAddr::V6(Ipv6Addr::from(source_bytes));
        let destination_ip = IpAddr::V6(Ipv6Addr::from(destination_bytes));

        let next_header = data[6];
        let protocol = match next_header {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            58 => Protocol::ICMP,
            p => Protocol::Unknown(p),
        };

        let payload = data[40..].to_vec();
        
        Some(Self {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    pub fn extract_http_info(&self) -> Option<HashMap<String, String>> {
        if self.protocol != Protocol::TCP {
            return None;
        }

        let payload_str = String::from_utf8_lossy(&self.payload);
        if !payload_str.contains("HTTP") {
            return None;
        }

        let mut headers = HashMap::new();
        let lines: Vec<&str> = payload_str.split("\r\n").collect();
        
        for line in lines {
            if line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    headers.insert(
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string()
                    );
                }
            }
        }

        Some(headers)
    }

    pub fn is_suspicious(&self) -> bool {
        match self.protocol {
            Protocol::ICMP => self.payload.len() > 1024,
            Protocol::Unknown(p) => p > 143,
            _ => false,
        }
    }
}

pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    suspicious_packets: Vec<NetworkPacket>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        Self {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            suspicious_packets: Vec::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: NetworkPacket) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(packet.protocol.clone()).or_insert(0) += 1;
        
        if packet.is_suspicious() {
            self.suspicious_packets.push(packet.clone());
        }

        if let Some(http_headers) = packet.extract_http_info() {
            println!("HTTP packet detected from {:?}", packet.source_ip);
            for (key, value) in http_headers {
                println!("  {}: {}", key, value);
            }
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        
        stats.insert("total_packets".to_string(), self.packet_count.to_string());
        stats.insert("suspicious_packets".to_string(), self.suspicious_packets.len().to_string());
        
        for (protocol, count) in &self.protocol_stats {
            let protocol_name = match protocol {
                Protocol::TCP => "TCP".to_string(),
                Protocol::UDP => "UDP".to_string(),
                Protocol::ICMP => "ICMP".to_string(),
                Protocol::Unknown(p) => format!("Unknown({})", p),
            };
            stats.insert(protocol_name, count.to_string());
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_packet_parsing() {
        let mut raw_data = vec![0x45, 0x00, 0x00, 0x1C];
        raw_data.extend(vec![0; 8]);
        raw_data.push(0x06);
        raw_data.extend(vec![0; 2]);
        raw_data.extend(vec![192, 168, 1, 1]);
        raw_data.extend(vec![192, 168, 1, 2]);
        raw_data.extend(vec![0x01, 0x02, 0x03, 0x04]);

        let packet = NetworkPacket::new(&raw_data).unwrap();
        
        assert_eq!(packet.source_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(packet.destination_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)));
        assert_eq!(packet.protocol, Protocol::TCP);
        assert_eq!(packet.payload, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_suspicious_detection() {
        let packet = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            protocol: Protocol::ICMP,
            payload: vec![0; 1025],
            timestamp: 1234567890,
        };

        assert!(packet.is_suspicious());
    }
}