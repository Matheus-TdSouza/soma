use crate::file::{SECTOR_ALIGN, aligned, open_direct};
use rand::prelude::*;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};

pub fn rand_benchmark(path: &Path, n: usize, direct: bool) -> io::Result<Vec<Duration>> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid number of reads",
        ));
    }
    let mut file = if direct {
        open_direct(path)?
    } else {
        File::open(path)?
    };
    let file_size = file.metadata()?.len();
    let n_blocks = file_size / SECTOR_ALIGN as u64;
    if n_blocks == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File is too small",
        ));
    }
    let mut rng = rand::rng();
    let mut raw = vec![0u8; SECTOR_ALIGN * 2];
    let buf = aligned(&mut raw, SECTOR_ALIGN);
    let mut durations = Vec::with_capacity(n);
    for _ in 0..n {
        let block = rng.random_range(0..n_blocks);
        let offset = block * SECTOR_ALIGN as u64;
        let start = Instant::now();
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(buf)?;
        let duration = start.elapsed();
        durations.push(duration);
    }
    Ok(durations)
}
