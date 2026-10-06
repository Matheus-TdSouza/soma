# soma

A decentralized storage network where nodes prove they still hold the data they were paid to store.
Each file is committed to a Merkle root; nodes answer random challenges with chunks and Merkle paths, and a smart contract verifies the answers.

Current stage: disk read and hashing benchmarks, used to size the cost of answering challenges.

## Layout

- `hash-benchmark/`: SHA-256 throughput and disk read benchmarks.

## Running

The read benchmark opens `benchmark-dd.bin` relative to the working directory (`../benchmark-dd.bin`), so run it from inside the crate, with the file at the repository root:

```
cd hash-benchmark
cargo run --release
```
