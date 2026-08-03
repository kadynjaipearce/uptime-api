#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::net::Ipv4Addr;

use uptime_api::core::handler::dns::{
    DnsRecordType, build_query_packet, encode_domain_name, parse_dns_response,
};

#[test]
fn encodes_two_labels() {
    let encoded = encode_domain_name("example.com").unwrap();
    let mut expected = vec![7];
    expected.extend_from_slice(b"example");
    expected.push(3);
    expected.extend_from_slice(b"com");
    expected.push(0);
    assert_eq!(encoded, expected);
}

#[test]
fn encodes_single_label() {
    let encoded = encode_domain_name("localhost").unwrap();
    let mut expected = vec![9];
    expected.extend_from_slice(b"localhost");
    expected.push(0);
    assert_eq!(encoded, expected);
}

#[test]
fn encodes_subdomain_with_three_labels() {
    let encoded = encode_domain_name("api.example.com").unwrap();
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
    let encoded = encode_domain_name("").unwrap();
    assert_eq!(encoded, vec![0]);
}

#[test]
fn ends_with_null_terminator() {
    let encoded = encode_domain_name("example.com").unwrap();
    assert_eq!(*encoded.last().unwrap(), 0);
}

#[test]
fn label_length_matches_byte_count() {
    let encoded = encode_domain_name("example.com").unwrap();
    // first byte is the length of "example"
    assert_eq!(encoded[0] as usize, "example".len());
    // next `example.len()` bytes are the label itself
    assert_eq!(&encoded[1..1 + "example".len()], b"example");
}

#[test]
fn max_length_label_of_63_bytes() {
    let label = "a".repeat(63);
    let domain = format!("{label}.com");
    let encoded = encode_domain_name(&domain).unwrap();
    assert_eq!(encoded[0], 63);
    assert_eq!(&encoded[1..64], label.as_bytes());
}

// A DNS header is a fixed 12 bytes: ID(2) FLAGS(2) QDCOUNT(2) ANCOUNT(2)
// NSCOUNT(2) ARCOUNT(2), followed by QNAME, QTYPE(2), QCLASS(2).
const HEADER_LEN: usize = 12;

