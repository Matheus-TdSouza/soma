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
