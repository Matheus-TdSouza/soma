use crate::hasher::MerkleHasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Commitment {
    pub root: [u8; 32],
    pub leaf_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub index: usize,
    pub siblings: Vec<[u8; 32]>,
}

pub fn verify<H: MerkleHasher>(
    commitment: &Commitment,
    leaf: &[u8],
    proof: &Proof,
) -> bool {
    if proof.index >= commitment.leaf_count {
        return false;
    }

    let Some(padded) = commitment.leaf_count.checked_next_power_of_two() else {
        return false;
    };
    let depth = padded.trailing_zeros() as usize;

    if proof.siblings.len() != depth {
        return false;
    }

    let mut hash = H::leaf(leaf);
    let mut idx = proof.index;

    for sibling in &proof.siblings {
        hash = if idx % 2 == 0 {
            H::node(&hash, sibling)
        } else {
            H::node(sibling, &hash)
        };

        idx /= 2;
    }

    hash == commitment.root
}