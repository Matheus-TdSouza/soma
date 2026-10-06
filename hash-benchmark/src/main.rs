use sha2::{Sha256, Digest};
use std::time::Instant;
use std::fs::File;
use std::io;
use std::io::Read;
const SIZE: usize = 1073741824;
const SIZE_F64: f64 = SIZE as f64;
const BENCHMARK_PATH: &str = "../benchmark-dd.bin";

fn main() {
    println!("{:?}", read_benchmark());
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
