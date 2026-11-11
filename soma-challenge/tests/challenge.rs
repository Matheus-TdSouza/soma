use soma_challenge::{
    CONFIDENCE, Challenge, K, MISSING_FRACTION, ReadLatency, Rejection, Response, challenge_count,
    check_answer, check_response, check_round, miss_probability, respond, round_deadline,
};
use soma_merkle::{MerkleTree, Sha256Hasher};
use std::io;
use std::time::Duration;

type H = Sha256Hasher;
const LEAF_SIZE: usize = 16;

fn data(leaves: usize) -> Vec<u8> {
    (0..leaves * LEAF_SIZE)
        .map(|b| (b * 7 + b / 251) as u8)
        .collect()
}

fn leaf(data: &[u8], i: usize) -> io::Result<Vec<u8>> {
    Ok(data.chunks(LEAF_SIZE).nth(i).unwrap().to_vec())
}

fn seed(n: u64) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[..8].copy_from_slice(&n.to_le_bytes());
    s
}

#[test]
fn k_is_459_for_one_percent_at_99_percent() {
    assert_eq!(challenge_count(MISSING_FRACTION, CONFIDENCE), Some(K));
    assert_eq!(K, 459);
    assert!(miss_probability(0.01, 459) <= 0.01);
    assert!(miss_probability(0.01, 458) > 0.01);
}

#[test]
fn challenge_count_lands_on_exact_bounds() {
    // 0.5^2 = 0.25 exactly: 2 draws suffice for 75%.
    assert_eq!(challenge_count(0.5, 0.75), Some(2));
    assert_eq!(challenge_count(0.5, 0.5), Some(1));
    assert_eq!(challenge_count(0.1, 0.9), Some(22));
}

#[test]
fn challenge_count_rejects_out_of_range_inputs() {
    for (f, c) in [
        (0.0, 0.99),
        (1.0, 0.99),
        (0.01, 0.0),
        (0.01, 1.0),
        (-0.1, 0.5),
        (f64::NAN, 0.5),
    ] {
        assert_eq!(challenge_count(f, c), None, "f={f} c={c}");
    }
}

#[test]
fn indices_are_deterministic_in_range_and_depend_on_seed_and_root() {
    let a = MerkleTree::<H>::build(&data(1000), LEAF_SIZE)
        .unwrap()
        .commitment();
    let mut other = data(1000);
    other[0] ^= 1;
    let b = MerkleTree::<H>::build(&other, LEAF_SIZE)
        .unwrap()
        .commitment();
    let c = Challenge {
        seed: seed(1),
        k: K,
    };

    let ia = c.indices(&a);
    assert_eq!(ia.len(), K);
    assert!(ia.iter().all(|&i| i < 1000));
    assert_eq!(ia, c.indices(&a));
    assert_ne!(
        ia,
        Challenge {
            seed: seed(2),
            k: K
        }
        .indices(&a)
    );
    assert_ne!(ia, c.indices(&b));
}

#[test]
fn indices_are_roughly_uniform() {
    let commitment = MerkleTree::<H>::build(&data(10), LEAF_SIZE)
        .unwrap()
        .commitment();
    let draws = Challenge {
        seed: seed(7),
        k: 100_000,
    }
    .indices(&commitment);
    let mut counts = [0usize; 10];
    for i in draws {
        counts[i] += 1;
    }
    // Expected 10,000 per bucket, standard deviation ~95.
    assert!(
        counts.iter().all(|&c| (9_500..=10_500).contains(&c)),
        "{counts:?}"
    );
}

#[test]
fn honest_response_passes_for_every_tree_shape() {
    for leaves in 1..=9 {
        let data = data(leaves);
        let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
        let c = Challenge {
            seed: seed(leaves as u64),
            k: K,
        };
        let response = respond(&tree, &c, |i| leaf(&data, i)).unwrap();
        assert_eq!(response.answers.len(), K);
        assert_eq!(
            check_response::<H>(&tree.commitment(), &c, &response),
            Ok(())
        );
    }
}

#[test]
fn node_missing_one_percent_is_caught_about_99_percent_of_the_time() {
    let leaves = 10_000;
    let data = data(leaves);
    let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
    let commitment = tree.commitment();
    // Deleted every 100th leaf and answers those with zeros.
    let cheat = |i: usize| -> io::Result<Vec<u8>> {
        if i.is_multiple_of(100) {
            Ok(vec![0; LEAF_SIZE])
        } else {
            leaf(&data, i)
        }
    };

    let rounds = 300;
    let caught = (0..rounds)
        .filter(|&r| {
            let c = Challenge {
                seed: seed(r),
                k: K,
            };
            let response = respond(&tree, &c, cheat).unwrap();
            matches!(
                check_response::<H>(&commitment, &c, &response),
                Err(Rejection::InvalidProof { .. })
            )
        })
        .count();
    // Expected ~297; escaping more than 12 times has probability < 1e-6.
    assert!(
        caught >= rounds as usize - 12,
        "caught {caught} of {rounds}"
    );
}

fn honest(c: &Challenge) -> (soma_merkle::Commitment, Response) {
    let data = data(100);
    let tree = MerkleTree::<H>::build(&data, LEAF_SIZE).unwrap();
    (
        tree.commitment(),
        respond(&tree, c, |i| leaf(&data, i)).unwrap(),
    )
}

