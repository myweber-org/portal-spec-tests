use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: IpAddr,
    destination_ip: IpAddr,
    protocol: Protocol,
    payload: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<Protocol, usize>,
    ip_traffic: HashMap<IpAddr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(packet.protocol.clone()).or_insert(0) += 1;
        *self.ip_traffic.entry(packet.source_ip).or_insert(0) += 1;
        *self.ip_traffic.entry(packet.destination_ip).or_insert(0) += 1;
        
        self.detect_anomalies(packet);
    }

    fn detect_anomalies(&self, packet: &NetworkPacket) {
        if packet.payload.len() > 1500 {
            println!("Warning: Oversized packet detected from {:?}", packet.source_ip);
        }
        
        if packet.source_ip == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
            println!("Alert: Packet with invalid source IP detected");
        }
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        PacketStatistics {
            total_packets: self.packet_count,
            top_protocol: self.find_top_protocol(),
            unique_ips: self.ip_traffic.len(),
        }
    }

    fn find_top_protocol(&self) -> Option<Protocol> {
        self.protocol_stats
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(protocol, _)| protocol.clone())
    }
}

#[derive(Debug)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub top_protocol: Option<Protocol>,
    pub unique_ips: usize,
}

impl Protocol {
    pub fn from_byte(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: Protocol::TCP,
            payload: vec![0u8; 100],
            timestamp: 1234567890,
        };
        
        analyzer.analyze_packet(&packet);
        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.unique_ips, 2);
    }
}