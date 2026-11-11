//! Storage challenges. A seed picks `k` leaves of a committed file; the node
//! answers with each leaf and its Merkle path, and the answer counts only if it
//! arrives before the deadline.
//!
//! Rounds run off chain between the provider and verifiers (`check_round`).
//! Only a contested answer goes to the contract, which checks that single
//! answer (`check_answer`) without deriving the other `k - 1` indices.

use sha2::{Digest, Sha256};
use soma_merkle::{Commitment, MerkleHasher, MerkleTree, Proof, verify};
use std::io;
use std::time::Duration;

/// Domain tag for index derivation, so challenge hashes never collide with
/// leaf or node hashes.
pub const DOMAIN: &[u8] = b"soma/challenge/v1";

/// A round should catch a node missing this fraction of its leaves...
pub const MISSING_FRACTION: f64 = 0.01;
/// ...with at least this probability.
pub const CONFIDENCE: f64 = 0.99;
/// `challenge_count(MISSING_FRACTION, CONFIDENCE)`.
pub const K: usize = 459;

/// Smallest `k` such that `k` independent uniform draws hit at least one
/// missing leaf with probability `confidence`, i.e. `(1 - f)^k <= 1 - c`.
/// Returns `None` unless both arguments lie strictly between 0 and 1.
pub fn challenge_count(missing_fraction: f64, confidence: f64) -> Option<usize> {
    let valid = |x: f64| x > 0.0 && x < 1.0;
    if !valid(missing_fraction) || !valid(confidence) {
        return None;
    }
    let target = 1.0 - confidence;
    let estimate = (target.ln() / (1.0 - missing_fraction).ln()).ceil() as usize;
    // The logarithms can land a hair off an integer; settle on the exact bound.
    let mut k = estimate.max(1);
    while k > 1 && miss_probability(missing_fraction, k - 1) <= target {
        k -= 1;
    }
    while miss_probability(missing_fraction, k) > target {
        k += 1;
    }
    Some(k)
}

