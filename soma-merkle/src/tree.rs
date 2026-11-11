use crate::hasher::MerkleHasher;
use crate::proof::{Commitment, Proof};
use rayon::prelude::*;
use std::io::{self, Read};
use std::marker::PhantomData;

/// Bytes read from the input per block in `build_reader`, rounded down to a
/// multiple of the leaf size.
const READ_BLOCK: usize = 8 << 20;

pub struct MerkleTree<H: MerkleHasher> {
    levels: Vec<Vec<[u8; 32]>>,
    leaf_count: usize,
    _hasher: PhantomData<H>,
}

impl<H: MerkleHasher> MerkleTree<H> {
    pub fn build(data: &[u8], leaf_size: usize) -> Option<Self> {
        if leaf_size == 0 || data.is_empty() {
            return None;
        }

        let leaves = data.par_chunks(leaf_size).map(H::leaf).collect();
        Some(Self::from_leaves(leaves))
    }

    /// Builds the same tree as `build` over the whole contents of `reader`,
    /// holding only one block of input in memory at a time.
    pub fn build_reader<R: Read>(mut reader: R, leaf_size: usize) -> io::Result<Option<Self>> {
        if leaf_size == 0 {
            return Ok(None);
        }

        let block_size = (READ_BLOCK / leaf_size).max(1) * leaf_size;
        let mut buf = vec![0u8; block_size];
        let mut leaves = Vec::new();

        loop {
            let n = fill(&mut reader, &mut buf)?;
            leaves.par_extend(buf[..n].par_chunks(leaf_size).map(H::leaf));
            if n < block_size {
                break;
            }
        }

        if leaves.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self::from_leaves(leaves)))
    }

    fn from_leaves(mut leaves: Vec<[u8; 32]>) -> Self {
        let leaf_count = leaves.len();
        leaves.resize(leaf_count.next_power_of_two(), [0u8; 32]);

        let mut levels = Vec::new();
        let mut current = leaves;

        while current.len() > 1 {
            let next = current
                .chunks(2)
                .map(|pair| H::node(&pair[0], &pair[1]))
                .collect();
            levels.push(std::mem::replace(&mut current, next));
        }
        levels.push(current);

        Self {
            levels,
            leaf_count,
            _hasher: PhantomData,
        }
    }

    pub fn root(&self) -> [u8; 32] {
        self.levels.last().expect("tree has at least one level")[0]
    }

    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    pub fn prove(&self, i: usize) -> Option<Proof> {
        if i >= self.leaf_count {
            return None;
        }

        let mut idx = i;
        let mut siblings = Vec::with_capacity(self.levels.len() - 1);

        for level in &self.levels[..self.levels.len() - 1] {
            siblings.push(level[idx ^ 1]);
            idx /= 2;
        }

        Some(Proof { index: i, siblings })
    }

    pub fn commitment(&self) -> Commitment {
        Commitment {
            root: self.root(),
            leaf_count: self.leaf_count,
        }
    }
}

/// Reads until `buf` is full or the reader hits EOF; returns the bytes read.
fn fill<R: Read>(reader: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}
