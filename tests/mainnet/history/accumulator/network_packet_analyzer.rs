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

    pub fn get_statistics(&self) -> PacketStats {
        PacketStats {
            total_packets: self.packet_count,
            unique_protocols: self.protocol_stats.len(),
            unique_ips: self.ip_traffic.len(),
            most_common_protocol: self.find_most_common_protocol(),
            top_talker: self.find_top_talker(),
        }
    }

    fn find_most_common_protocol(&self) -> Option<(u8, usize)> {
        self.protocol_stats
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&protocol, &count)| (protocol, count))
    }

    fn find_top_talker(&self) -> Option<(Ipv4Addr, usize)> {
        self.ip_traffic
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&ip, &count)| (ip, count))
    }
}

#[derive(Debug)]
pub struct PacketStats {
    pub total_packets: usize,
    pub unique_protocols: usize,
    pub unique_ips: usize,
    pub most_common_protocol: Option<(u8, usize)>,
    pub top_talker: Option<(Ipv4Addr, usize)>,
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
    }
}