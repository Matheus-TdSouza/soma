use hash_benchmark::file::{BENCHMARK_PATH, generate};
use hash_benchmark::hash::{hash_benchmark, parse_algo};
use hash_benchmark::random_read::rand_benchmark;
use hash_benchmark::seq::seq_benchmark;
use hash_benchmark::size::parse_size;
use hash_benchmark::stats::print_latency_stats;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let size = args.get(2).and_then(|s| parse_size(s));
    match args.get(1).map(String::as_str) {
        Some("generate") => {
            let size =
                size.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid size"))?;
            generate(Path::new(BENCHMARK_PATH), size)?;
            println!("File generated with {} bytes", size);
            Ok(())
        }
        Some("hash") => {
            let size =
                size.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid size"))?;
            let algo = match args.get(3) {
                Some(s) => Some(parse_algo(s).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Invalid hash algorithm")
                })?),
                None => None,
            };
            hash_benchmark(size, algo);
            Ok(())
        }
        Some("seq") => {
            let direct = args.get(3).map(String::as_str) == Some("direct");
            let buffer_size = size.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid buffer size")
            })?;
            seq_benchmark(Path::new(BENCHMARK_PATH), buffer_size, direct)?;
            Ok(())
        }
        Some("rand") => {
            let n = args
                .get(2)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Missing number of reads")
                })?
                .parse::<usize>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Invalid number of reads")
                })?;
            let direct = args.get(3).map(String::as_str) == Some("direct");
            let mut durations = rand_benchmark(Path::new(BENCHMARK_PATH), n, direct)?;
            print_latency_stats(&mut durations);
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: hash-benchmark generate <size> | hash <size> [sha256|blake3|blake3-mt] | seq <buffer_size> [direct] | rand <n> [direct]"
            );
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input"))
        }
    }
}