#[test]
fn missing_or_extra_answers_are_rejected() {
    let c = Challenge {
        seed: seed(3),
        k: K,
    };
    let (commitment, response) = honest(&c);

    let mut short = response.clone();
    short.answers.pop();
    assert_eq!(
        check_response::<H>(&commitment, &c, &short),
        Err(Rejection::WrongCount {
            expected: K,
            got: K - 1
        })
    );

    let mut long = response.clone();
    long.answers.push(response.answers[0].clone());
    assert_eq!(
        check_response::<H>(&commitment, &c, &long),
        Err(Rejection::WrongCount {
            expected: K,
            got: K + 1
        })
    );
}

#[test]
fn answer_for_a_leaf_that_was_not_asked_is_rejected() {
    let c = Challenge {
        seed: seed(4),
        k: K,
    };
    let (commitment, response) = honest(&c);
    let position = (0..K)
        .find(|&p| response.answers[p].proof.index != response.answers[0].proof.index)
        .unwrap();

    // A valid proof, but for the leaf asked at position 0.
    let mut swapped = response.clone();
    swapped.answers[position] = response.answers[0].clone();
    assert_eq!(
        check_response::<H>(&commitment, &c, &swapped),
        Err(Rejection::WrongIndex {
            position,
            expected: response.answers[position].proof.index,
            got: response.answers[0].proof.index,
        })
    );
}

#[test]
fn late_response_is_rejected_even_if_correct() {
    let c = Challenge {
        seed: seed(5),
        k: K,
    };
    let (commitment, response) = honest(&c);
    let deadline = Duration::from_millis(53);

    assert_eq!(
        check_round::<H>(&commitment, &c, &response, deadline, deadline),
        Ok(())
    );
    let late = deadline + Duration::from_nanos(1);
    assert_eq!(
        check_round::<H>(&commitment, &c, &response, late, deadline),
        Err(Rejection::Late {
            elapsed: late,
            deadline
        })
    );
}

#[test]
fn response_size_counts_leaves_and_siblings() {
    let c = Challenge {
        seed: seed(6),
        k: K,
    };
    let (_, response) = honest(&c);
    // 100 leaves pad to 128: 7 siblings per proof.
    assert_eq!(response.size_bytes(), K * (LEAF_SIZE + 32 * 7));
}

#[test]
fn round_deadline_matches_measured_d01_latencies() {
    // docs/benchmarks.md, random 4 KiB reads.
    let nvme = ReadLatency {
        mean: Duration::from_micros(115),
        std_dev: Duration::from_micros(29),
    };
    let hdd = ReadLatency {
        mean: Duration::from_micros(9_011),
        std_dev: Duration::from_micros(2_661),
    };

    assert_eq!(
        round_deadline(K, nvme, 0.0, Duration::ZERO),
        Duration::from_micros(52_785)
    );
    assert_eq!(
        round_deadline(K, hdd, 0.0, Duration::ZERO),
        Duration::from_micros(4_136_049)
    );

    // z = 2 reproduces the ± column: 2·√459·29 µs ≈ 1.24 ms.
    let spread =
        round_deadline(K, nvme, 2.0, Duration::ZERO) - round_deadline(K, nvme, 0.0, Duration::ZERO);
    assert!(
        spread > Duration::from_micros(1_240) && spread < Duration::from_micros(1_245),
        "{spread:?}"
    );

    let overhead = Duration::from_millis(100);
    assert_eq!(
        round_deadline(K, nvme, 0.0, overhead),
        round_deadline(K, nvme, 0.0, Duration::ZERO) + overhead
    );
}

#[test]
fn challenge_count_matches_table_in_docs() {
    for (f, c, k) in [
        (0.01, 0.99, 459),
        (0.01, 0.999, 688),
        (0.05, 0.99, 90),
        (0.001, 0.99, 4_603),
    ] {
        assert_eq!(challenge_count(f, c), Some(k), "f={f} c={c}");
    }
}

#[test]
fn index_matches_indices_and_rejects_missing_positions() {
    let c = Challenge {
        seed: seed(8),
        k: K,
    };
    let (commitment, _) = honest(&c);
    let all = c.indices(&commitment);
    for (position, &i) in all.iter().enumerate() {
        assert_eq!(c.index(&commitment, position), Some(i));
    }
    assert_eq!(c.index(&commitment, K), None);
    let empty = soma_merkle::Commitment {
        root: [0; 32],
        leaf_count: 0,
    };
    assert_eq!(c.index(&empty, 0), None);
}

#[test]
fn contested_answer_is_checked_on_its_own() {
    let c = Challenge {
        seed: seed(9),
        k: K,
    };
    let (commitment, response) = honest(&c);

    for (position, answer) in response.answers.iter().enumerate() {
        assert_eq!(check_answer::<H>(&commitment, &c, position, answer), Ok(()));
    }

    let answer = &response.answers[10];
    let mut bad_leaf = answer.clone();
    bad_leaf.leaf[0] ^= 1;
    assert_eq!(
        check_answer::<H>(&commitment, &c, 10, &bad_leaf),
        Err(Rejection::InvalidProof { position: 10 })
    );

    // A valid answer for another position does not settle this one.
    let other = (0..K)
        .find(|&p| response.answers[p].proof.index != answer.proof.index)
        .unwrap();
    assert!(matches!(
        check_answer::<H>(&commitment, &c, other, answer),
        Err(Rejection::WrongIndex { .. })
    ));

    assert_eq!(
        check_answer::<H>(&commitment, &c, K, answer),
        Err(Rejection::NoSuchPosition { position: K, k: K })
    );
}

#[test]
fn dispute_carries_one_proof_of_about_a_kilobyte() {
    // 100 leaves of 16 B: 7 siblings.
    let c = Challenge {
        seed: seed(10),
        k: K,
    };
    let (_, response) = honest(&c);
    assert_eq!(response.answers[0].size_bytes(), LEAF_SIZE + 32 * 7);
}
