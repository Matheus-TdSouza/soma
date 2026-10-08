use sha2::{Digest, Sha256};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum HashAlgo {
    Sha256,
    Blake3,
    Blake3Mt,
}

pub fn parse_algo(s: &str) -> Option<HashAlgo> {
    match s {
        "sha256" => Some(HashAlgo::Sha256),
        "blake3" => Some(HashAlgo::Blake3),
        "blake3-mt" => Some(HashAlgo::Blake3Mt),
        _ => None,
    }
}

pub fn hash_benchmark(size: usize, algo: Option<HashAlgo>) {
    let data = vec![1u8; size];
    match algo {
        Some(algo) => time_hash(&data, algo),
        None => {
            for algo in [HashAlgo::Sha256, HashAlgo::Blake3, HashAlgo::Blake3Mt] {
                time_hash(&data, algo);
            }
        }
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

fn time_hash(data: &[u8], algo: HashAlgo) {
    let start = Instant::now();
    let hash = compute_hash(data, algo);
    let runtime = start.elapsed();
    std::hint::black_box(&hash);
    let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    let throughput = (data.len() as f64 / 1024.0 / 1024.0) / runtime.as_secs_f64();
    println!(
        "{:?}: {} | {:?} | {:.2} MiB/s",
        algo, hex, runtime, throughput
    );
}
