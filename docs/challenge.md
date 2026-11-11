# Challenge protocol

A node that stores a file committed as `(root, leaf_count)` proves it still holds the data by answering challenges: a public random seed picks `k` leaves, the node returns each leaf with its Merkle path, and the answer counts only if it arrives before a deadline. Implemented in `soma-challenge`; `soma-benchmark challenge` runs real rounds against a file on disk.

Rounds run off chain: the provider answers verifiers directly. Only a contested answer goes to the contract, as a single proof of about 1 KB, so the on-chain cost stays near zero as long as someone is watching.

## Round

1. A seed `s` (32 bytes) becomes public. Where it comes from (block hash, randomness beacon) is not decided yet; it must be unpredictable to the node until this moment.
2. Both sides derive the challenged indices for `j = 0 .. k-1`:

   ```
   d_j = SHA-256("soma/challenge/v1" || s || root || le64(leaf_count) || le64(j))
   i_j = le128(d_j[0..16]) mod leaf_count
   ```

   Every field has a fixed width, so the encoding is unambiguous. The modulo bias is below `leaf_count / 2^128`. Draws are independent and may repeat. The verifier must reproduce this bit for bit, so any change needs a new domain tag.
3. The node sends verifiers, off chain and in order, `(leaf_j, proof_j)` for every `j`.
4. A verifier rejects the round (`check_round`) if the answer arrived after the deadline, has the wrong number of answers, has `proof_j.index != i_j`, or any proof fails `verify` against the commitment.

## Disputes

When a verifier rejects a round it contests one position `j`. The provider must then post `(leaf_j, proof_j)` to the contract, which runs `check_answer`: it derives only `i_j` (one SHA-256) and verifies one Merkle path. If the provider does not post a valid answer within the dispute window, it is penalized.

A dispute carries one leaf plus 32 bytes per tree level:

| Shard | Leaf 256 B | Leaf 1 KiB | Leaf 4 KiB |
|---|---|---|---|
| 1 GiB | 960 B | 1,664 B | 4,672 B |
| 100 GiB | 1,184 B | 1,888 B | 4,896 B |

At 16 gas per calldata byte plus about 30 SHA-256 precompile calls, that is tens of thousands of gas per dispute, paid only when someone contests.

## Number of challenges

If a node lost a fraction `f` of the leaves, one draw misses the loss with probability `1 - f`, and `k` independent draws all miss it with probability `(1 - f)^k`. The smallest `k` with `(1 - f)^k <= 1 - c` is `challenge_count(f, c)`:

| Missing `f` | Confidence `c` | `k` |
|---|---|---|
| 1% | 99% | **459** |
| 1% | 99.9% | 688 |
| 5% | 99% | 90 |
| 0.1% | 99% | 4,603 |

The protocol uses `f = 1%`, `c = 99%`, so `k = 459`. A node missing less than 1% is caught less often per round, but repeated rounds catch it eventually.

## Deadline

The node answers with one storage read per challenge, so the tree must stay in memory (or at least everything but the leaves). Reads are sequential in the reference prover. The time for `k` reads is close to normal, which gives:

```
deadline = k · mean + z · √k · σ + overhead
```

where `mean` and `σ` come from `rand --direct` on the storage class, `z` sets how often an honest node misses the deadline (z = 2: 2.3%, z = 4: 3·10⁻⁵, z = 6: 10⁻⁹), and `overhead` covers everything except disk reads: network round trip, sending the response and the verifier's own timing error. `round_deadline` computes it.

With the D01 measurements (`docs/benchmarks.md`) and `k = 459`:

| Drive | `k · mean` | `√k · σ` | z = 4, before overhead |
|---|---|---|---|
| D01-nvme1 | 52.8 ms | 0.62 ms | 55.3 ms |
| D01-hdd1 | 4.136 s | 57 ms | 4.364 s |

Checked against real rounds on a second machine (Linux, NVMe, 1 GiB file, 1 KiB leaves): `rand --direct` gave mean 59.7 µs and σ 25.1 µs, predicting 27.4 ± 1.1 ms (z = 2). Ten measured rounds took 24.9 to 26.1 ms. The same rounds without `--direct`, served from the page cache, took 0.5 ms.

## Response size

Off chain, a round sends `k` answers. For a 1 GiB file:

| Leaf | Depth | Per answer | Per round (k = 459) |
|---|---|---|---|
| 256 B | 22 | 960 B | 441 KB |
| 1 KiB | 20 | 1,664 B | 764 KB |

Sending the response belongs in `overhead`. 764 KB takes about 61 ms at 100 Mbit/s, more than the NVMe reads themselves.

## Leaf size

`leaf_size` stays a parameter until the network fixes it as a constant. Smaller leaves shrink both the round (441 KB at 256 B against 764 KB at 1 KiB) and the dispute, at the cost of a slower commitment (`soma-benchmark merkle`). With disputes carrying a single proof, the on-chain side no longer decides the size by itself; off-chain bandwidth and commitment time now weigh as much.

## Future optimization: multiproof

The `k` paths of a round share their upper levels: with 459 leaves in a tree of depth 20, roughly the top 8 levels are almost entirely shared. A multiproof sends each needed sibling once, and nodes that the verifier can compute from the leaves not at all. Simulated for a 1 GiB file and `k = 459`:

| Leaf | Siblings saved | Round |
|---|---|---|
| 256 B | 44% | 441 → 298 KB (−32%) |
| 1 KiB | 49% | 764 → 621 KB (−19%) |
| 4 KiB | 54% | 2,144 → 2,000 KB (−7%) |

The leaves themselves cannot be shared, so the gain shrinks as leaves grow. It pays off only while `k` stays in the hundreds, and it does not apply to disputes, which carry a single path. Noted as future work, not implemented.

## Open questions

- **Seed source.** Who publishes the seed, and how the start of the deadline is fixed.
- **Who watches.** Disputes only happen if verifiers check rounds. Verifiers need a reason to do the work (for example a share of the penalty), and providers must not be able to tell which rounds are watched.
- **The deadline is enforced off chain only.** A dispute proves that an answer is wrong or missing, not that it was late: the dispute window is measured in blocks, not milliseconds. A provider that deleted data but can fetch the contested leaf within the window passes the dispute.
- **One deadline or one per storage class.** A single deadline must fit the slowest accepted class (HDD, about 4.4 s), and gives faster nodes that much slack.
- **What the deadline protects.** Sampling catches deleted data. The deadline only rules out sources slower than the deadline. A node missing 1% of a file needs about 5 of the 459 leaves from elsewhere, which one parallel network fetch covers well within 4 s, and often within 55 ms. Stopping a node from outsourcing its storage needs more than a deadline.
- **Round-time tails.** `z` assumes the per-read numbers describe whole rounds. `soma-benchmark challenge` measures rounds directly and should be run on each storage class before fixing the deadline.
