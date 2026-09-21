//! JavaScript backend runner: emit a bundle and execute it under Node.

use std::path::{Path, PathBuf};
use std::process::Command;

/// An emitted bundle on disk.
pub struct JsBundleDir {
    /// Directory holding `app.js`, `aipo-runtime.js` and `app.js.map`.
    pub dir: PathBuf,
}

/// Emits `text` (already checked) to a fresh directory under the system temp dir.
pub fn emit_bundle(
    file_name: &str,
    text: &str,
    module: &aipo_ir::CoreModule,
    tag: &str,
) -> JsBundleDir {
    let bundle = aipo_js::emit_js(file_name, text, module);
    let dir = std::env::temp_dir().join(format!("aipo-testkit-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("bundle dir is writable");
    std::fs::write(dir.join("app.js"), &bundle.app_js).expect("app.js is writable");
    std::fs::write(dir.join("aipo-runtime.js"), &bundle.runtime_js).expect("shim is writable");
    std::fs::write(dir.join("app.js.map"), &bundle.source_map).expect("map is writable");
    JsBundleDir { dir }
}

/// Runs `node app.js` in `dir`, returning `(exit_code, stdout, stderr)`.
///
/// Panics when `node` itself cannot start (requires Node >= 20); a failing
/// *program* is reported through its exit code, never as a harness error.
pub fn run_node(dir: &Path) -> (Option<i32>, String, String) {
    let output = Command::new("node")
        .arg("app.js")
        .current_dir(dir)
        .output()
        .expect("node starts (requires Node >= 20)");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Human-readable Node version, for evidence records.
#[must_use]
pub fn node_version() -> String {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|_| "node-not-found".to_string())
}

/// One decoded mapping segment: generated line → (source index, source line,
/// source column), per the ECMA-426 VLQ encoding.
pub struct Mapping {
    /// Zero-based generated line.
    pub generated_line: usize,
    /// Index into `sources`.
    pub source: u32,
    /// Zero-based original line.
    pub source_line: u32,
    /// Zero-based original column.
    pub source_column: u32,
}

/// Decodes the `mappings` field of a source map into per-line segments.
///
/// Only the first segment of each generated line is decoded: the Aipo emitter
/// maps whole lines (column granularity is a documented non-goal).
pub fn decode_mappings(mappings: &str) -> Vec<Mapping> {
    let mut out = Vec::new();
    for (generated_line, line) in mappings.split(';').enumerate() {
        if line.is_empty() {
            continue;
        }
        let bytes = line.as_bytes();
        let mut used = 0usize;
        let _gen_col = decode_vlq(bytes, &mut used);
        let source = decode_vlq(bytes, &mut used) as u32;
        let source_line = decode_vlq(bytes, &mut used) as u32;
        let source_column = decode_vlq(bytes, &mut used) as u32;
        out.push(Mapping {
            generated_line,
            source,
            source_line,
            source_column,
        });
    }
    out
}

fn decode_vlq(bytes: &[u8], used: &mut usize) -> i64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = 0i64;
    let mut shift = 0u32;
    loop {
        let byte = bytes.get(*used).copied().unwrap_or(b'A');
        *used += 1;
        let digit = ALPHABET.iter().position(|&c| c == byte).unwrap_or(0) as i64;
        result |= (digit & 31) << shift;
        shift += 5;
        if digit & 32 == 0 {
            break;
        }
    }
    if result & 1 == 1 {
        -(result >> 1)
    } else {
        result >> 1
    }
}
