use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use chrono::Duration;
use tokio::net::UdpSocket;

const QUERY_TIMEOUT: Duration = Duration::seconds(2);

// encode domain names into byte array format for dns lookup.
pub fn encode_domain_name(domain: &str) -> Vec<u8> {
    if domain.is_empty() {
        return vec![0];
    }

    let mut result: Vec<u8> = Vec::with_capacity(domain.len() + 2); // allocate extra 2 spots for beginning and trailing 0.

    for word in domain.split('.') {
        debug_assert!(word.len() <= 63, "DNS label exceeds 63 bytes: {word}");
        result.push(word.len() as u8);
        result.extend_from_slice(word.as_bytes());
    }

    result.push(0_u8);

    result
}

/*
 *  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 *  |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
 *  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 */

#[repr(u16)]
#[allow(dead_code)]
pub enum DnsQuery {
    A = 1,
    NS = 2,
    CNAME = 5,
}

pub fn build_query_packet(encoded_domain: &[u8], query_type: DnsQuery) -> (Vec<u8>, u16) {
    let random: u16 = rand::random();
    let [high, low] = random.to_be_bytes();
    let [q_hi, q_low] = (query_type as u16).to_be_bytes();

    let header_bytes = [
        high, low, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let type_bytes = [q_hi, q_low, 0x00, 0x01];

    let query = [&header_bytes[..], encoded_domain, &type_bytes].concat();

    (query, random)
}

async fn send_dns_packet(query: &[u8], server: Ipv4Addr) -> Result<Vec<u8>, anyhow::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let socket_addr = SocketAddrV4::new(server, 53);

    socket.send_to(query, socket_addr).await?;

    let mut buffer = [0u8; 512]; // RFC 1035 size. not tested.

    let (amt, src) =
        tokio::time::timeout(QUERY_TIMEOUT.to_std()?, socket.recv_from(&mut buffer)).await??;

    if src.ip() != IpAddr::V4(server) {
        anyhow::bail!("received DNS response from unexpected source {src}, expected {server}");
    }

    Ok(buffer[..amt].to_vec())
}

fn parse_dns_response(_response: Vec<u8>, _id: u16) -> Result<Ipv4Addr, anyhow::Error> {
    unimplemented!()
}

async fn query_server(
    encoded_domain: &[u8],
    server: Ipv4Addr,
    query_type: DnsQuery,
) -> Result<Ipv4Addr, anyhow::Error> {
    let (query, tx_id) = build_query_packet(encoded_domain, query_type);
    let transaction = send_dns_packet(&query, server).await?;
    let _result = parse_dns_response(transaction, tx_id);

    unimplemented!()
}

pub async fn resolve(domain: &str, servers: &[Ipv4Addr]) -> Result<IpAddr, anyhow::Error> {
    let encoded_domain = encode_domain_name(domain);
    let mut last_err = None;

    for &server in servers {
        match query_server(&encoded_domain, server, DnsQuery::A).await {
            Ok(ip) => return Ok(IpAddr::V4(ip)),
            Err(err) => last_err = Some(err),
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no DNS servers configured")))
}

pub struct DnsProbe {
    pub server: Ipv4Addr,
    pub result: Result<Ipv4Addr, anyhow::Error>,
    pub elapsed: std::time::Duration,
}

pub async fn compare_resolvers(domain: &str, servers: &[Ipv4Addr]) -> Vec<DnsProbe> {
    let mut set = tokio::task::JoinSet::new();

    let domain = Arc::new(encode_domain_name(domain));

    for &server in servers {
        let domain_clone = Arc::clone(&domain);
        set.spawn(async move {
            let started = std::time::Instant::now();
            let result = query_server(&domain_clone, server, DnsQuery::A).await;
            DnsProbe {
                server,
                result,
                elapsed: started.elapsed(),
            }
        });
    }

    let mut probes = Vec::with_capacity(servers.len());
    while let Some(res) = set.join_next().await {
        if let Ok(probe) = res {
            probes.push(probe);
        }
    }

    probes
}
