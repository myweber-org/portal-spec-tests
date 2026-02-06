use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: Protocol,
    pub payload_length: usize,
}

impl PacketHeader {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let version = data[0] >> 4;
        
        match version {
            4 => Self::parse_ipv4(data),
            6 => Self::parse_ipv6(data),
            _ => None,
        }
    }

    fn parse_ipv4(data: &[u8]) -> Option<Self> {
        let ihl = (data[0] & 0x0F) as usize * 4;
        if data.len() < ihl {
            return None;
        }

        let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]).to_string();
        let destination_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]).to_string();
        
        let protocol = match data[9] {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            p => Protocol::Unknown(p),
        };

        let total_length = ((data[2] as usize) << 8) | (data[3] as usize);
        let payload_length = total_length.saturating_sub(ihl);

        Some(Self {
            source_ip,
            destination_ip,
            protocol,
            payload_length,
        })
    }

    fn parse_ipv6(data: &[u8]) -> Option<Self> {
        if data.len() < 40 {
            return None;
        }

        let source_ip = Ipv6Addr::from([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ]).to_string();

        let destination_ip = Ipv6Addr::from([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]).to_string();

        let protocol = match data[6] {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            58 => Protocol::ICMP,
            p => Protocol::Unknown(p),
        };

        let payload_length = ((data[4] as usize) << 8) | (data[5] as usize);

        Some(Self {
            source_ip,
            destination_ip,
            protocol,
            payload_length,
        })
    }
}

pub fn extract_payload(packet: &[u8]) -> Vec<u8> {
    match PacketHeader::from_bytes(packet) {
        Some(header) => {
            let header_length = match header.protocol {
                Protocol::TCP | Protocol::UDP | Protocol::ICMP => {
                    if packet[0] >> 4 == 4 {
                        ((packet[0] & 0x0F) as usize * 4)
                    } else {
                        40
                    }
                }
                Protocol::Unknown(_) => 0,
            };
            
            if header_length > 0 && packet.len() > header_length {
                packet[header_length..].to_vec()
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_tcp_packet() {
        let packet = vec![
            0x45, 0x00, 0x00, 0x34, 0x00, 0x00, 0x40, 0x00,
            0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
            0xc0, 0xa8, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x50, 0x02, 0x20, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74,
        ];

        let header = PacketHeader::from_bytes(&packet).unwrap();
        assert_eq!(header.source_ip, "192.168.1.1");
        assert_eq!(header.destination_ip, "192.168.1.2");
        assert_eq!(header.protocol, Protocol::TCP);
        assert_eq!(header.payload_length, 20);
    }

    #[test]
    fn test_payload_extraction() {
        let packet = vec![
            0x45, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x40, 0x00,
            0x40, 0x11, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
            0xc0, 0xa8, 0x01, 0x02, 0x68, 0x65, 0x6C, 0x6C,
            0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ];

        let payload = extract_payload(&packet);
        assert_eq!(payload, b"hello world");
    }
}