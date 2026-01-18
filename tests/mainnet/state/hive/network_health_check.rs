use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::Packet;
use pnet::transport::{transport_channel, TransportChannelType::Layer3};
use pnet::transport::TransportProtocol::Ipv4;

const ICMP_HEADER_SIZE: usize = 8;
const PAYLOAD_SIZE: usize = 32;

pub struct NetworkProbe {
    target: IpAddr,
    timeout: Duration,
    sequence: u16,
}

impl NetworkProbe {
    pub fn new(target: IpAddr) -> Self {
        NetworkProbe {
            target,
            timeout: Duration::from_secs(2),
            sequence: 0,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn send_ping(&mut self) -> Result<bool, String> {
        let total_len = ICMP_HEADER_SIZE + PAYLOAD_SIZE;
        let mut buffer = vec![0u8; total_len];
        let mut packet = MutableEchoRequestPacket::new(&mut buffer).ok_or("Failed to create packet")?;

        packet.set_icmp_type(IcmpTypes::EchoRequest);
        packet.set_identifier(1234);
        packet.set_sequence_number(self.sequence);
        self.sequence = self.sequence.wrapping_add(1);

        let checksum = pnet::packet::icmp::checksum(&packet.to_immutable());
        packet.set_checksum(checksum);

        let (mut tx, _) = transport_channel(1024, Layer3(Ipv4(Ipv4(0))))
            .map_err(|e| format!("Channel creation failed: {}", e))?;

        let dest = match self.target {
            IpAddr::V4(addr) => addr,
            IpAddr::V6(_) => return Err("IPv6 not supported in this example".to_string()),
        };

        tx.send_to(packet, IpAddr::V4(dest))
            .map_err(|e| format!("Send failed: {}", e))?;

        Ok(true)
    }

    pub fn check_connectivity(&mut self, attempts: usize) -> f64 {
        let mut successful = 0;
        for _ in 0..attempts {
            if self.send_ping().is_ok() {
                successful += 1;
            }
            std::thread::sleep(self.timeout);
        }
        (successful as f64 / attempts as f64) * 100.0
    }
}

pub fn probe_network_host(host: &str) -> Result<f64, String> {
    let addr: IpAddr = host.parse().map_err(|e| format!("Invalid address: {}", e))?;
    let mut probe = NetworkProbe::new(addr);
    let success_rate = probe.check_connectivity(4);
    Ok(success_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_probe() {
        let result = probe_network_host("127.0.0.1");
        assert!(result.is_ok());
        let rate = result.unwrap();
        println!("Localhost connectivity: {:.1}%", rate);
    }
}