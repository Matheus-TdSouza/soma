mod cli;

use clap::Parser;
use cli::{Cli, Command};
use hash_benchmark::file::generate;
use hash_benchmark::hash::hash_benchmark;
use hash_benchmark::random_read::rand_benchmark;
use hash_benchmark::seq::seq_benchmark;
use hash_benchmark::stats::print_latency_stats;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { size, path } => {
            generate(&path, size)?;
            println!("File generated with {} bytes", size);
        }
        Command::Hash { size, algo } => hash_benchmark(size, algo),
        Command::Seq { buf_size, direct, path } => {
            seq_benchmark(&path, buf_size, direct)?;
        }
        Command::Rand { n, direct, path } => {
            let mut durations = rand_benchmark(&path, n, direct)?;
            print_latency_stats(&mut durations);
        }
    }
    Ok(())
}