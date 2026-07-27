use uptime_api::core::handler::dns::{DnsQuery, build_query_packet, encode_domain_name};

#[test]
fn encodes_two_labels() {
    let encoded = encode_domain_name("example.com");
    let mut expected = vec![7];
    expected.extend_from_slice(b"example");
    expected.push(3);
    expected.extend_from_slice(b"com");
    expected.push(0);
    assert_eq!(encoded, expected);
}

#[test]
fn encodes_single_label() {
    let encoded = encode_domain_name("localhost");
    let mut expected = vec![9];
    expected.extend_from_slice(b"localhost");
    expected.push(0);
    assert_eq!(encoded, expected);
}

#[test]
fn encodes_subdomain_with_three_labels() {
    let encoded = encode_domain_name("api.example.com");
    let mut expected = vec![3];
    expected.extend_from_slice(b"api");
    expected.push(7);
    expected.extend_from_slice(b"example");
    expected.push(3);
    expected.extend_from_slice(b"com");
    expected.push(0);
    assert_eq!(encoded, expected);
}

#[test]
fn encodes_empty_domain_as_root() {
    let encoded = encode_domain_name("");
    assert_eq!(encoded, vec![0]);
}

#[test]
fn ends_with_null_terminator() {
    let encoded = encode_domain_name("example.com");
    assert_eq!(*encoded.last().unwrap(), 0);
}

#[test]
fn label_length_matches_byte_count() {
    let encoded = encode_domain_name("example.com");
    // first byte is the length of "example"
    assert_eq!(encoded[0] as usize, "example".len());
    // next `example.len()` bytes are the label itself
    assert_eq!(&encoded[1..1 + "example".len()], b"example");
}

#[test]
fn max_length_label_of_63_bytes() {
    let label = "a".repeat(63);
    let domain = format!("{label}.com");
    let encoded = encode_domain_name(&domain);
    assert_eq!(encoded[0], 63);
    assert_eq!(&encoded[1..64], label.as_bytes());
}

// A DNS header is a fixed 12 bytes: ID(2) FLAGS(2) QDCOUNT(2) ANCOUNT(2)
// NSCOUNT(2) ARCOUNT(2), followed by QNAME, QTYPE(2), QCLASS(2).
const HEADER_LEN: usize = 12;

#[test]
fn packet_starts_with_the_returned_transaction_id() {
    let encoded = encode_domain_name("example.com");
    let (packet, id) = build_query_packet(encoded, DnsQuery::A);
    assert_eq!(&packet[0..2], &id.to_be_bytes());
}

#[test]
fn packet_length_matches_header_plus_qname_plus_qtype_and_qclass() {
    let encoded = encode_domain_name("example.com");
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    assert_eq!(packet.len(), HEADER_LEN + qname_len + 4);
}

#[test]
fn flags_request_recursion_desired() {
    let encoded = encode_domain_name("example.com");
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    assert_eq!(&packet[2..4], &[0x01, 0x00]);
}

#[test]
fn qdcount_is_one() {
    let encoded = encode_domain_name("example.com");
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    assert_eq!(&packet[4..6], &[0x00, 0x01]);
}

#[test]
fn ancount_nscount_arcount_are_zero() {
    let encoded = encode_domain_name("example.com");
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    assert_eq!(&packet[6..HEADER_LEN], &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn qname_section_matches_encoded_domain() {
    let encoded = encode_domain_name("example.com");
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(encoded.clone(), DnsQuery::A);
    assert_eq!(&packet[HEADER_LEN..HEADER_LEN + qname_len], encoded.as_slice());
}

#[test]
fn qtype_reflects_a_record() {
    let encoded = encode_domain_name("example.com");
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    let qtype = HEADER_LEN + qname_len;
    assert_eq!(&packet[qtype..qtype + 2], &[0x00, 0x01]);
}

#[test]
fn qtype_reflects_ns_record() {
    let encoded = encode_domain_name("example.com");
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(encoded, DnsQuery::NS);
    let qtype = HEADER_LEN + qname_len;
    assert_eq!(&packet[qtype..qtype + 2], &[0x00, 0x02]);
}

#[test]
fn qclass_is_internet() {
    let encoded = encode_domain_name("example.com");
    let (packet, _id) = build_query_packet(encoded, DnsQuery::A);
    let qclass = packet.len() - 2;
    assert_eq!(&packet[qclass..], &[0x00, 0x01]);
}

#[test]
fn transaction_ids_vary_between_calls() {
    let ids: std::collections::HashSet<u16> = (0..20)
        .map(|_| build_query_packet(encode_domain_name("example.com"), DnsQuery::A).1)
        .collect();
    assert!(ids.len() > 1, "expected varying transaction ids across calls");
}
