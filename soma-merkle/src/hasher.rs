use sha2::{Digest, Sha256};

pub trait MerkleHasher {
    fn leaf(data: &[u8]) -> [u8; 32];
    fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32];
}

pub struct Sha256Hasher;
pub struct Blake3Hasher;

impl MerkleHasher for Sha256Hasher {
    fn leaf(data: &[u8]) -> [u8; 32] {
        Sha256::new_with_prefix([0x00])
            .chain_update(data)
            .finalize()
            .into()
    }

    fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        Sha256::new_with_prefix([0x01])
            .chain_update(left)
            .chain_update(right)
            .finalize()
            .into()
    }
}

impl MerkleHasher for Blake3Hasher {
    fn leaf(data: &[u8]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[0x00]);
        hasher.update(data);
        *hasher.finalize().as_bytes()
    }

    fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[0x01]);
        hasher.update(left);
        hasher.update(right);
        *hasher.finalize().as_bytes()
    }
}
