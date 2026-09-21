//! Aipo benchmark runner: establishes reproducible baselines, not gates.
//!
//! Wall-clock timing on shared runners is noisy, so this runner never
//! passes/fails: it prints median/MAD samples plus scaling ratios, and writes
//! a JSON baseline for later comparison on a dedicated runner. CI compiles
//! this crate; temporal regression gates stay manual until a stable
//! environment exists (see `docs/performance/tiers.md`).
//!
//! Usage: `cargo run --release -p aipo-bench [--quick] [--json <path>]`.

use aipo_testkit::timez::{self, Sample};

mod workloads;

fn samples(quick: bool) -> usize {
    if quick { 3 } else { 7 }
}

fn print_sample(sample: &Sample) {
    let throughput = sample
        .bytes_per_sec
        .map(|bytes| format!(" | {:>10.0} B/s", bytes))
        .unwrap_or_default();
    println!(
        "  {:<46} {:>12} input={:<10} mad={:?} n={}{}",
        sample.name,
        format!("{:?}", sample.median),
        sample.input,
        sample.mad,
        sample.samples,
        throughput
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.iter().any(|arg| arg == "--quick");
    let json_path = args
        .iter()
        .position(|arg| arg == "--json")
        .and_then(|at| args.get(at + 1))
        .cloned();
    let rounds = samples(quick);

    println!("aipo-bench (rounds={rounds})");
    println!(
        "rustc {} | {}",
        rustc_version_string(),
        std::env::consts::OS
    );
    let mut samples = Vec::new();

    println!("== frontend stages ==");
    for sample in workloads::frontend(rounds, quick) {
        print_sample(&sample);
        samples.push(sample);
    }
    println!("== end-to-end commands ==");
    for sample in workloads::commands(rounds, quick) {
        print_sample(&sample);
        samples.push(sample);
    }
    println!("== VM workloads (execution only) ==");
    for sample in workloads::vm_workloads(rounds, quick) {
        print_sample(&sample);
        samples.push(sample);
    }
    println!("== JS backend (frontend + emit + node) ==");
    for sample in workloads::js_workloads(rounds, quick) {
        print_sample(&sample);
        samples.push(sample);
    }
    println!("== scaling (list build/iterate, N..8N) ==");
    for line in workloads::scaling(rounds, quick) {
        println!("  {line}");
    }

    if let Some(path) = json_path {
        let payload: Vec<serde_json_like::Entry> = samples
            .iter()
            .map(|sample| serde_json_like::Entry {
                name: sample.name.clone(),
                input: sample.input.clone(),
                median_ns: sample.median.as_nanos(),
                mad_ns: sample.mad.as_nanos(),
                samples: sample.samples,
                bytes_per_sec: sample.bytes_per_sec,
            })
            .collect();
        std::fs::write(&path, serde_json_like::render(&payload)).expect("baseline is writable");
        println!("baseline written to {path}");
    }
    let _ = timez::measure("warmup-check", "0", None, 1, || {});
}

fn rustc_version_string() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Minimal JSON rendering without a serialization dependency.
mod serde_json_like {
    pub(crate) struct Entry {
        pub name: String,
        pub input: String,
        pub median_ns: u128,
        pub mad_ns: u128,
        pub samples: usize,
        pub bytes_per_sec: Option<f64>,
    }

    fn escape(text: &str) -> String {
        text.replace('\\', "\\\\").replace('"', "\\\"")
    }

    pub(crate) fn render(entries: &[Entry]) -> String {
        let mut out = String::from("[\n");
        for (index, entry) in entries.iter().enumerate() {
            out.push_str(&format!(
                "  {{\"name\": \"{}\", \"input\": \"{}\", \"median_ns\": {}, \"mad_ns\": {}, \"samples\": {}, \"bytes_per_sec\": {}}}",
                escape(&entry.name),
                escape(&entry.input),
                entry.median_ns,
                entry.mad_ns,
                entry.samples,
                entry
                    .bytes_per_sec
                    .map_or("null".to_string(), |v| format!("{v:.1}")),
            ));
            if index + 1 < entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("]\n");
        out
    }
}
