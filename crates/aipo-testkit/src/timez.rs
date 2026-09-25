//! Minimal timing helper for benchmarks: warmup, repeated samples, and a
//! median/MAD summary that is robust without external statistics crates.

use std::time::{Duration, Instant};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Sample {
    pub name: String,
    pub input: String,
    pub median: Duration,
    pub mad: Duration,
    pub samples: usize,
    pub bytes_per_sec: Option<f64>,
    pub raw_samples: Vec<Duration>,
    pub setup_median: Option<Duration>,
    pub execution_median: Option<Duration>,
    pub raw_setup_samples: Vec<Duration>,
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
    let (median, mad) = summarize(&times);
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
        raw_samples: times,
        setup_median: None,
        execution_median: None,
        raw_setup_samples: Vec::new(),
    }
}

#[allow(missing_docs)]
pub fn measure_split<F>(
    name: &str,
    input: &str,
    input_bytes: Option<u64>,
    samples: usize,
    mut work: F,
) -> Sample
where
    F: FnMut() -> (Duration, Duration),
{
    work();
    let mut total_samples = Vec::with_capacity(samples);
    let mut setup_samples = Vec::with_capacity(samples);
    let mut execution_samples = Vec::with_capacity(samples);
    for _ in 0..samples {
        let (setup, execution) = work();
        total_samples.push(setup + execution);
        setup_samples.push(setup);
        execution_samples.push(execution);
    }
    let (median, mad) = summarize(&total_samples);
    let (setup_median, _) = summarize(&setup_samples);
    let (execution_median, _) = summarize(&execution_samples);
    let bytes_per_sec = input_bytes.map(|bytes| {
        let seconds = execution_median.as_secs_f64().max(f64::MIN_POSITIVE);
        bytes as f64 / seconds
    });
    Sample {
        name: name.to_string(),
        input: input.to_string(),
        median,
        mad,
        samples,
        bytes_per_sec,
        raw_samples: total_samples,
        setup_median: Some(setup_median),
        execution_median: Some(execution_median),
        raw_setup_samples: setup_samples,
    }
}

fn summarize(samples: &[Duration]) -> (Duration, Duration) {
    let mut sorted = samples.to_vec();
    sorted.sort();
    let median = sorted[sorted.len() / 2];
    let mut deviations: Vec<Duration> = sorted
        .iter()
        .map(|sample| sample.abs_diff(median))
        .collect();
    deviations.sort();
    (median, deviations[deviations.len() / 2])
}
