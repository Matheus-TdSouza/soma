use std::time::Duration;

pub struct LatencyStats {
    pub p50: Duration,
    pub p90: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub std_dev: Duration,
    pub iops: f64,
}

pub fn percentile(sorted: &[Duration], p: usize) -> Duration {
    sorted[p * (sorted.len() - 1) / 100]
}

pub fn std_dev(durations: &[Duration]) -> Duration {
    let n = durations.len();
    if n < 2 {
        return Duration::ZERO;
    }
    let mean = durations.iter().map(Duration::as_secs_f64).sum::<f64>() / n as f64;
    let variance = durations
        .iter()
        .map(|d| (d.as_secs_f64() - mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;
    Duration::from_secs_f64(variance.sqrt())
}

pub fn latency_stats(durations: &mut [Duration]) -> LatencyStats {
    durations.sort();
    let n = durations.len();
    let total = durations.iter().sum::<Duration>();
    LatencyStats {
        p50: percentile(durations, 50),
        p90: percentile(durations, 90),
        p99: percentile(durations, 99),
        max: durations[n - 1],
        mean: total.div_f64(n as f64),
        std_dev: std_dev(durations),
        iops: n as f64 / total.as_secs_f64(),
    }
}
