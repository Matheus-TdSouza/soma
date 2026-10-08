use hash_benchmark::report::{
    CsvWriter, HEADER, RunMeta, generate_rows, meta_rows, rand_rows, seq_rows, write_row,
};
use hash_benchmark::seq::SeqResult;
use hash_benchmark::stats::LatencyStats;
use std::time::Duration;

#[test]
fn seq_rows_writes_runtime_and_throughput() {
    let r = SeqResult {
        bytes: 1 << 20,
        runtime: Duration::from_millis(1),
    };
    let mut out = Vec::new();
    for row in seq_rows(4096, 2, &r) {
        write_row(&mut out, "pc-c", &row).unwrap();
    }
    let text = String::from_utf8(out).unwrap();
    assert_eq!(
        text,
        "pc-c,seq_direct,4096,2,runtime_us,1000.000\npc-c,seq_direct,4096,2,throughput_mib_s,1000.000\n"
    );
}

#[test]
fn hash_algo_name_round_trips() {
    use hash_benchmark::hash::{HashAlgo, parse_algo};
    for algo in [HashAlgo::Sha256, HashAlgo::Blake3, HashAlgo::Blake3Mt] {
        assert_eq!(parse_algo(algo.name()).map(|a| a.name()), Some(algo.name()));
    }
}

#[test]
fn rand_rows_writes_eight_metrics() {
    let stats = LatencyStats {
        p50: Duration::from_micros(185),
        p90: Duration::from_micros(240),
        p99: Duration::from_micros(355),
        max: Duration::from_micros(4500),
        mean: Duration::from_micros(192),
        std_dev: Duration::from_micros(60),
        iops: 5200.0,
    };
    let rows = rand_rows(1, 10000, &stats);
    let metrics: Vec<&str> = rows.iter().map(|r| r.metric).collect();
    assert_eq!(
        metrics,
        [
            "reads",
            "p50_us",
            "p90_us",
            "p99_us",
            "max_us",
            "mean_us",
            "std_dev_us",
            "iops"
        ]
    );
    assert!(
        rows.iter()
            .all(|r| r.test == "rand_direct" && r.param == "4096" && r.rep == 1)
    );
    assert_eq!(rows[0].value, "10000");
    assert_eq!(rows[1].value, "185.000");
}

#[test]
fn meta_rows_have_empty_param_and_rep_zero() {
    let meta = RunMeta {
        os: "windows",
        arch: "x86_64",
        cpu_count: 12,
        tool_version: "0.1.0",
        timestamp_unix: 1_791_470_000,
        file_size_bytes: 1 << 30,
        hash_size_bytes: 1 << 30,
        reps: 3,
    };
    let rows = meta_rows(&meta);
    assert_eq!(rows.len(), 8);
    assert!(
        rows.iter()
            .all(|r| r.test == "meta" && r.param.is_empty() && r.rep == 0)
    );
}

#[test]
fn generate_rows_writes_runtime_and_throughput() {
    let rows = generate_rows(1 << 20, Duration::from_millis(1));
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|r| r.test == "write" && r.param == "1048576" && r.rep == 0)
    );
    assert_eq!(rows[0].value, "1000.000");
    assert_eq!(rows[1].value, "1000.000");
}

#[test]
fn csv_writer_writes_header_then_rows() -> std::io::Result<()> {
    let path = std::env::temp_dir().join("soma_report_csv_writer.csv");
    let r = SeqResult {
        bytes: 1 << 20,
        runtime: Duration::from_millis(1),
    };
    {
        let mut csv = CsvWriter::create(&path, "pc-c")?;
        csv.write(&seq_rows(4096, 1, &r))?;
    }
    let text = std::fs::read_to_string(&path)?;
    std::fs::remove_file(&path)?;
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], HEADER);
    assert!(lines[1].starts_with("pc-c,seq_direct,4096,1,runtime_us,"));
    Ok(())
}
