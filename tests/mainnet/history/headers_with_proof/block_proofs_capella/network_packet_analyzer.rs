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

    pub fn is_from_ip(&self, ip: &Ipv4Addr) -> bool {
        &self.source_ip == ip
    }

    pub fn is_to_ip(&self, ip: &Ipv4Addr) -> bool {
        &self.destination_ip == ip
    }

    pub fn protocol_matches(&self, protocol: &Protocol) -> bool {
        &self.protocol == protocol
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    pub fn contains_pattern(&self, pattern: &[u8]) -> bool {
        self.payload.windows(pattern.len()).any(|window| window == pattern)
    }
}

pub struct PacketFilter {
    source_ip_filter: Option<Ipv4Addr>,
    destination_ip_filter: Option<Ipv4Addr>,
    protocol_filter: Option<Protocol>,
    min_payload_size: Option<usize>,
    pattern_filter: Option<Vec<u8>>,
}

impl PacketFilter {
    pub fn new() -> Self {
        Self {
            source_ip_filter: None,
            destination_ip_filter: None,
            protocol_filter: None,
            min_payload_size: None,
            pattern_filter: None,
        }
    }

    pub fn set_source_ip_filter(&mut self, ip: Ipv4Addr) -> &mut Self {
        self.source_ip_filter = Some(ip);
        self
    }

    pub fn set_destination_ip_filter(&mut self, ip: Ipv4Addr) -> &mut Self {
        self.destination_ip_filter = Some(ip);
        self
    }

    pub fn set_protocol_filter(&mut self, protocol: Protocol) -> &mut Self {
        self.protocol_filter = Some(protocol);
        self
    }

    pub fn set_min_payload_size(&mut self, size: usize) -> &mut Self {
        self.min_payload_size = Some(size);
        self
    }

    pub fn set_pattern_filter(&mut self, pattern: Vec<u8>) -> &mut Self {
        self.pattern_filter = Some(pattern);
        self
    }

    pub fn matches(&self, packet: &NetworkPacket) -> bool {
        if let Some(ref source_ip) = self.source_ip_filter {
            if !packet.is_from_ip(source_ip) {
                return false;
            }
        }

        if let Some(ref dest_ip) = self.destination_ip_filter {
            if !packet.is_to_ip(dest_ip) {
                return false;
            }
        }

        if let Some(ref protocol) = self.protocol_filter {
            if !packet.protocol_matches(protocol) {
                return false;
            }
        }

        if let Some(min_size) = self.min_payload_size {
            if packet.payload_size() < min_size {
                return false;
            }
        }

        if let Some(ref pattern) = self.pattern_filter {
            if !packet.contains_pattern(pattern) {
                return false;
            }
        }

        true
    }
}

pub fn filter_packets(packets: Vec<NetworkPacket>, filter: &PacketFilter) -> Vec<NetworkPacket> {
    packets
        .into_iter()
        .filter(|packet| filter.matches(packet))
        .collect()
}