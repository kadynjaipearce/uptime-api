use std::net::IpAddr;

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

/// Resolves `domain` to an IP address, tracing the lookup manually rather
/// than going through the OS resolver, so the DNS stage's own timing can be
/// captured for `checks.dns_ms`.
pub async fn resolve(domain: &str) -> Result<IpAddr, anyhow::Error> {
    let _ = encode_domain_name(domain);
    unimplemented!()
}
