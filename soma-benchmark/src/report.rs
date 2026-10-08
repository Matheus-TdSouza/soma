use crate::hash::HashResult;
use crate::seq::SeqResult;
use crate::stats::LatencyStats;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Duration;

pub const HEADER: &str = "label,test,param,rep,metric,value";

pub struct Row {
    pub test: &'static str,
    pub param: String,
    pub rep: u32,
    pub metric: &'static str,
    pub value: String,
}

pub struct RunMeta {
    pub os: &'static str,
    pub arch: &'static str,
    pub cpu_count: usize,
    pub tool_version: &'static str,
    pub timestamp_unix: u64,
    pub file_size_bytes: usize,
    pub hash_size_bytes: usize,
    pub reps: u32,
}

pub fn us(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

pub fn seq_rows(buf_size: usize, rep: u32, r: &SeqResult) -> Vec<Row> {
    vec![
        Row {
            test: "seq_direct",
            param: buf_size.to_string(),
            rep,
            metric: "runtime_us",
            value: format!("{:.3}", us(r.runtime)),
        },
        Row {
            test: "seq_direct",
            param: buf_size.to_string(),
            rep,
            metric: "throughput_mib_s",
            value: format!("{:.3}", r.throughput_mib_s()),
        },
    ]
}

pub fn rand_rows(rep: u32, reads: usize, stats: &LatencyStats) -> Vec<Row> {
    let metrics: [(&str, String); 8] = [
        ("reads", reads.to_string()),
        ("p50_us", format!("{:.3}", us(stats.p50))),
        ("p90_us", format!("{:.3}", us(stats.p90))),
        ("p99_us", format!("{:.3}", us(stats.p99))),
        ("max_us", format!("{:.3}", us(stats.max))),
        ("mean_us", format!("{:.3}", us(stats.mean))),
        ("std_dev_us", format!("{:.3}", us(stats.std_dev))),
        ("iops", format!("{:.3}", stats.iops)),
    ];

    metrics
        .into_iter()
        .map(|(metric, value)| Row {
            test: "rand_direct",
            param: "4096".to_string(),
            rep,
            metric,
            value,
        })
        .collect()
}

pub fn hash_rows(rep: u32, r: &HashResult) -> Vec<Row> {
    let metrics = [
        ("runtime_us", format!("{:.3}", us(r.runtime))),
        ("throughput_mib_s", format!("{:.3}", r.throughput_mib_s())),
    ];

    metrics
        .into_iter()
        .map(|(metric, value)| Row {
            test: "hash",
            param: r.algo.name().to_string(),
            rep,
            metric,
            value,
        })
        .collect()
}

pub fn meta_rows(meta: &RunMeta) -> Vec<Row> {
    let metrics = [
        ("os", meta.os.to_string()),
        ("arch", meta.arch.to_string()),
        ("cpu_count", meta.cpu_count.to_string()),
        ("timestamp_unix", meta.timestamp_unix.to_string()),
        ("file_size_bytes", meta.file_size_bytes.to_string()),
        ("hash_size_bytes", meta.hash_size_bytes.to_string()),
        ("tool_version", meta.tool_version.to_string()),
        ("reps", meta.reps.to_string()),
    ];

    metrics
        .into_iter()
        .map(|(metric, value)| Row {
            test: "meta",
            param: String::new(),
            rep: 0,
            metric,
            value,
        })
        .collect()
}

pub fn generate_rows(bytes: usize, runtime: Duration) -> Vec<Row> {
    let throughput = bytes as f64 / 1024.0 / 1024.0 / runtime.as_secs_f64();
    let metrics = [
        ("runtime_us", format!("{:.3}", us(runtime))),
        ("throughput_mib_s", format!("{:.3}", throughput)),
    ];

    metrics
        .into_iter()
        .map(|(metric, value)| Row {
            test: "write",
            param: bytes.to_string(),
            rep: 0,
            metric,
            value,
        })
        .collect()
}

pub fn write_row(out: &mut impl Write, label: &str, row: &Row) -> io::Result<()> {
    writeln!(
        out,
        "{},{},{},{},{},{}",
        label, row.test, row.param, row.rep, row.metric, row.value
    )
}

pub struct CsvWriter {
    out: BufWriter<File>,
    label: String,
}

impl CsvWriter {
    pub fn create(path: &Path, label: &str) -> io::Result<Self> {
        let mut out = BufWriter::new(File::create(path)?);
        writeln!(out, "{}", HEADER)?;
        out.flush()?;
        Ok(Self {
            out,
            label: label.to_string(),
        })
    }

    pub fn write(&mut self, rows: &[Row]) -> io::Result<()> {
        for row in rows {
            write_row(&mut self.out, &self.label, row)?;
        }
        self.out.flush()
    }
}
