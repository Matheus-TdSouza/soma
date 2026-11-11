use soma_benchmark::file::generate;
use soma_benchmark::random_read::{rand_benchmark, rand_qd_benchmark};
use std::path::PathBuf;

fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn rand_qd_makes_every_read_across_threads() -> std::io::Result<()> {
    let p = path("soma_rand_qd.bin");
    generate(&p, 64 * 4096)?;
    // 103 reads over 4 threads do not split evenly; 3 reads over 8 threads
    // leave some threads idle.
    let result = (|| {
        for (n, qd) in [(103, 4), (3, 8), (50, 1)] {
            let run = rand_qd_benchmark(&p, n, qd, false)?;
            assert_eq!(run.latencies.len(), n, "n={n} qd={qd}");
            assert!(run.iops() > 0.0);
        }
        assert_eq!(rand_benchmark(&p, 20, false)?.len(), 20);
        Ok(())
    })();
    std::fs::remove_file(&p)?;
    result
}

#[test]
fn rand_qd_rejects_zero_reads_zero_depth_and_tiny_files() -> std::io::Result<()> {
    let p = path("soma_rand_qd_invalid.bin");
    generate(&p, 4096)?;
    let zero_reads = rand_qd_benchmark(&p, 0, 4, false).is_err();
    let zero_depth = rand_qd_benchmark(&p, 10, 0, false).is_err();
    let small = path("soma_rand_qd_small.bin");
    generate(&small, 100)?;
    let too_small = rand_qd_benchmark(&small, 10, 2, false).is_err();
    std::fs::remove_file(&p)?;
    std::fs::remove_file(&small)?;
    assert!(zero_reads && zero_depth && too_small);
    Ok(())
}
