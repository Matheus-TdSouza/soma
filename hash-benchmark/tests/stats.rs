use hash_benchmark::stats::{percentile, std_dev};
use std::time::Duration;

#[test]
fn percentile_returns_expected_values() {
    let durations: Vec<Duration> = (1..=100)
        .map(Duration::from_millis)
        .collect();
    assert_eq!(percentile(&durations, 50), Duration::from_millis(50));
    assert_eq!(percentile(&durations, 99), Duration::from_millis(99));
    assert_eq!(percentile(&durations, 100), Duration::from_millis(100));
}

#[test]
fn std_dev_returns_sample_standard_deviation() {
    let durations: Vec<Duration> = (1..=5)
        .map(Duration::from_millis)
        .collect();
    let s = std_dev(&durations).as_secs_f64();
    assert!((s - 2.5e-6_f64.sqrt()).abs() < 1e-9);
}

#[test]
fn std_dev_returns_zero_single_sample() {
    let durations = vec![Duration::from_millis(1)];
    assert_eq!(std_dev(&durations), Duration::ZERO);
}
