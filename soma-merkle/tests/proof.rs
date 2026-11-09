use soma_merkle::{MerkleHasher, MerkleTree, verify};

const LEAF_SIZE: usize = 4;

/// `n` leaves of distinct bytes; the last one is shorter when `n` is odd, so
/// partial final leaves are covered too.
fn data(n: usize) -> Vec<u8> {
    let len = n * LEAF_SIZE - if n % 2 == 1 { 1 } else { 0 };
    (0..len).map(|b| b as u8).collect()
}

fn leaf(data: &[u8], i: usize) -> &[u8] {
    data.chunks(LEAF_SIZE).nth(i).unwrap()
}

fn round_trip<H: MerkleHasher>() {
    for n in 1..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        assert_eq!(c.leaf_count, n);
        for i in 0..n {
            let proof = tree.prove(i).unwrap();
            assert!(verify::<H>(&c, leaf(&data, i), &proof), "n={n} i={i}");
        }
    }
}

fn tampered_leaf<H: MerkleHasher>() {
    for n in 1..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        for i in 0..n {
            let proof = tree.prove(i).unwrap();
            let original = leaf(&data, i);
            for byte in 0..original.len() {
                let mut bad = original.to_vec();
                bad[byte] ^= 1;
                assert!(!verify::<H>(&c, &bad, &proof), "n={n} i={i} byte={byte}");
            }
            let mut longer = original.to_vec();
            longer.push(0);
            assert!(!verify::<H>(&c, &longer, &proof), "n={n} i={i} longer");
            let shorter = &original[..original.len() - 1];
            assert!(!verify::<H>(&c, shorter, &proof), "n={n} i={i} shorter");
        }
    }
}

fn tampered_sibling<H: MerkleHasher>() {
    for n in 1..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        for i in 0..n {
            let proof = tree.prove(i).unwrap();
            for s in 0..proof.siblings.len() {
                let mut bad = proof.clone();
                bad.siblings[s][0] ^= 1;
                assert!(!verify::<H>(&c, leaf(&data, i), &bad), "n={n} i={i} sibling={s}");
            }
        }
    }
}

fn proof_checked_as_other_index<H: MerkleHasher>() {
    for n in 2..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        for i in 0..n {
            let proof = tree.prove(i).unwrap();
            for j in (0..n).filter(|&j| j != i) {
                let mut moved = proof.clone();
                moved.index = j;
                assert!(!verify::<H>(&c, leaf(&data, i), &moved), "n={n} i={i} as j={j}, leaf i");
                assert!(!verify::<H>(&c, leaf(&data, j), &moved), "n={n} i={i} as j={j}, leaf j");
            }
        }
    }
}

fn index_out_of_range<H: MerkleHasher>() {
    for n in 1..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        assert!(tree.prove(n).is_none(), "n={n}");
        assert!(tree.prove(usize::MAX).is_none(), "n={n}");

        let proof = tree.prove(n - 1).unwrap();
        let padded = n.next_power_of_two();
        // `n - 1 + padded` shares its low bits with `n - 1`: the path alone would accept it.
        for index in (n..=padded).chain([n - 1 + padded, usize::MAX]) {
            let mut bad = proof.clone();
            bad.index = index;
            assert!(!verify::<H>(&c, leaf(&data, n - 1), &bad), "n={n} index={index}");
        }
    }
}

fn wrong_path_length<H: MerkleHasher>() {
    for n in 1..=9 {
        let data = data(n);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = tree.commitment();
        for i in 0..n {
            let proof = tree.prove(i).unwrap();

            let mut longer = proof.clone();
            longer.siblings.push([0u8; 32]);
            assert!(!verify::<H>(&c, leaf(&data, i), &longer), "n={n} i={i} extra");

            if !proof.siblings.is_empty() {
                let mut shorter = proof.clone();
                shorter.siblings.pop();
                assert!(!verify::<H>(&c, leaf(&data, i), &shorter), "n={n} i={i} missing");
            }
        }
    }
}

macro_rules! for_both_hashers {
    ($($name:ident),* $(,)?) => {
        mod sha256 {
            $( #[test] fn $name() { super::$name::<soma_merkle::Sha256Hasher>(); } )*
        }
        mod blake3 {
            $( #[test] fn $name() { super::$name::<soma_merkle::Blake3Hasher>(); } )*
        }
    };
}

for_both_hashers!(
    round_trip,
    tampered_leaf,
    tampered_sibling,
    proof_checked_as_other_index,
    index_out_of_range,
    wrong_path_length,
);

/// SHA-256 without the 0x00/0x01 prefixes, so `leaf(l || r) == node(l, r)`.
struct NoDomainHasher;

impl MerkleHasher for NoDomainHasher {
    fn leaf(data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        Sha256::digest(data).into()
    }

    fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        Sha256::new().chain_update(left).chain_update(right).finalize().into()
    }
}

#[test]
fn path_length_check_rejects_internal_node_as_leaf() {
    // With domain separation a short path already fails on the hash; this
    // hasher has none, so only the length check in `verify` stops the forgery.
    let data = data(4);
    let tree = MerkleTree::<NoDomainHasher>::build(&data, LEAF_SIZE).unwrap();
    let c = tree.commitment();
    let proof = tree.prove(0).unwrap();

    let l0 = NoDomainHasher::leaf(leaf(&data, 0));
    let l1 = NoDomainHasher::leaf(leaf(&data, 1));
    let mut forged_leaf = l0.to_vec();
    forged_leaf.extend_from_slice(&l1);
    let mut forged = proof.clone();
    forged.siblings.remove(0);

    assert!(verify::<NoDomainHasher>(&c, leaf(&data, 0), &proof));
    assert!(!verify::<NoDomainHasher>(&c, &forged_leaf, &forged));
}
