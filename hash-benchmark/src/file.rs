use rand::prelude::*;
use std::fs::File;
use std::io::{self, Write};

pub const BENCHMARK_PATH: &str = "../benchmark.bin";
pub(crate) const SECTOR_ALIGN: usize = 4096;

pub fn generate(path: &str, total: usize) -> io::Result<()> {
    let mut file = File::create(path)?;
    let mut rng = rand::rng();
    let mut buf = vec![0u8; 8 << 20];
    let mut remaining = total;
    while remaining > 0 {
        let n = remaining.min(buf.len());
        rng.fill_bytes(&mut buf[..n]);
        file.write_all(&buf[..n])?;
        remaining -= n;
    }
    file.sync_all()?;
    Ok(())
}

pub(crate) fn aligned(raw: &mut [u8], size: usize) -> &mut [u8] {
    let offset = raw.as_ptr().align_offset(SECTOR_ALIGN);
    &mut raw[offset..offset + size]
}

#[cfg(windows)]
pub(crate) fn open_direct(path: &str) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::fs::OpenOptions;
    const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING)
        .open(path)?;
    Ok(file)
}

#[cfg(not(windows))]
pub(crate) fn open_direct(_path: &str) -> io::Result<File> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "direct I/O only implemented on Windows"))
}
