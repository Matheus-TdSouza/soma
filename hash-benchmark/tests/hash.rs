use hash_benchmark::hash::{HashAlgo, compute_hash, parse_algo};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn parse_algo_accepts_known_names() {
    assert!(matches!(parse_algo("sha256"), Some(HashAlgo::Sha256)));
    assert!(matches!(parse_algo("blake3"), Some(HashAlgo::Blake3)));
    assert!(matches!(parse_algo("blake3-mt"), Some(HashAlgo::Blake3Mt)));
}

#[test]
fn parse_algo_rejects_unknown_names() {
    assert!(parse_algo("blake").is_none());
    assert!(parse_algo("SHA256").is_none());
    assert!(parse_algo("").is_none());
}

#[test]
fn compute_hash_sha256_matches_known_vector() {
    assert_eq!(
        hex(&compute_hash(b"abc", HashAlgo::Sha256)),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn compute_hash_blake3_matches_known_vector() {
    assert_eq!(
        hex(&compute_hash(b"abc", HashAlgo::Blake3)),
        "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
    );
}

#[test]
fn compute_hash_blake3_single_and_multi_thread_agree() {
    let data = vec![7u8; 1 << 20];
    assert_eq!(
        compute_hash(&data, HashAlgo::Blake3),
        compute_hash(&data, HashAlgo::Blake3Mt)
    );
}
