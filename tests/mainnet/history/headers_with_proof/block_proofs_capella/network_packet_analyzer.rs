use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: u8,
    payload: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug)]
pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<u8, usize>,
    ip_traffic: HashMap<Ipv4Addr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;

        *self.protocol_stats.entry(packet.protocol).or_insert(0) += 1;
        *self.ip_traffic.entry(packet.source_ip).or_insert(0) += 1;
        *self.ip_traffic.entry(packet.destination_ip).or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> AnalyzerStats {
        let mut top_protocols: Vec<(u8, usize)> = self.protocol_stats
            .iter()
            .map(|(&proto, &count)| (proto, count))
            .collect();
        
        top_protocols.sort_by(|a, b| b.1.cmp(&a.1));
        
        let mut top_ips: Vec<(Ipv4Addr, usize)> = self.ip_traffic
            .iter()
            .map(|(&ip, &count)| (ip, count))
            .collect();
        
        top_ips.sort_by(|a, b| b.1.cmp(&a.1));

        AnalyzerStats {
            total_packets: self.packet_count,
            unique_protocols: self.protocol_stats.len(),
            unique_ips: self.ip_traffic.len(),
            top_protocols: top_protocols.into_iter().take(5).collect(),
            top_ips: top_ips.into_iter().take(10).collect(),
        }
    }

    pub fn reset(&mut self) {
        self.packet_count = 0;
        self.protocol_stats.clear();
        self.ip_traffic.clear();
    }
}

#[derive(Debug)]
pub struct AnalyzerStats {
    pub total_packets: usize,
    pub unique_protocols: usize,
    pub unique_ips: usize,
    pub top_protocols: Vec<(u8, usize)>,
    pub top_ips: Vec<(Ipv4Addr, usize)>,
}

impl NetworkPacket {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: u8,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        NetworkPacket {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.payload.is_empty() && self.timestamp > 0
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet1 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            6,
            vec![1, 2, 3, 4, 5],
            1234567890,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 2),
            Ipv4Addr::new(192, 168, 1, 1),
            17,
            vec![6, 7, 8, 9, 10],
            1234567891,
        );

        analyzer.process_packet(&packet1);
        analyzer.process_packet(&packet2);

        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.unique_protocols, 2);
        assert_eq!(stats.unique_ips, 2);
    }

    #[test]
    fn test_packet_validation() {
        let valid_packet = NetworkPacket::new(
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 2),
            1,
            vec![1, 2, 3],
            1000,
        );

        let invalid_packet = NetworkPacket::new(
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 2),
            1,
            vec![],
            0,
        );

        assert!(valid_packet.is_valid());
        assert!(!invalid_packet.is_valid());
        assert_eq!(valid_packet.payload_size(), 3);
    }
}use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub payload_size: usize,
}

pub struct PacketAnalyzer {
    protocol_counts: HashMap<Protocol, u32>,
    total_packets: u64,
    total_bytes: u64,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            protocol_counts: HashMap::new(),
            total_packets: 0,
            total_bytes: 0,
        }
    }

    pub fn analyze_packet(&mut self, header: &PacketHeader) {
        self.total_packets += 1;
        self.total_bytes += header.payload_size as u64;

        let count = self.protocol_counts.entry(header.protocol.clone()).or_insert(0);
        *count += 1;
    }

    pub fn get_protocol_distribution(&self) -> HashMap<Protocol, f64> {
        let mut distribution = HashMap::new();
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            distribution.insert(protocol.clone(), percentage);
        }
        
        distribution
    }

    pub fn get_average_packet_size(&self) -> f64 {
        if self.total_packets == 0 {
            0.0
        } else {
            self.total_bytes as f64 / self.total_packets as f64
        }
    }

    pub fn reset(&mut self) {
        self.protocol_counts.clear();
        self.total_packets = 0;
        self.total_bytes = 0;
    }
}

impl Protocol {
    pub fn from_u8(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(value),
        }
    }
}

pub fn parse_ipv4_address(bytes: [u8; 4]) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_from_u8() {
        assert_eq!(Protocol::from_u8(6), Protocol::TCP);
        assert_eq!(Protocol::from_u8(17), Protocol::UDP);
        assert_eq!(Protocol::from_u8(1), Protocol::ICMP);
        assert_eq!(Protocol::from_u8(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();
        
        let header1 = PacketHeader {
            source_ip: parse_ipv4_address([192, 168, 1, 1]),
            destination_ip: parse_ipv4_address([192, 168, 1, 2]),
            protocol: Protocol::TCP,
            payload_size: 1500,
        };

        let header2 = PacketHeader {
            source_ip: parse_ipv4_address([192, 168, 1, 2]),
            destination_ip: parse_ipv4_address([192, 168, 1, 1]),
            protocol: Protocol::UDP,
            payload_size: 512,
        };

        analyzer.analyze_packet(&header1);
        analyzer.analyze_packet(&header2);
        analyzer.analyze_packet(&header1);

        let distribution = analyzer.get_protocol_distribution();
        assert_eq!(distribution.get(&Protocol::TCP).unwrap(), &66.66666666666667);
        assert_eq!(distribution.get(&Protocol::UDP).unwrap(), &33.333333333333336);
        assert_eq!(analyzer.total_packets, 3);
        assert_eq!(analyzer.total_bytes, 3512);
        assert_eq!(analyzer.get_average_packet_size(), 1170.6666666666667);
    }
}