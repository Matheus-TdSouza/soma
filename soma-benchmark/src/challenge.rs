use crate::file::{SECTOR_ALIGN, aligned, open_direct};
use crate::merkle::MerkleAlgo;
use rand::prelude::*;
use soma_challenge::{Challenge, check_response, respond};
use soma_merkle::{Blake3Hasher, MerkleHasher, MerkleTree, Sha256Hasher};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};

/// Reads single leaves from the committed file. With direct I/O it reads the
/// aligned 4 KiB blocks around the leaf and cuts the leaf out of them.
pub struct LeafReader {
    file: File,
    file_len: u64,
    leaf_size: usize,
    raw: Vec<u8>,
}

impl LeafReader {
    pub fn open(path: &Path, leaf_size: usize, direct: bool) -> io::Result<Self> {
        let file = if direct {
            open_direct(path)?
        } else {
            File::open(path)?
        };
        let file_len = file.metadata()?.len();
        // A leaf can straddle one more block than its size alone needs.
        let max_read = (leaf_size + SECTOR_ALIGN - 1).div_ceil(SECTOR_ALIGN) * SECTOR_ALIGN;
        Ok(Self {
            file,
            file_len,
            leaf_size,
            raw: vec![0u8; max_read + SECTOR_ALIGN],
        })
    }

    pub fn read(&mut self, i: usize) -> io::Result<Vec<u8>> {
        let start = i as u64 * self.leaf_size as u64;
        let end = (start + self.leaf_size as u64).min(self.file_len);
        if start >= end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "leaf index past end of file",
            ));
        }
        let block_start = start / SECTOR_ALIGN as u64 * SECTOR_ALIGN as u64;
        let span = (end - block_start) as usize;
        let read_len = span.div_ceil(SECTOR_ALIGN) * SECTOR_ALIGN;
        let buf = aligned(&mut self.raw, read_len);

        self.file.seek(SeekFrom::Start(block_start))?;
        // Stop once the leaf is in: the last block may be partial, and a
        // second read from an unaligned EOF is not portable under direct I/O.
        let mut filled = 0;
        while filled < span {
            match self.file.read(&mut buf[filled..])? {
                0 => return Err(io::ErrorKind::UnexpectedEof.into()),
                n => filled += n,
            }
        }
        Ok(buf[(start - block_start) as usize..span].to_vec())
    }
}

pub struct ChallengeRun {
    pub leaf_count: usize,
    pub build: Duration,
    /// Time to answer each round: k leaf reads plus proof assembly.
    pub rounds: Vec<Duration>,
    pub response_bytes: usize,
}

/// Commits to the file at `path`, then answers `rounds` challenges of `k`
/// leaves with fresh random seeds, checking every response.
pub fn challenge_benchmark(
    path: &Path,
    leaf_size: usize,
    k: usize,
    rounds: usize,
    direct: bool,
    algo: MerkleAlgo,
) -> io::Result<ChallengeRun> {
    match algo {
        MerkleAlgo::Sha256 => run::<Sha256Hasher>(path, leaf_size, k, rounds, direct),
        MerkleAlgo::Blake3 => run::<Blake3Hasher>(path, leaf_size, k, rounds, direct),
    }
}

fn run<H: MerkleHasher>(
    path: &Path,
    leaf_size: usize,
    k: usize,
    rounds: usize,
    direct: bool,
) -> io::Result<ChallengeRun> {
    let start = Instant::now();
    let tree = MerkleTree::<H>::build_reader(File::open(path)?, leaf_size)?.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "empty file or zero leaf size")
    })?;
    let build = start.elapsed();
    let commitment = tree.commitment();

    let mut reader = LeafReader::open(path, leaf_size, direct)?;
    let mut rng = rand::rng();
    let mut times = Vec::with_capacity(rounds);
    let mut response_bytes = 0;
    for _ in 0..rounds {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let challenge = Challenge { seed, k };

        let start = Instant::now();
        let response = respond(&tree, &challenge, |i| reader.read(i))?;
        times.push(start.elapsed());

        check_response::<H>(&commitment, &challenge, &response)
            .map_err(|r| io::Error::other(format!("response rejected: {r:?}")))?;
        response_bytes = response.size_bytes();
    }

    Ok(ChallengeRun {
        leaf_count: tree.leaf_count(),
        build,
        rounds: times,
        response_bytes,
    })
}