#[test]
fn packet_starts_with_the_returned_transaction_id() {
    let encoded = encode_domain_name("example.com").unwrap();
    let (packet, id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(&packet[0..2], &id.to_be_bytes());
}

#[test]
fn packet_length_matches_header_plus_qname_plus_qtype_and_qclass() {
    let encoded = encode_domain_name("example.com").unwrap();
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(packet.len(), HEADER_LEN + qname_len + 4);
}

#[test]
fn flags_request_recursion_desired() {
    let encoded = encode_domain_name("example.com").unwrap();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(&packet[2..4], &[0x01, 0x00]);
}

#[test]
fn qdcount_is_one() {
    let encoded = encode_domain_name("example.com").unwrap();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(&packet[4..6], &[0x00, 0x01]);
}

#[test]
fn ancount_nscount_arcount_are_zero() {
    let encoded = encode_domain_name("example.com").unwrap();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(
        &packet[6..HEADER_LEN],
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn qname_section_matches_encoded_domain() {
    let encoded = encode_domain_name("example.com").unwrap();
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    assert_eq!(
        &packet[HEADER_LEN..HEADER_LEN + qname_len],
        encoded.as_slice()
    );
}

#[test]
fn qtype_reflects_a_record() {
    let encoded = encode_domain_name("example.com").unwrap();
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    let qtype = HEADER_LEN + qname_len;
    assert_eq!(&packet[qtype..qtype + 2], &[0x00, 0x01]);
}

#[test]
fn qtype_reflects_ns_record() {
    let encoded = encode_domain_name("example.com").unwrap();
    let qname_len = encoded.len();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::NS);
    let qtype = HEADER_LEN + qname_len;
    assert_eq!(&packet[qtype..qtype + 2], &[0x00, 0x02]);
}

#[test]
fn qclass_is_internet() {
    let encoded = encode_domain_name("example.com").unwrap();
    let (packet, _id) = build_query_packet(&encoded, DnsRecordType::A);
    let qclass = packet.len() - 2;
    assert_eq!(&packet[qclass..], &[0x00, 0x01]);
}

#[test]
fn transaction_ids_vary_between_calls() {
    let ids: std::collections::HashSet<u16> = (0..20)
        .map(|_| {
            build_query_packet(
                &encode_domain_name("example.com").unwrap(),
                DnsRecordType::A,
            )
            .1
        })
        .collect();
    assert!(
        ids.len() > 1,
        "expected varying transaction ids across calls"
    );
}

// ---------------------------------------------------------------------
// parse_dns_response
//
// These build raw DNS response packets by hand so `parse_dns_response`
// can be exercised without a real network round trip. Layout per RFC 1035:
//
//   Header  (12 bytes): ID FLAGS QDCOUNT ANCOUNT NSCOUNT ARCOUNT
//   Question:           QNAME QTYPE QCLASS
//   Answer (per record): NAME TYPE CLASS TTL RDLENGTH RDATA
//
// The question's QNAME always starts at byte offset 12, so answer NAME
// fields can point back at it with the compression pointer 0xC0 0x0C.
// ---------------------------------------------------------------------

const RCODE_NO_ERROR: u8 = 0;
const RCODE_SERVER_FAILURE: u8 = 2;
const RCODE_NAME_ERROR: u8 = 3; // NXDOMAIN

const TYPE_A: u16 = 1;
const TYPE_CNAME: u16 = 5;
const CLASS_IN: u16 = 1;

fn response_header(id: u16, flags_hi: u8, rcode: u8, qdcount: u16, ancount: u16) -> Vec<u8> {
    let mut header = Vec::with_capacity(12);
    header.extend_from_slice(&id.to_be_bytes());
    header.push(flags_hi); // QR/Opcode/AA/TC/RD
    header.push(0x80 | (rcode & 0x0F)); // RA=1, Z=0, RCODE
    header.extend_from_slice(&qdcount.to_be_bytes());
    header.extend_from_slice(&ancount.to_be_bytes());
    header.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    header.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT
    header
}

/// Standard "QR=1, RD=1" flags byte for a well-formed response to a
/// recursive query.
const FLAGS_HI_RESPONSE: u8 = 0x81;

fn question_section(domain: &str) -> Vec<u8> {
    let mut question = encode_domain_name(domain).unwrap();
    question.extend_from_slice(&TYPE_A.to_be_bytes());
    question.extend_from_slice(&CLASS_IN.to_be_bytes());
    question
}

/// An answer record whose NAME is a compression pointer back to the
/// question's QNAME at offset 12.
fn pointer_answer(record_type: u16, ttl: u32, rdata: &[u8]) -> Vec<u8> {
    let mut answer = vec![0xC0, 0x0C];
    answer.extend_from_slice(&record_type.to_be_bytes());
    answer.extend_from_slice(&CLASS_IN.to_be_bytes());
    answer.extend_from_slice(&ttl.to_be_bytes());
    answer.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
    answer.extend_from_slice(rdata);
    answer
}

fn a_record_rdata(ip: Ipv4Addr) -> Vec<u8> {
    ip.octets().to_vec()
}

fn valid_a_response(id: u16, domain: &str, ip: Ipv4Addr) -> Vec<u8> {
    let mut packet = response_header(id, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 1);
    packet.extend_from_slice(&question_section(domain));
    packet.extend_from_slice(&pointer_answer(TYPE_A, 300, &a_record_rdata(ip)));
    packet
}

#[test]
fn parses_a_record_and_returns_ip() {
    let expected_ip = Ipv4Addr::new(93, 184, 215, 14);
    let response = valid_a_response(0xABCD, "example.com", expected_ip);

    let ip = parse_dns_response(response, 0xABCD).unwrap();
    assert_eq!(ip, expected_ip);
}

#[test]
fn parses_uncompressed_name_in_answer() {
    let expected_ip = Ipv4Addr::new(10, 0, 0, 1);
    let domain = "example.com";

    let mut packet = response_header(0x1234, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 1);
    packet.extend_from_slice(&question_section(domain));

    // Answer NAME repeated as a full label sequence instead of a pointer.
    let mut answer = encode_domain_name(domain).unwrap();
    answer.extend_from_slice(&TYPE_A.to_be_bytes());
    answer.extend_from_slice(&CLASS_IN.to_be_bytes());
    answer.extend_from_slice(&300u32.to_be_bytes());
    let rdata = a_record_rdata(expected_ip);
    answer.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
    answer.extend_from_slice(&rdata);
    packet.extend_from_slice(&answer);

    let ip = parse_dns_response(packet, 0x1234).unwrap();
    assert_eq!(ip, expected_ip);
}

#[test]
fn skips_cname_answer_to_find_a_record() {
    let expected_ip = Ipv4Addr::new(203, 0, 113, 7);
    let mut packet = response_header(0x0042, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 2);
    packet.extend_from_slice(&question_section("example.com"));
    // First answer: a CNAME record that should be skipped.
    packet.extend_from_slice(&pointer_answer(
        TYPE_CNAME,
        300,
        b"\x03www\x07example\x03com\x00",
    ));
    // Second answer: the A record we actually want.
    packet.extend_from_slice(&pointer_answer(TYPE_A, 300, &a_record_rdata(expected_ip)));

    let ip = parse_dns_response(packet, 0x0042).unwrap();
    assert_eq!(ip, expected_ip);
}

#[test]
fn errors_when_no_a_record_present() {
    let mut packet = response_header(0x0042, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 1);
    packet.extend_from_slice(&question_section("example.com"));
    packet.extend_from_slice(&pointer_answer(
        TYPE_CNAME,
        300,
        b"\x03www\x07example\x03com\x00",
    ));

    assert!(parse_dns_response(packet, 0x0042).is_err());
}

#[test]
fn errors_on_transaction_id_mismatch() {
    let response = valid_a_response(0x1111, "example.com", Ipv4Addr::new(1, 2, 3, 4));
    assert!(parse_dns_response(response, 0x2222).is_err());
}

#[test]
fn errors_on_nxdomain_rcode() {
    let mut packet = response_header(0x5555, FLAGS_HI_RESPONSE, RCODE_NAME_ERROR, 1, 0);
    packet.extend_from_slice(&question_section("nonexistent.invalid"));

    assert!(parse_dns_response(packet, 0x5555).is_err());
}

#[test]
fn errors_on_server_failure_rcode() {
    let mut packet = response_header(0x5555, FLAGS_HI_RESPONSE, RCODE_SERVER_FAILURE, 1, 0);
    packet.extend_from_slice(&question_section("example.com"));

    assert!(parse_dns_response(packet, 0x5555).is_err());
}

#[test]
fn errors_when_answer_count_is_zero() {
    let mut packet = response_header(0x7777, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 0);
    packet.extend_from_slice(&question_section("example.com"));

    assert!(parse_dns_response(packet, 0x7777).is_err());
}

#[test]
fn errors_when_qr_bit_indicates_a_query_not_a_response() {
    // FLAGS_HI with QR=0 (still a query), which should never be treated
    // as a valid answer to parse.
    let mut packet = response_header(0x8888, 0x01, RCODE_NO_ERROR, 1, 1);
    packet.extend_from_slice(&question_section("example.com"));
    packet.extend_from_slice(&pointer_answer(
        TYPE_A,
        300,
        &a_record_rdata(Ipv4Addr::new(1, 1, 1, 1)),
    ));

    assert!(parse_dns_response(packet, 0x8888).is_err());
}

#[test]
fn errors_on_truncated_header() {
    // Fewer than the 12 header bytes required.
    let packet = vec![0x12, 0x34, 0x81, 0x80];
    assert!(parse_dns_response(packet, 0x1234).is_err());
}

#[test]
fn errors_on_response_truncated_mid_answer() {
    let full = valid_a_response(0x9999, "example.com", Ipv4Addr::new(8, 8, 8, 8));
    // Cut the packet off partway through the answer's RDATA.
    let truncated = full[..full.len() - 2].to_vec();

    assert!(parse_dns_response(truncated, 0x9999).is_err());
}

#[test]
fn errors_on_malformed_a_record_rdlength() {
    // RDLENGTH says 6 bytes but TYPE_A should always carry exactly 4.
    let mut packet = response_header(0xAAAA, FLAGS_HI_RESPONSE, RCODE_NO_ERROR, 1, 1);
    packet.extend_from_slice(&question_section("example.com"));
    packet.extend_from_slice(&pointer_answer(TYPE_A, 300, b"\x01\x02\x03\x04\x05\x06"));

    assert!(parse_dns_response(packet, 0xAAAA).is_err());
}
