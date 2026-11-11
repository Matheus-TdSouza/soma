#![cfg(target_os = "linux")]

use soma_benchmark::file::generate;
use soma_benchmark::random_read::rand_benchmark;
use soma_benchmark::seq::seq_benchmark;
use std::path::PathBuf;

/// Under target/, not /tmp: tmpfs rejects O_DIRECT.
fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn seq_direct_reads_whole_file_including_unaligned_tail() -> std::io::Result<()> {
    let p = path("soma_seq_direct.bin");
    let total = 3 * 4096 + 100;
    generate(&p, total)?;
    let result = seq_benchmark(&p, 4096, true, usize::MAX);
    std::fs::remove_file(&p)?;
    assert_eq!(result?.bytes, total);
    Ok(())
}

#[test]
fn rand_direct_reads_aligned_blocks() -> std::io::Result<()> {
    let p = path("soma_rand_direct.bin");
    generate(&p, 64 * 4096)?;
    let result = rand_benchmark(&p, 100, true);
    std::fs::remove_file(&p)?;
    assert_eq!(result?.len(), 100);
    Ok(())
}

#[test]
fn rand_qd_direct_reads_from_every_thread() -> std::io::Result<()> {
    use soma_benchmark::random_read::rand_qd_benchmark;
    let p = path("soma_rand_qd_direct.bin");
    generate(&p, 64 * 4096)?;
    let result = rand_qd_benchmark(&p, 101, 8, true);
    std::fs::remove_file(&p)?;
    assert_eq!(result?.latencies.len(), 101);
    Ok(())
}

#[test]
fn leaf_reader_direct_matches_file_contents() -> std::io::Result<()> {
    use soma_benchmark::challenge::LeafReader;
    let p = path("soma_leaf_reader_direct.bin");
    // Not a multiple of 4 KiB, so the last leaf is partial and ends mid-block.
    let total = 5 * 4096 + 300;
    generate(&p, total)?;
    let bytes = std::fs::read(&p)?;
    let result = (|| {
        // 1000 B leaves straddle block boundaries; 4096 B leaves align with them.
        for leaf_size in [32, 1000, 1024, 4096, 6000] {
            let mut reader = LeafReader::open(&p, leaf_size, true)?;
            for (i, expected) in bytes.chunks(leaf_size).enumerate() {
                assert_eq!(reader.read(i)?, expected, "leaf_size={leaf_size} i={i}");
            }
            assert!(reader.read(bytes.len().div_ceil(leaf_size)).is_err());
        }
        Ok(())
    })();
    std::fs::remove_file(&p)?;
    result
}

#[test]
fn challenge_rounds_pass_with_direct_reads() -> std::io::Result<()> {
    use soma_benchmark::challenge::challenge_benchmark;
    use soma_benchmark::merkle::MerkleAlgo;
    let p = path("soma_challenge_direct.bin");
    generate(&p, (1 << 20) + 123)?;
    let run = challenge_benchmark(&p, 1024, 459, 2, true, MerkleAlgo::Sha256);
    std::fs::remove_file(&p)?;
    let run = run?;
    assert_eq!(run.leaf_count, 1025);
    assert_eq!(run.rounds.len(), 2);
    // 1025 leaves pad to 2048: 11 siblings. The 123 B last leaf may be drawn,
    // so allow for it.
    assert!(run.response_bytes <= 459 * (1024 + 32 * 11));
    assert!(run.response_bytes > 458 * (1024 + 32 * 11));
    Ok(())
}
