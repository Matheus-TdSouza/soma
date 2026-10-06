use sha2::{Sha256, Digest};
use std::time::Instant;
use std::fs::File;
use std::io::{self, Read, Write};
use rand::prelude::*;
const SIZE: usize = 1073741824;
const SIZE_F64: f64 = SIZE as f64;
const BENCHMARK_PATH: &str = "../benchmark.bin";

fn main() {
    println!("{:?}", read_benchmark());
}

fn parse_size(s: &str) -> Option<usize> {
    let i = s.len().checked_sub(1)?;
    let (number, unit) = s.split_at_checked(i)?;
    let number = number.parse::<usize>().ok()?;
    let mult = match unit {
        "K" | "k" => 1 << 10,
        "M" | "m" => 1 << 20,
        "G" | "g" => 1 << 30,
        _ => return None,
    };
    number.checked_mul(mult)
}

fn generate(path: &str, total: usize) -> io::Result<()> {
    let mut file = File::create(path)?;
    let mut rng = rand::rng();
    let mut buf = vec![0u8; 8 << 20];
    let mut remaining = total;
    while remaining > 0 {
        let n = remaining.min(buf.len());
        rng.fill_bytes(&mut buf[..n]);
        file.write_all(&buf[..n])?;
        remaining -= n;
    }
    file.sync_all()?;
    Ok(())
}

fn hash_benchmark() {
    let mut data: Vec<u8> = Vec::with_capacity(SIZE);
    data.resize(SIZE, 1);
    let start = Instant::now();
    let hash = Sha256::digest(data);
    let runtime = start.elapsed();
    print!("Hash: ");
    for data in hash {
        print!("{:02x}", data);
    }
    println!();
    println!("Runtime: {:?}", runtime);
    let runtime_secs: f64 = runtime.as_secs_f64();
    let throughput = (SIZE_F64 / 1024.0 / 1024.0) / runtime_secs;
    println!("Throughput: {:?} MiB/s", throughput);
}

fn read_benchmark() -> io::Result<usize> {
    let mut file = File::open(BENCHMARK_PATH)?;
    let mut buffer = [0u8; 4096];
    let bytes = file.read(&mut buffer)?;
    Ok(bytes)
}
