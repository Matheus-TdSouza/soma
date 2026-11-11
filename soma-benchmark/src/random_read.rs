use crate::file::{SECTOR_ALIGN, aligned, open_direct};
use rand::prelude::*;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Barrier;
use std::thread;
use std::time::{Duration, Instant};

pub struct RandRun {
    /// Latency of every read, from all threads.
    pub latencies: Vec<Duration>,
    /// Wall-clock time from the moment all threads start reading until the
    /// last one finishes.
    pub wall: Duration,
}

impl RandRun {
    /// Completed reads per second of wall-clock time. With more than one read
    /// in flight this is the number that matters; the latency-based IOPS in
    /// `LatencyStats` only holds for a queue depth of 1.
    pub fn iops(&self) -> f64 {
        self.latencies.len() as f64 / self.wall.as_secs_f64()
    }
}

/// `n` random aligned 4 KiB reads, one at a time.
pub fn rand_benchmark(path: &Path, n: usize, direct: bool) -> io::Result<Vec<Duration>> {
    Ok(rand_qd_benchmark(path, n, 1, direct)?.latencies)
}

/// `n` random aligned 4 KiB reads with up to `qd` in flight: `qd` threads,
/// each with its own file handle, issue synchronous reads. Separate handles
/// matter on Windows, where reads on one synchronous handle are serialized.
pub fn rand_qd_benchmark(path: &Path, n: usize, qd: usize, direct: bool) -> io::Result<RandRun> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid number of reads",
        ));
    }
    if qd == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Queue depth must be at least 1",
        ));
    }
    // Open every handle before timing starts.
    let files = (0..qd)
        .map(|_| {
            if direct {
                open_direct(path)
            } else {
                File::open(path)
            }
        })
        .collect::<io::Result<Vec<_>>>()?;
    let n_blocks = files[0].metadata()?.len() / SECTOR_ALIGN as u64;
    if n_blocks == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File is too small",
        ));
    }

    let barrier = Barrier::new(qd + 1);
    let (results, wall) = thread::scope(|s| {
        let handles: Vec<_> = files
            .into_iter()
            .enumerate()
            .map(|(t, file)| {
                let reads = n / qd + usize::from(t < n % qd);
                let barrier = &barrier;
                s.spawn(move || {
                    barrier.wait();
                    read_random_blocks(file, n_blocks, reads)
                })
            })
            .collect();
        barrier.wait();
        let start = Instant::now();
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("reader thread panicked"))
            .collect();
        (results, start.elapsed())
    });

    let mut latencies = Vec::with_capacity(n);
    for r in results {
        latencies.extend(r?);
    }
    Ok(RandRun { latencies, wall })
}

fn read_random_blocks(mut file: File, n_blocks: u64, reads: usize) -> io::Result<Vec<Duration>> {
    let mut rng = rand::rng();
    let mut raw = vec![0u8; SECTOR_ALIGN * 2];
    let buf = aligned(&mut raw, SECTOR_ALIGN);
    let mut durations = Vec::with_capacity(reads);
    for _ in 0..reads {
        let block = rng.random_range(0..n_blocks);
        let offset = block * SECTOR_ALIGN as u64;
        let start = Instant::now();
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(buf)?;
        durations.push(start.elapsed());
    }
    Ok(durations)
}
