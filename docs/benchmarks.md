# Benchmarks

Disk and hashing measurements collected with `soma-benchmark suite`. They size the cost of storing data and answering storage challenges: sequential reads bound the setup cost (reading a whole file to commit to it), random 4 KiB reads bound the cost of answering a challenge, and hash throughput bounds the CPU side of both.

Devices are anonymized. Each machine gets an ID (`D01`, `D02`, ...) and each drive a suffix (`D01-nvme1`). No names, usernames or paths are recorded.

## Devices

| Drive ID | CPU | Threads | RAM | Drive | Capacity | Used before run | OS | Date |
|---|---|---|---|---|---|---|---|---|
| D01-nvme1 | Intel Core i7, 9th gen | 8 | not recorded | NVMe PCIe 3.0 (model to confirm) | 1 TB | ~65% | Windows | 2026-10-08 |
| D01-hdd1 | Intel Core i7, 9th gen | 8 | not recorded | SATA HDD, RPM not recorded | 1 TB | ~0% | Windows | 2026-10-08 |

## Method

Tool version `0.1.0`. Every value below is the median of 3 repetitions.

- **Write**: time to create the benchmark file with random bytes (one run).
- **Sequential read**: unbuffered (`--direct`) read of the first 1 GiB of the file, one request at a time.
- **Random read**: 10,000 unbuffered 4 KiB reads at uniformly random aligned offsets over the whole file, one request at a time.
- **Hash**: throughput over a 1 GiB in-memory buffer; depends only on the CPU.

## Results

### Write

| Drive ID | File size | Throughput (MiB/s) |
|---|---|---|
| D01-nvme1 | 1 GiB | 819 |
| D01-hdd1 | 100 GiB | 197 |

### Sequential read (MiB/s)

| Drive ID | 4 KiB | 64 KiB | 1 MiB | 8 MiB |
|---|---|---|---|---|
| D01-nvme1 | 81 | 732 | 1613 | 1794 |
| D01-hdd1 | 49 | 204 | 204 | 204 |

### Random 4 KiB read

| Drive ID | File size | p50 (µs) | p90 (µs) | p99 (µs) | Mean (µs) | Std dev (µs) | IOPS |
|---|---|---|---|---|---|---|---|
| D01-nvme1 | 1 GiB | 106 | 159 | 175 | 115 | 29 | 8,701 |
| D01-hdd1 | 100 GiB | 8,981 | 12,450 | 14,159 | 9,011 | 2,661 | 111 |

### Hash (MiB/s)

| Machine | SHA-256 | BLAKE3 (1 thread) | BLAKE3 (all threads) |
|---|---|---|---|
| D01 | 282 | 4,444 | 31,669 |

## Derived: time to answer one challenge round

A round of `k` challenges read one at a time takes about `k × mean ± 2 × √k × std dev`. With `k = 459` (enough to catch a node missing 1% of its data with 99% confidence):

| Drive ID | Round time |
|---|---|
| D01-nvme1 | 52.8 ± 1.3 ms |
| D01-hdd1 | 4.14 ± 0.11 s |

## Caveats

- **Freshly written SSD data.** `suite` reads the file right after writing it. Consumer SSDs often keep recent writes in a faster cache area, so random read latency may be optimistic compared with data stored for weeks. An earlier ad hoc run on D01-nvme1 measured a p50 near 185 µs against 106 µs here; not yet confirmed.
- **HDD file size.** Random read latency on a hard drive grows with the area the head has to cover. The D01-hdd1 file covers about 11% of the disk, written at the start of an empty drive (fastest zone).
- **One request at a time.** All reads are synchronous. Drives serving parallel requests reach higher IOPS.
- **Windows only.** `--direct` is not implemented on other systems yet.

## Adding a device

1. Run `suite` once per drive, with an anonymous label:

   ```
   cargo run --release -p soma-benchmark -- suite --label D02-nvme1 --out D02-nvme1.csv
   ```

   For hard drives, use a large `--size` (for example `100G`) and a `--path` on that drive.
2. Record the specs for the devices table: CPU model, thread count, RAM, drive model, interface, capacity and how full the drive was.
3. Add a row to each table using the medians from the CSV.
