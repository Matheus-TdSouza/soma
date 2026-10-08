use clap::{Parser, Subcommand};
use hash_benchmark::file::BENCHMARK_PATH;
use hash_benchmark::hash::{HashAlgo, parse_algo};
use hash_benchmark::size::parse_size;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "soma disk and hash benchmarks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Write a file of random bytes")]
    Generate {
        #[arg(value_parser = parse_size_arg, help = "File size, e.g. 1G")]
        size: usize,
        #[arg(long, default_value = BENCHMARK_PATH)]
        path: PathBuf,
    },
    #[command(about = "Hash an in-memory buffer")]
    Hash {
        #[arg(value_parser = parse_size_arg)]
        size: usize,
        #[arg(value_parser = parse_algo_arg, help = "sha256, blake3 or blake3-mt; all three if omitted")]
        algo: Option<HashAlgo>,
    },
    #[command(about = "Read the whole file sequentially")]
    Seq {
        #[arg(value_parser = parse_size_arg)]
        buf_size: usize,
        #[arg(long)]
        direct: bool,
        #[arg(long, default_value = BENCHMARK_PATH)]
        path: PathBuf,
    },
    #[command(about = "Random 4 KiB reads with latency percentiles")]
    Rand {
        n: usize,
        #[arg(long)]
        direct: bool,
        #[arg(long, default_value = BENCHMARK_PATH)]
        path: PathBuf,
    },
}

fn parse_size_arg(s: &str) -> Result<usize, String> {
    parse_size(s).ok_or_else(|| format!("invalid size '{s}', expected e.g. 4K, 1M, 2G"))
}

fn parse_algo_arg(s: &str) -> Result<HashAlgo, String> {
    parse_algo(s).ok_or_else(|| format!("invalid algorithm '{s}', expected sha256, blake3 or blake3-mt"))
}