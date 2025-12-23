use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: Protocol,
    payload: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl NetworkPacket {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: Protocol,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        Self {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp,
        }
    }

    pub fn is_suspicious(&self) -> bool {
        matches!(self.protocol, Protocol::ICMP)
            || self.payload.len() > 1500
            || self.source_ip.is_private() != self.destination_ip.is_private()
    }

    pub fn protocol_name(&self) -> String {
        match self.protocol {
            Protocol::TCP => "TCP".to_string(),
            Protocol::UDP => "UDP".to_string(),
            Protocol::ICMP => "ICMP".to_string(),
            Protocol::Other(code) => format!("Protocol_{}", code),
        }
    }
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_distribution: HashMap<String, usize>,
    suspicious_packets: Vec<NetworkPacket>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        Self {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            suspicious_packets: Vec::new(),
        }
    }

    pub fn process_packet(&mut self, packet: NetworkPacket) {
        self.packet_count += 1;

        let protocol_name = packet.protocol_name();
        *self.protocol_distribution.entry(protocol_name).or_insert(0) += 1;

        if packet.is_suspicious() {
            self.suspicious_packets.push(packet);
        }
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        PacketStatistics {
            total_packets: self.packet_count,
            suspicious_count: self.suspicious_packets.len(),
            protocol_distribution: self.protocol_distribution.clone(),
        }
    }

    pub fn filter_by_protocol(&self, protocol: Protocol) -> Vec<&NetworkPacket> {
        self.suspicious_packets
            .iter()
            .filter(|p| p.protocol == protocol)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub suspicious_count: usize,
    pub protocol_distribution: HashMap<String, usize>,
}

impl PacketStatistics {
    pub fn print_summary(&self) {
        println!("Total packets analyzed: {}", self.total_packets);
        println!("Suspicious packets detected: {}", self.suspicious_count);
        println!("Protocol distribution:");
        for (protocol, count) in &self.protocol_distribution {
            println!("  {}: {}", protocol, count);
        }
    }
}

pub fn parse_raw_packet_data(data: &[u8]) -> Option<NetworkPacket> {
    if data.len() < 20 {
        return None;
    }

    let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let destination_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    let protocol_byte = data[9];

    let protocol = match protocol_byte {
        6 => Protocol::TCP,
        17 => Protocol::UDP,
        1 => Protocol::ICMP,
        other => Protocol::Other(other),
    };

    let payload = data[20..].to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Some(NetworkPacket::new(
        source_ip,
        destination_ip,
        protocol,
        payload,
        timestamp,
    ))
}