/// Probability that `k` draws all land on leaves the node still holds.
pub fn miss_probability(missing_fraction: f64, k: usize) -> f64 {
    (1.0 - missing_fraction).powf(k as f64)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Challenge {
    pub seed: [u8; 32],
    pub k: usize,
}

impl Challenge {
    /// Leaf indices to answer, in order. Draws are independent, so an index
    /// can repeat; the detection bound assumes exactly that.
    pub fn indices(&self, commitment: &Commitment) -> Vec<usize> {
        if commitment.leaf_count == 0 {
            return Vec::new();
        }
        (0..self.k)
            .map(|j| challenge_index(&self.seed, commitment, j))
            .collect()
    }

    /// The leaf asked at `position`, or `None` if the round has no such
    /// position. Costs one hash, which is all a contract needs for a dispute.
    pub fn index(&self, commitment: &Commitment, position: usize) -> Option<usize> {
        (commitment.leaf_count > 0 && position < self.k)
            .then(|| challenge_index(&self.seed, commitment, position))
    }
}

/// `SHA-256(DOMAIN || seed || root || leaf_count || j)`, first 16 bytes as a
/// little-endian integer, reduced mod `leaf_count`. Every field has a fixed
/// width; the modulo bias is below `leaf_count / 2^128`.
fn challenge_index(seed: &[u8; 32], commitment: &Commitment, j: usize) -> usize {
    let digest = Sha256::new()
        .chain_update(DOMAIN)
        .chain_update(seed)
        .chain_update(commitment.root)
        .chain_update((commitment.leaf_count as u64).to_le_bytes())
        .chain_update((j as u64).to_le_bytes())
        .finalize();
    let x = u128::from_le_bytes(digest[..16].try_into().expect("16 bytes"));
    (x % commitment.leaf_count as u128) as usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub leaf: Vec<u8>,
    pub proof: Proof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub answers: Vec<Answer>,
}

impl Answer {
    /// Leaf plus 32 bytes per sibling: what a dispute puts in calldata.
    pub fn size_bytes(&self) -> usize {
        self.leaf.len() + 32 * self.proof.siblings.len()
    }
}

impl Response {
    /// What one round sends to a verifier off chain. Answers repeat the upper
    /// siblings they share; a multiproof would send those once (future work).
    pub fn size_bytes(&self) -> usize {
        self.answers.iter().map(Answer::size_bytes).sum()
    }
}

/// Answers `challenge` from `tree`, fetching each leaf with `read_leaf(i)`.
/// The tree must already be in memory: the deadline budgets one storage read
/// per challenge, not one per tree level.
pub fn respond<H: MerkleHasher>(
    tree: &MerkleTree<H>,
    challenge: &Challenge,
    mut read_leaf: impl FnMut(usize) -> io::Result<Vec<u8>>,
) -> io::Result<Response> {
    let answers = challenge
        .indices(&tree.commitment())
        .into_iter()
        .map(|i| {
            let proof = tree.prove(i).expect("challenge index < leaf_count");
            Ok(Answer {
                leaf: read_leaf(i)?,
                proof,
            })
        })
        .collect::<io::Result<_>>()?;
    Ok(Response { answers })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    Late {
        elapsed: Duration,
        deadline: Duration,
    },
    WrongCount {
        expected: usize,
        got: usize,
    },
    NoSuchPosition {
        position: usize,
        k: usize,
    },
    WrongIndex {
        position: usize,
        expected: usize,
        got: usize,
    },
    InvalidProof {
        position: usize,
    },
}

/// Checks that `response` answers exactly the leaves `challenge` picks, in
/// order, each with a valid proof against `commitment`.
pub fn check_response<H: MerkleHasher>(
    commitment: &Commitment,
    challenge: &Challenge,
    response: &Response,
) -> Result<(), Rejection> {
    let indices = challenge.indices(commitment);
    if response.answers.len() != indices.len() {
        return Err(Rejection::WrongCount {
            expected: indices.len(),
            got: response.answers.len(),
        });
    }
    for (position, (answer, &expected)) in response.answers.iter().zip(&indices).enumerate() {
        check_index_and_proof::<H>(commitment, position, expected, answer)?;
    }
    Ok(())
}

/// Checks one answer on its own: the dispute path. A verifier contests
/// `position`; the provider must produce the leaf asked there and its proof.
pub fn check_answer<H: MerkleHasher>(
    commitment: &Commitment,
    challenge: &Challenge,
    position: usize,
    answer: &Answer,
) -> Result<(), Rejection> {
    let expected = challenge
        .index(commitment, position)
        .ok_or(Rejection::NoSuchPosition {
            position,
            k: challenge.k,
        })?;
    check_index_and_proof::<H>(commitment, position, expected, answer)
}

fn check_index_and_proof<H: MerkleHasher>(
    commitment: &Commitment,
    position: usize,
    expected: usize,
    answer: &Answer,
) -> Result<(), Rejection> {
    if answer.proof.index != expected {
        return Err(Rejection::WrongIndex {
            position,
            expected,
            got: answer.proof.index,
        });
    }
    if !verify::<H>(commitment, &answer.leaf, &answer.proof) {
        return Err(Rejection::InvalidProof { position });
    }
    Ok(())
}

/// A full round: on time, then correct. `elapsed` runs from the moment the
/// seed became public to the moment the response arrived.
pub fn check_round<H: MerkleHasher>(
    commitment: &Commitment,
    challenge: &Challenge,
    response: &Response,
    elapsed: Duration,
    deadline: Duration,
) -> Result<(), Rejection> {
    if elapsed > deadline {
        return Err(Rejection::Late { elapsed, deadline });
    }
    check_response::<H>(commitment, challenge, response)
}

/// Random read latency of one storage class, as measured by `rand --direct`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadLatency {
    pub mean: Duration,
    pub std_dev: Duration,
}

/// Deadline for `k` sequential reads: `k·mean + z·√k·σ + overhead`. The sum of
/// `k` reads is close to normal, so `z` sets how often an honest node misses
/// the deadline (z = 2: 2.3%, z = 4: 3e-5, z = 6: 1e-9). `overhead` covers
/// everything that is not a disk read: network round trip, sending the
/// response, hashing.
pub fn round_deadline(k: usize, latency: ReadLatency, z: f64, overhead: Duration) -> Duration {
    latency.mean * k as u32 + latency.std_dev.mul_f64(z * (k as f64).sqrt()) + overhead
}
