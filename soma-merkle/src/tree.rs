use std::marker::PhantomData;
use crate::hasher::MerkleHasher;
use crate::proof::{Commitment, Proof};

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

        let mut leaves: Vec<[u8; 32]> =
            data.chunks(leaf_size).map(H::leaf).collect();

        let leaf_count = leaves.len();
        let padded_count = leaf_count.next_power_of_two();

        leaves.resize(padded_count, [0u8; 32]);

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

        Some(Self {
            levels,
            leaf_count,
            _hasher: PhantomData,
        })
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
