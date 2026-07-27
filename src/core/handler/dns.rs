use std::{
    net::{IpAddr, Ipv4Addr},
    u8,
};

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

pub fn build_query_packet(encoded_domain: Vec<u8>, query_type: DnsQuery) -> (Vec<u8>, u16) {
    let random: u16 = rand::random();
    let [high, low] = random.to_be_bytes();
    let [q_hi, q_low] = (query_type as u16).to_be_bytes();

    let header_bytes = [
        high, low, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let type_bytes = [q_hi, q_low, 0x00, 0x01];

    let query = [&header_bytes[..], &encoded_domain, &type_bytes].concat();

    (query, random)
}

async fn send_dns_packet(query: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    unimplemented!()
}

fn parse_dns_response(response: Vec<u8>, id: u16) -> Result<Ipv4Addr, anyhow::Error> {
    unimplemented!()
}

/// Resolves `domain` to an IP address, tracing the lookup manually rather
/// than going through the OS resolver, so the DNS stage's own timing can be
/// captured for `checks.dns_ms`.
pub async fn resolve(domain: &str) -> Result<IpAddr, anyhow::Error> {
    let encoded_domain = encode_domain_name(domain);
    let (query, id) = build_query_packet(encoded_domain, DnsQuery::A);
    let response = send_dns_packet(query).await?;
    let ip = parse_dns_response(response, id)?;

    Ok(IpAddr::V4(ip))
}
