use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

impl From<u8> for Protocol {
    fn from(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            id => Protocol::Unknown(id),
        }
    }
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: Ipv4Addr,
    pub destination_ip: Ipv4Addr,
    pub protocol: Protocol,
    pub ttl: u8,
    pub payload_length: usize,
}

pub struct PacketAnalyzer {
    protocol_stats: HashMap<Protocol, u32>,
    total_packets: u64,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            protocol_stats: HashMap::new(),
            total_packets: 0,
        }
    }

    pub fn analyze_packet(&mut self, header: &PacketHeader) {
        self.total_packets += 1;
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;
    }

    pub fn get_protocol_distribution(&self) -> HashMap<Protocol, f64> {
        let mut distribution = HashMap::new();
        
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            distribution.insert(protocol.clone(), percentage);
        }
        
        distribution
    }

    pub fn reset_stats(&mut self) {
        self.protocol_stats.clear();
        self.total_packets = 0;
    }

    pub fn total_packets_analyzed(&self) -> u64 {
        self.total_packets
    }
}

pub fn parse_ipv4_header(data: &[u8]) -> Option<PacketHeader> {
    if data.len() < 20 {
        return None;
    }

    let version_and_ihl = data[0];
    let version = version_and_ihl >> 4;
    
    if version != 4 {
        return None;
    }

    let ttl = data[8];
    let protocol = Protocol::from(data[9]);
    
    let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let destination_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    
    let total_length = ((data[2] as u16) << 8) | data[3] as u16;
    let header_length = (version_and_ihl & 0x0F) as usize * 4;
    let payload_length = total_length as usize - header_length;

    Some(PacketHeader {
        source_ip,
        destination_ip,
        protocol,
        ttl,
        payload_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Protocol::from(6), Protocol::TCP);
        assert_eq!(Protocol::from(17), Protocol::UDP);
        assert_eq!(Protocol::from(1), Protocol::ICMP);
        assert_eq!(Protocol::from(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_analyzer_stats() {
        let mut analyzer = PacketAnalyzer::new();
        
        let header = PacketHeader {
            source_ip: Ipv4Addr::new(192, 168, 1, 1),
            destination_ip: Ipv4Addr::new(192, 168, 1, 2),
            protocol: Protocol::TCP,
            ttl: 64,
            payload_length: 100,
        };

        analyzer.analyze_packet(&header);
        analyzer.analyze_packet(&header);
        
        let distribution = analyzer.get_protocol_distribution();
        assert_eq!(distribution.get(&Protocol::TCP), Some(&100.0));
        assert_eq!(analyzer.total_packets_analyzed(), 2);
    }

    #[test]
    fn test_parse_valid_ipv4_header() {
        let mut data = vec![0u8; 20];
        data[0] = 0x45;
        data[8] = 64;
        data[9] = 6;
        data[12] = 192;
        data[13] = 168;
        data[14] = 1;
        data[15] = 1;
        data[16] = 192;
        data[17] = 168;
        data[18] = 1;
        data[19] = 2;
        data[2] = 0x00;
        data[3] = 40;

        let header = parse_ipv4_header(&data).unwrap();
        assert_eq!(header.source_ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(header.destination_ip, Ipv4Addr::new(192, 168, 1, 2));
        assert_eq!(header.protocol, Protocol::TCP);
        assert_eq!(header.ttl, 64);
        assert_eq!(header.payload_length, 20);
    }
}