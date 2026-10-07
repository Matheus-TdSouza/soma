use sha2::{Sha256, Digest};
use std::time::Instant;
use std::fs::File;
use std::io::{self, Read, Write};
use rand::prelude::*;

const BENCHMARK_PATH: &str = "../benchmark.bin";
const SECTOR_ALIGN: usize = 4096;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let size = args.get(2).and_then(|s| parse_size(s));
    match args.get(1).map(String::as_str) {
        Some("generate") => {
            let size = size.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid size"))?;
            generate(BENCHMARK_PATH, size)?;
            println!("File generated with {} bytes", size);
            Ok(())
        }
        Some("hash") => {
            let size = size.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid size"))?;
            hash_benchmark(size);
            Ok(())
        }
        Some("seq") => {
            let direct = args.get(3).map(String::as_str) == Some("direct");
            let buffer_size = size.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid buffer size"))?;
            seq_benchmark(buffer_size, direct)?;
            Ok(())
        }
        _ => {
            eprintln!("usage: hash-benchmark generate <size> | hash <size> | seq <buffer_size> [direct]");
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input"))
        }
    }
}

fn parse_size(s: &str) -> Option<usize> {
    let i = s.len().checked_sub(1)?;
    let (number, unit) = s.split_at_checked(i)?;
    let number = number.parse::<usize>().ok()?;
    let mult = match unit {
        "K" | "k" => 1 << 10,
        "M" | "m" => 1 << 20,
        "G" | "g" => 1 << 30,
        _ => return None,
    };
    number.checked_mul(mult)
}

fn generate(path: &str, total: usize) -> io::Result<()> {
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

fn hash_benchmark(size: usize) {
    let size_f64 = size as f64;
    let mut data: Vec<u8> = Vec::with_capacity(size);
    data.resize(size, 1);
    let start = Instant::now();
    let hash = Sha256::digest(data);
    let runtime = start.elapsed();
    print!("Hash: ");
    for data in hash {
        print!("{:02x}", data);
    }
    println!();
    println!("Runtime: {:?}", runtime);
    let runtime_secs = runtime.as_secs_f64();
    let throughput = (size_f64 / 1024.0 / 1024.0) / runtime_secs;
    println!("Throughput: {:?} MiB/s", throughput);
}

fn seq_benchmark(buf_size: usize, direct: bool) -> io::Result<usize> {
    if buf_size == 0 || !buf_size.is_multiple_of(SECTOR_ALIGN) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid buffer size"));
    }
    let mut file = if direct {
        open_direct(BENCHMARK_PATH)?
    } else {
        File::open(BENCHMARK_PATH)?
    };
    let mut raw = vec![0u8; buf_size + SECTOR_ALIGN];
    let offset = raw.as_ptr().align_offset(SECTOR_ALIGN);
    let buf = &mut raw[offset..offset + buf_size];
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

#[cfg(windows)]
fn open_direct(path: &str) -> io::Result<File> {
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
fn open_direct(_path: &str) -> io::Result<File> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "direct I/O only implemented on Windows"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn generate_temp(name: &str, total: usize) -> io::Result<Vec<u8>> {
        let path = std::env::temp_dir().join(name);
        let path = path.to_str().unwrap();
        generate(path, total)?;
        let bytes = std::fs::read(path)?;
        std::fs::remove_file(path)?;
        Ok(bytes)
    }

    #[test]
    fn parse_size_gigabytes() {
        assert_eq!(parse_size("4G"), Some(4 << 30));
    }

    #[test]
    fn parse_size_megabytes() {
        assert_eq!(parse_size("512m"), Some(512 << 20));
    }

    #[test]
    fn parse_size_kilobytes() {
        assert_eq!(parse_size("256k"), Some(256 << 10));
    }

    #[test]
    fn parse_size_zerobytes() {
        assert_eq!(parse_size("0K"), Some(0));
    }

    #[test]
    fn parse_size_rejects_invalid_unit() {
        assert_eq!(parse_size("4é"), None);
    }

    #[test]
    fn parse_size_rejects_num_only() {
        assert_eq!(parse_size("4096"), None);
    }

    #[test]
    fn parse_size_rejects_letter_only() {
        assert_eq!(parse_size("G"), None);
    }
    
    #[test]
    fn parse_size_rejects_invalid_format() {
        assert_eq!(parse_size("4GB"), None);
    }

    #[test]
    fn parse_size_rejects_signed_format() {
        assert_eq!(parse_size("-1G"), None);
    }
    
    #[test]
    fn parse_size_rejects_overflow() {
        assert_eq!(parse_size("20000000000G"), None);
    }

    #[test]
    fn parse_size_rejects_empty() {
        assert_eq!(parse_size(""), None);
    }

    #[test]
    fn generate_writes_exact_size() -> io::Result<()> {
        let total = (8 << 20) + 1;
        let bytes = generate_temp("soma_generate_exact.bin", total)?;
        assert_eq!(bytes.len(), total);
        Ok(())
    }

    #[test]
    fn generate_writes_zerobytes() -> io::Result<()> {
        let total = 0;
        let bytes = generate_temp("soma_generate_zerobytes.bin", total)?;
        assert_eq!(bytes.len(), total);
        Ok(())
    }

    #[test]
    fn generate_writes_small_size() -> io::Result<()> {
        let total = 10;
        let bytes = generate_temp("soma_generate_small_size.bin", total)?;
        assert_eq!(bytes.len(), total);
        Ok(())
    }

    #[test]
    fn generate_writes_non_zero_content() -> io::Result<()> {
        let total = 4096;
        let bytes = generate_temp("soma_generate_non_zero.bin", total)?;
        assert_eq!(bytes.len(), total);
        assert!(bytes.iter().any(|&b| b != 0));
        Ok(())
    }
}
