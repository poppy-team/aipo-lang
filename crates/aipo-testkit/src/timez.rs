//! Minimal timing helper for benchmarks: warmup, repeated samples, and a
//! median/MAD summary that is robust without external statistics crates.

use std::time::{Duration, Instant};

/// Summary of one measured workload.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Workload name.
    pub name: String,
    /// Input size descriptor (for example `"10 KiB"` or `"N=400"`).
    pub input: String,
    /// Median wall time over samples.
    pub median: Duration,
    /// Median absolute deviation, the reported noise figure.
    pub mad: Duration,
    /// Samples taken.
    pub samples: usize,
    /// Throughput in input bytes per second, when the input is byte-sized.
    pub bytes_per_sec: Option<f64>,
}

/// Times `work` after one warmup call, returning the [`Sample`].
pub fn measure<F>(
    name: &str,
    input: &str,
    input_bytes: Option<u64>,
    samples: usize,
    mut work: F,
) -> Sample
where
    F: FnMut(),
{
    work();
    let mut times: Vec<Duration> = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        work();
        times.push(start.elapsed());
    }
    times.sort();
    let median = times[times.len() / 2];
    let mut deviations: Vec<Duration> = times.iter().map(|t| t.abs_diff(median)).collect();
    deviations.sort();
    let mad = deviations[deviations.len() / 2];
    let bytes_per_sec = input_bytes.map(|bytes| {
        let seconds = median.as_secs_f64().max(f64::MIN_POSITIVE);
        bytes as f64 / seconds
    });
    Sample {
        name: name.to_string(),
        input: input.to_string(),
        median,
        mad,
        samples,
        bytes_per_sec,
    }
}
