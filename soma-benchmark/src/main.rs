mod cli;
mod suite;

use clap::Parser;
use cli::{Cli, Command};
use soma_benchmark::file::generate;
use soma_benchmark::hash::hash_benchmark;
use soma_benchmark::random_read::rand_benchmark;
use soma_benchmark::seq::seq_benchmark;
use soma_benchmark::stats::latency_stats;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Suite {
            label,
            path,
            size,
            reps,
            out,
            keep,
        } => suite::run(&label, &path, size, reps, &out, keep)?,
        Command::Generate { size, path } => {
            generate(&path, size)?;
            println!("File generated with {} bytes", size);
        }
        Command::Hash { size, algo } => {
            for r in hash_benchmark(size, algo) {
                println!(
                    "{:?}: {} | {:?} | {:.2} MiB/s",
                    r.algo,
                    r.hex(),
                    r.runtime,
                    r.throughput_mib_s()
                );
            }
        }
        Command::Seq {
            buf_size,
            direct,
            path,
        } => {
            let result = seq_benchmark(&path, buf_size, direct, usize::MAX)?;
            println!("Read: {} bytes", result.bytes);
            println!("Runtime: {:?}", result.runtime);
            println!("Throughput: {:.2} MiB/s", result.throughput_mib_s());
        }
        Command::Rand { n, direct, path } => {
            let mut durations = rand_benchmark(&path, n, direct)?;
            let s = latency_stats(&mut durations);
            println!("p50: {:?}", s.p50);
            println!("p90: {:?}", s.p90);
            println!("p99: {:?}", s.p99);
            println!("Max: {:?}", s.max);
            println!("Average: {:?}", s.mean);
            println!("Std dev: {:?}", s.std_dev);
            println!("IOPS: {:.2}", s.iops);
        }
    }
    Ok(())
}
