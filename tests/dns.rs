use uptime_api::core::handler::dns::encode_domain_name;

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
