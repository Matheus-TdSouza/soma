use rand::prelude::*;
use soma_merkle::{Blake3Hasher, MerkleHasher, MerkleTree, Sha256Hasher, verify};
use std::time::{Duration, Instant};

/// Candidate leaf sizes. One becomes a network constant once chosen; until
/// then `leaf_size` is a parameter everywhere.
pub const LEAF_SIZES: [usize; 4] = [32, 256, 1 << 10, 4 << 10];
pub const DEFAULT_LEAF_SIZE: usize = 1 << 10;

const VERIFY_SAMPLES: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerkleAlgo {
    Sha256,
    Blake3,
}

impl MerkleAlgo {
    pub const ALL: [MerkleAlgo; 2] = [MerkleAlgo::Sha256, MerkleAlgo::Blake3];

    pub fn name(&self) -> &'static str {
        match self {
            MerkleAlgo::Sha256 => "sha256",
            MerkleAlgo::Blake3 => "blake3",
        }
    }
}

pub fn parse_merkle_algo(s: &str) -> Option<MerkleAlgo> {
    match s {
        "sha256" => Some(MerkleAlgo::Sha256),
        "blake3" => Some(MerkleAlgo::Blake3),
        _ => None,
    }
}

pub struct MerkleResult {
    pub algo: MerkleAlgo,
    pub leaf_size: usize,
    pub bytes: usize,
    pub leaf_count: usize,
    /// Time to hash every leaf (in parallel) and build all levels.
    pub build: Duration,
    /// Siblings in one proof.
    pub depth: usize,
    /// Mean time to verify one proof.
    pub verify: Duration,
}

impl MerkleResult {
    pub fn build_throughput_mib_s(&self) -> f64 {
        self.bytes as f64 / 1024.0 / 1024.0 / self.build.as_secs_f64()
    }

    /// Bytes a challenge answer carries: the leaf plus 32 bytes per sibling.
    /// Leaves out the index, which the verifier already knows.
    pub fn proof_bytes(&self) -> usize {
        self.leaf_size + 32 * self.depth
    }
}

/// Builds a tree over `size` in-memory bytes, then times `VERIFY_SAMPLES`
/// proofs at random indices. Returns `None` for an empty buffer or a zero
/// leaf size.
pub fn merkle_benchmark(size: usize, leaf_size: usize, algo: MerkleAlgo) -> Option<MerkleResult> {
    let data = vec![1u8; size];
    match algo {
        MerkleAlgo::Sha256 => run::<Sha256Hasher>(&data, leaf_size, algo),
        MerkleAlgo::Blake3 => run::<Blake3Hasher>(&data, leaf_size, algo),
    }
}

fn run<H: MerkleHasher>(data: &[u8], leaf_size: usize, algo: MerkleAlgo) -> Option<MerkleResult> {
    let start = Instant::now();
    let tree = MerkleTree::<H>::build(data, leaf_size)?;
    let build = start.elapsed();

    let commitment = tree.commitment();
    let leaf_count = tree.leaf_count();
    let mut rng = rand::rng();
    let challenges: Vec<_> = (0..VERIFY_SAMPLES)
        .map(|_| {
            let i = rng.random_range(0..leaf_count);
            let leaf = data.chunks(leaf_size).nth(i).expect("i < leaf_count");
            (leaf, tree.prove(i).expect("i < leaf_count"))
        })
        .collect();
    let depth = challenges[0].1.siblings.len();

    let start = Instant::now();
    for (leaf, proof) in &challenges {
        assert!(verify::<H>(&commitment, leaf, proof), "honest proof rejected");
    }
    let verify = start.elapsed().div_f64(VERIFY_SAMPLES as f64);

    Some(MerkleResult {
        algo,
        leaf_size,
        bytes: data.len(),
        leaf_count,
        build,
        depth,
        verify,
    })
}
