use crate::file::{SECTOR_ALIGN, aligned, open_direct};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct SeqResult {
    pub bytes: usize,
    pub runtime: Duration,
}

impl SeqResult {
    pub fn throughput_mib_s(&self) -> f64 {
        self.bytes as f64 / 1024.0 / 1024.0 / self.runtime.as_secs_f64()
    }
}

pub fn seq_benchmark(
    path: &Path,
    buf_size: usize,
    direct: bool,
    limit: usize,
) -> io::Result<SeqResult> {
    if buf_size == 0 || !buf_size.is_multiple_of(SECTOR_ALIGN) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid buffer size",
        ));
    }
    let mut file = if direct {
        open_direct(path)?
    } else {
        File::open(path)?
    };
    let mut raw = vec![0u8; buf_size + SECTOR_ALIGN];
    let buf = aligned(&mut raw, buf_size);
    let mut total = 0;
    let start = Instant::now();
    while total < limit {
        let bytes = file.read(buf)?;
        if bytes == 0 {
            break;
        }
        total += bytes;
    }
    let runtime = start.elapsed();
    Ok(SeqResult {
        bytes: total,
        runtime,
    })
}
