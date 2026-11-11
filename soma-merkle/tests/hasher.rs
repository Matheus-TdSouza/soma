use soma_merkle::{MerkleHasher, Sha256Hasher, Blake3Hasher};

fn assert_domains_disjoint<H: MerkleHasher>() {
    let a = [1u8; 32];
    let b = [2u8; 32];
    let mut fake_leaf = vec![0x01];
    fake_leaf.extend_from_slice(&a);
    fake_leaf.extend_from_slice(&b);
    assert_ne!(H::leaf(&fake_leaf), H::node(&a, &b));
}

#[test]
fn domains_disjoint_sha256() { assert_domains_disjoint::<Sha256Hasher>(); }

#[test]
fn domains_disjoint_blake3() { assert_domains_disjoint::<Blake3Hasher>(); }

#[test]
fn sha256_empty_leaf_matches_rfc6962() {
    let expected = "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d";
    let got: String = Sha256Hasher::leaf(&[]).iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(got, expected);
}