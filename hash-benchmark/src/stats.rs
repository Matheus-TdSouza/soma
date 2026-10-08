use std::time::Duration;

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

pub fn print_latency_stats(durations: &mut [Duration]) {
    durations.sort();
    let n = durations.len();
    let total = durations.iter().sum::<Duration>();
    let average = total.div_f64(n as f64);
    let iops = n as f64 / total.as_secs_f64();
    println!("p50: {:?}", percentile(durations, 50));
    println!("p90: {:?}", percentile(durations, 90));
    println!("p99: {:?}", percentile(durations, 99));
    println!("Max: {:?}", durations[n - 1]);
    println!("Average: {:?}", average);
    println!("Std dev: {:?}", std_dev(durations));
    println!("IOPS: {:.2}", iops);
}
