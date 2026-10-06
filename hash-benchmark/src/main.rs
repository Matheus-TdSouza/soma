use sha2::{Sha256, Digest};
use std::time::Instant;
use std::fs::File;
use std::io;
use std::io::Read;
const TAMANHO: usize = 1073741824;
const TAMANHO_F64: f64 = TAMANHO as f64;

fn main() {
    println!("{:?}", benchmark_leitor());
}

fn benchmark_hash() {
    let mut dados: Vec<u8> = Vec::with_capacity(TAMANHO);
    dados.resize(TAMANHO, 1);
    let inicio = Instant::now();
    let hash = Sha256::digest(dados);
    let duracao = inicio.elapsed();
    print!("Hash: ");
    for dado in hash {
        print!("{:02x}", dado);
    }
    println!();
    println!("Tempo: {:?}", duracao);
    let duracao_secs: f64 = duracao.as_secs_f64();
    let throughput = (TAMANHO_F64 / 1024.0 / 1024.0) / duracao_secs;
    println!("Throughput: {:?} MiB/s", throughput);
}

fn benchmark_leitor() -> io::Result<usize> {
    let mut file = File::open("../benchmark-dd.bin")?;
    let mut buffer = [0u8; 4096];
    let bytes = file.read(&mut buffer)?;
    Ok(bytes)
}
