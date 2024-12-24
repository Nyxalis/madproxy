use proxy_protocol::{
    version2::{ProxyAddresses, ProxyCommand, ProxyTransportProtocol},
    ProxyHeader, encode,
};
use std::net::SocketAddr;

pub struct ProxyProtocol {
    client_addr: SocketAddr,
    dest_addr: SocketAddr,
}

impl ProxyProtocol {
    pub fn new(client_addr: SocketAddr, dest_addr: SocketAddr) -> Self {
        Self {
            client_addr,
            dest_addr,
        }
    }

    pub fn generate_header(&self) -> Vec<u8> {
        let proxy_addr = match (self.client_addr, self.dest_addr) {
            (SocketAddr::V4(source), SocketAddr::V4(destination)) => ProxyAddresses::Ipv4 {
                source,
                destination,
            },
            (SocketAddr::V6(source), SocketAddr::V6(destination)) => ProxyAddresses::Ipv6 {
                source,
                destination,
            },
            _ => panic!("Mixed IP versions are not supported"),
        };

        encode(ProxyHeader::Version2 {
            command: ProxyCommand::Proxy,
            transport_protocol: ProxyTransportProtocol::Stream,
            addresses: proxy_addr,
        })
        .expect("Failed to encode PROXY protocol header")
        .to_vec()
    }
} 