use hash_benchmark::file::generate;
use std::io;

fn generate_temp(name: &str, total: usize) -> io::Result<Vec<u8>> {
    let path = std::env::temp_dir().join(name);
    generate(&path, total)?;
    let bytes = std::fs::read(&path)?;
    std::fs::remove_file(&path)?;
    Ok(bytes)
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
