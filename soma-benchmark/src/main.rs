mod cli;
mod suite;

use clap::Parser;
use cli::{Cli, Command};
use soma_benchmark::challenge::challenge_benchmark;
use soma_benchmark::file::generate;
use soma_benchmark::hash::hash_benchmark;
use soma_benchmark::merkle::{LEAF_SIZES, MerkleAlgo, merkle_benchmark};
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
        Command::Merkle {
            size,
            leaf_size,
            algo,
        } => {
            let algos = algo.map_or(MerkleAlgo::ALL.to_vec(), |a| vec![a]);
            let leaf_sizes = leaf_size.map_or(LEAF_SIZES.to_vec(), |l| vec![l]);
            for algo in algos {
                for &leaf_size in &leaf_sizes {
                    let Some(r) = merkle_benchmark(size, leaf_size, algo) else {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "size must be > 0"));
                    };
                    println!(
                        "{} leaf {} B: {} leaves, build {:?} ({:.2} MiB/s), depth {}, proof {} B, verify {:?}",
                        algo.name(),
                        leaf_size,
                        r.leaf_count,
                        r.build,
                        r.build_throughput_mib_s(),
                        r.depth,
                        r.proof_bytes(),
                        r.verify
                    );
                }
            }
        }
        Command::Challenge {
            path,
            leaf_size,
            k,
            rounds,
            direct,
            algo,
        } => {
            let run = challenge_benchmark(&path, leaf_size, k as usize, rounds as usize, direct, algo)?;
            println!(
                "Committed {} leaves of {} B in {:?}; response {} B per round",
                run.leaf_count, leaf_size, run.build, run.response_bytes
            );
            for (i, t) in run.rounds.iter().enumerate() {
                println!("Round {}: {:?} ({:?} per challenge)", i + 1, t, *t / k);
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
