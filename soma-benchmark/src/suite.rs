use soma_benchmark::file::generate;
use soma_benchmark::hash::hash_benchmark;
use soma_benchmark::random_read::rand_benchmark;
use soma_benchmark::report::{
    CsvWriter, RunMeta, generate_rows, hash_rows, meta_rows, rand_rows, seq_rows,
};
use soma_benchmark::seq::seq_benchmark;
use soma_benchmark::stats::latency_stats;
use std::io;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SEQ_BUFFERS: [usize; 4] = [4 << 10, 64 << 10, 1 << 20, 8 << 20];
const SEQ_LIMIT: usize = 1 << 30;
const RAND_READS: usize = 10_000;
const HASH_SIZE: usize = 1 << 30;

pub fn run(
    label: &str,
    path: &Path,
    size: usize,
    reps: u32,
    out: &Path,
    keep: bool,
) -> io::Result<()> {
    refuse_existing(path)?;
    refuse_existing(out)?;

    let mut csv = CsvWriter::create(out, label)?;
    let meta = RunMeta {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        cpu_count: std::thread::available_parallelism().map_or(1, |n| n.get()),
        tool_version: env!("CARGO_PKG_VERSION"),
        timestamp_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        file_size_bytes: size,
        hash_size_bytes: HASH_SIZE,
        reps,
    };
    csv.write(&meta_rows(&meta))?;

    eprintln!("generating {} bytes at {}", size, path.display());
    let start = Instant::now();
    generate(path, size)?;
    csv.write(&generate_rows(size, start.elapsed()))?;

    let result = measure(&mut csv, path, reps);

    if !keep && let Err(e) = std::fs::remove_file(path) {
        eprintln!("warning: could not delete {}: {}", path.display(), e);
    }

    result?;
    eprintln!("done, results in {}", out.display());
    Ok(())
}

fn measure(csv: &mut CsvWriter, path: &Path, reps: u32) -> io::Result<()> {
    for rep in 1..=reps {
        for buf_size in SEQ_BUFFERS {
            eprintln!("[{}/{}] seq {} bytes", rep, reps, buf_size);
            let r = seq_benchmark(path, buf_size, true, SEQ_LIMIT)?;
            csv.write(&seq_rows(buf_size, rep, &r))?;
        }

        eprintln!("[{}/{}] rand {} reads", rep, reps, RAND_READS);
        let mut durations = rand_benchmark(path, RAND_READS, true)?;
        let stats = latency_stats(&mut durations);
        csv.write(&rand_rows(rep, RAND_READS, &stats))?;

        eprintln!("[{}/{}] hash {} bytes", rep, reps, HASH_SIZE);
        for r in hash_benchmark(HASH_SIZE, None) {
            csv.write(&hash_rows(rep, &r))?;
        }
    }
    Ok(())
}

fn refuse_existing(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists, refusing to overwrite it",
                path.display()
            ),
        ));
    }
    Ok(())
}
