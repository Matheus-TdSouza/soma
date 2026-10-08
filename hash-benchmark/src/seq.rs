use crate::file::{SECTOR_ALIGN, aligned, open_direct};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;

pub fn seq_benchmark(path: &Path, buf_size: usize, direct: bool) -> io::Result<usize> {
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
    loop {
        let bytes = file.read(buf)?;
        if bytes == 0 {
            break;
        }
        total += bytes;
    }
    let runtime = start.elapsed();
    let runtime_secs = runtime.as_secs_f64();
    let throughput = (total as f64 / 1024.0 / 1024.0) / runtime_secs;
    println!("Read: {:?} bytes", total);
    println!("Runtime: {:?}", runtime);
    println!("Throughput: {:.2} MiB/s", throughput);
    Ok(total)
}
