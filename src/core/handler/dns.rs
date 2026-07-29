use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use chrono::Duration;
use tokio::net::UdpSocket;

const QUERY_TIMEOUT: Duration = Duration::seconds(2);

// encode domain names into byte array format for dns lookup.
pub fn encode_domain_name(domain: &str) -> Result<Vec<u8>, anyhow::Error> {
    if domain.is_empty() {
        return Ok(vec![0]);
    }

    if domain.len() > u8::MAX as usize {
        anyhow::bail!("domain name exceeds {} bytes: {domain}", u8::MAX);
    }

    let mut result: Vec<u8> = Vec::with_capacity(domain.len() + 2); // allocate extra 2 spots for beginning and trailing 0.

    for word in domain.split('.') {
        debug_assert!(word.len() <= 63, "DNS label exceeds 63 bytes: {word}");
        result.push(word.len() as u8);
        result.extend_from_slice(word.as_bytes());
    }

    result.push(0_u8);

    Ok(result)
}

/*
 *  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 *  |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
 *  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 */

#[repr(u16)]
#[allow(dead_code)]
pub enum DnsRecordType {
    A = 1,
    AAAA = 28,
    NS = 2,
    CNAME = 5,
    TXT = 16,
}

impl DnsRecordType {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::A),
            28 => Some(Self::AAAA),
            2 => Some(Self::NS),
            5 => Some(Self::CNAME),
            16 => Some(Self::TXT),
            _ => None,
        }
    }
}

pub fn build_query_packet(encoded_domain: &[u8], query_type: DnsRecordType) -> (Vec<u8>, u16) {
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

pub fn parse_dns_response(response: Vec<u8>, dns_id: u16) -> Result<Ipv4Addr, anyhow::Error> {
    let mut cursor = 0_usize;

    let response_id = read_u16(&response, &mut cursor)?;
    if response_id != dns_id {
        anyhow::bail!("response id mismatch: got {response_id}, expected {dns_id}");
    }

    let response_flags = read_u16(&response, &mut cursor)?;
    let qr = (response_flags >> 15) & 0x1;
    if qr != 1 {
        anyhow::bail!("expected a DNS response, but QR bit indicates a query");
    }

    let rcode = response_flags & 0x000f;
    if rcode != 0 {
        anyhow::bail!("DNS server returned error code: {rcode}");
    }

    let qdcount = read_u16(&response, &mut cursor)?;
    let ancount = read_u16(&response, &mut cursor)?;
    let _nscount = read_u16(&response, &mut cursor)?;
    let _arcount = read_u16(&response, &mut cursor)?;

    // skip the entire question section, as we only care about the answer section
    for _ in 0..qdcount {
        skip_name(&response, &mut cursor)?;
        let _qtype = read_u16(&response, &mut cursor)?;
        let _qclass = read_u16(&response, &mut cursor)?;
    }

    if ancount == 0 {
        anyhow::bail!("DNS response contained no answers");
    }

    for _ in 0..ancount {
        skip_name(&response, &mut cursor)?;
        let record = read_u16(&response, &mut cursor)?;
        let _class = read_u16(&response, &mut cursor)?;
        let _ttl = read_u32(&response, &mut cursor)?;
        let rdlength = read_u16(&response, &mut cursor)? as usize;

        if cursor + rdlength > response.len() {
            anyhow::bail!("answer record RDATA runs past end of response");
        }
        let rdata = &response[cursor..cursor + rdlength];
        cursor += rdlength;

        if let Some(DnsRecordType::A) = DnsRecordType::from_u16(record) {
            if rdlength != 4 {
                anyhow::bail!("A record RDATA length was {rdlength}, expected 4");
            }
            return Ok(Ipv4Addr::new(rdata[0], rdata[1], rdata[2], rdata[3]));
        }
    }

    Err(anyhow::anyhow!("no A record found in DNS response"))
}

fn skip_name(data: &[u8], cursor: &mut usize) -> Result<(), anyhow::Error> {
    loop {
        let len = *data
            .get(*cursor)
            .ok_or_else(|| anyhow::anyhow!("cursor out of bounds"))?;

        if len == 0 {
            *cursor += 1;
            return Ok(());
        }

        if len & 0xC0 == 0xC0 {
            if *cursor + 2 > data.len() {
                anyhow::bail!("cursor out of bounds");
            }
            *cursor += 2;
            return Ok(());
        }

        let label_start = *cursor + 1;
        let label_end = label_start + len as usize;
        if label_end > data.len() {
            anyhow::bail!("cursor out of bounds");
        }
        *cursor = label_end;
    }
}

fn read_u16(data: &[u8], cursor: &mut usize) -> Result<u16, anyhow::Error> {
    if *cursor + 2 > data.len() {
        return Err(anyhow::anyhow!("cursor out of bounds"));
    }
    let value = u16::from_be_bytes([data[*cursor], data[*cursor + 1]]);
    *cursor += 2;
    Ok(value)
}

fn read_u32(data: &[u8], cursor: &mut usize) -> Result<u32, anyhow::Error> {
    if *cursor + 4 > data.len() {
        return Err(anyhow::anyhow!("cursor out of bounds"));
    }
    let value = u32::from_be_bytes([
        data[*cursor],
        data[*cursor + 1],
        data[*cursor + 2],
        data[*cursor + 3],
    ]);
    *cursor += 4;
    Ok(value)
}

async fn query_server(
    encoded_domain: &[u8],
    server: Ipv4Addr,
    query_type: DnsRecordType,
) -> Result<Ipv4Addr, anyhow::Error> {
    let (query, tx_id) = build_query_packet(encoded_domain, query_type);
    let transaction = send_dns_packet(&query, server).await?;
    let result = parse_dns_response(transaction, tx_id)?;

    Ok(result)
}

pub async fn resolve(domain: &str, servers: &[Ipv4Addr]) -> Result<IpAddr, anyhow::Error> {
    let encoded_domain = encode_domain_name(domain)?;

    if servers.is_empty() {
        anyhow::bail!("no DNS servers configured");
    }

    let mut errors = Vec::with_capacity(servers.len());

    for &server in servers {
        match query_server(&encoded_domain, server, DnsRecordType::A).await {
            Ok(ip) => return Ok(IpAddr::V4(ip)),
            Err(err) => errors.push(format!("{server}: {err}")),
        }
    }

    Err(anyhow::anyhow!(
        "all DNS servers failed: {}",
        errors.join("; ")
    ))
}

pub struct DnsProbe {
    pub server: Ipv4Addr,
    pub result: Result<Ipv4Addr, anyhow::Error>,
    pub elapsed: std::time::Duration,
}

pub async fn compare_resolvers(domain: &str, servers: &[Ipv4Addr]) -> Vec<DnsProbe> {
    let mut set = tokio::task::JoinSet::new();

    let domain = match encode_domain_name(domain) {
        Ok(encoded) => Arc::new(encoded),
        Err(_) => return Vec::new(),
    };

    for &server in servers {
        let domain_clone = Arc::clone(&domain);
        set.spawn(async move {
            let started = std::time::Instant::now();
            let result = query_server(&domain_clone, server, DnsRecordType::A).await;
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
