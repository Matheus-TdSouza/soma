use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum HashAlgo {
    Sha256,
    Blake3,
    Blake3Mt,
}

pub struct HashResult {
    pub algo: HashAlgo,
    pub hash: [u8; 32],
    pub bytes: usize,
    pub runtime: Duration,
}

impl HashResult {
    pub fn throughput_mib_s(&self) -> f64 {
        self.bytes as f64 / 1024.0 / 1024.0 / self.runtime.as_secs_f64()
    }

    pub fn hex(&self) -> String {
        self.hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

pub fn parse_algo(s: &str) -> Option<HashAlgo> {
    match s {
        "sha256" => Some(HashAlgo::Sha256),
        "blake3" => Some(HashAlgo::Blake3),
        "blake3-mt" => Some(HashAlgo::Blake3Mt),
        _ => None,
    }
}

pub fn hash_benchmark(size: usize, algo: Option<HashAlgo>) -> Vec<HashResult> {
    let data = vec![1u8; size];
    match algo {
        Some(algo) => vec![time_hash(&data, algo)],
        None => [HashAlgo::Sha256, HashAlgo::Blake3, HashAlgo::Blake3Mt]
            .into_iter()
            .map(|algo| time_hash(&data, algo))
            .collect(),
    }
}

pub fn compute_hash(data: &[u8], algo: HashAlgo) -> [u8; 32] {
    match algo {
        HashAlgo::Sha256 => Sha256::digest(data).into(),
        HashAlgo::Blake3 => *blake3::hash(data).as_bytes(),
        HashAlgo::Blake3Mt => *blake3::Hasher::new()
            .update_rayon(data)
            .finalize()
            .as_bytes(),
    }
}

fn time_hash(data: &[u8], algo: HashAlgo) -> HashResult {
    let start = Instant::now();
    let hash = compute_hash(data, algo);
    let runtime = start.elapsed();
    std::hint::black_box(&hash);
    HashResult {
        algo,
        hash,
        bytes: data.len(),
        runtime,
    }
}
