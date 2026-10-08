# soma

A decentralized storage network where nodes prove they still hold the data they were paid to store.
Each file is committed to a Merkle root; nodes answer random challenges with chunks and Merkle paths, and a smart contract verifies the answers.

Current stage: disk read and hashing benchmarks, used to size the cost of answering challenges.

## Layout

- `hash-benchmark/`: disk read and hashing benchmarks.
  - `src/main.rs`, `src/cli.rs`: command line (clap).
  - `src/lib.rs`: library modules (`file`, `seq`, `random_read`, `hash`, `stats`, `size`).
  - `tests/`: tests against the library's public API.

## Running

Always build in release mode; debug builds distort the numbers. From the repository root:

```
cargo run --release -p hash-benchmark -- <command> [options]
```

From inside `hash-benchmark/`, `-p hash-benchmark` can be dropped. Use `--help` on the program or on any command to see every option.

| Command | What it measures |
|---|---|
| `generate <size> [--path P]` | Writes a file of random bytes to benchmark against, e.g. `generate 1G` |
| `seq <buffer_size> [--direct] [--path P]` | Sequential read of the whole file: bytes, time and MiB/s |
| `rand <n> [--direct] [--path P]` | `n` random 4 KiB reads: p50, p90, p99, max, mean, standard deviation and IOPS |
| `hash <size> [sha256\|blake3\|blake3-mt]` | Hash throughput over an in-memory buffer; runs all three algorithms when none is given |

Sizes take a `K`, `M` or `G` suffix (`4K`, `1M`, `2G`). `--path` defaults to `benchmark.bin` in the current directory.

Example, measuring a second drive:

```
cargo run --release -p hash-benchmark -- generate 1G --path E:/bench.bin
cargo run --release -p hash-benchmark -- seq 1M --direct --path E:/bench.bin
cargo run --release -p hash-benchmark -- rand 10000 --direct --path E:/bench.bin
```

## Notes on measuring

- **Use `--direct` for disk numbers.** Without it, reads may be served from the operating system's page cache and measure RAM instead of the disk. `--direct` bypasses the cache and is currently implemented on Windows only. Buffer sizes must be multiples of 4 KiB.
- **Git Bash paths.** Bash treats `\` as an escape character, so `E:\bench.bin` reaches the program as `E:bench.bin`. Use forward slashes (`E:/bench.bin`) or quote the path.
- **Hard drives need large files.** A small file sits in a narrow band of the platter, so random reads barely move the head and latency comes out optimistic. Use a file that covers a large share of the disk for realistic random read numbers.
- **Clean up.** Benchmark files can be large. They are ignored by git (`*.bin`) but stay on disk until deleted.

## Testing

```
cargo test
```
