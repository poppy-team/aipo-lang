//! Integration coverage for local package resolution in the source commands.

#![forbid(unsafe_code)]

use aipo_package::{Lockfile, PackageId};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const EXIT_SUCCESS: u8 = 0;
const EXIT_LANGUAGE_FAILURE: u8 = 1;
static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "github-http")]
#[test]
fn fetch_github_requires_explicit_cache_and_output_directories() {
    let arguments = vec![
        "package".to_string(),
        "fetch-github".to_string(),
        "acme/packages".to_string(),
        "0123456789abcdef0123456789abcdef01234567".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, 2);
    assert!(String::from_utf8_lossy(&stderr).contains("--cache"));
}

#[test]
fn local_package_default_namespace_is_used_by_all_source_commands() {
    let tree = TempTree::new("default-namespace");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nlet answer = http.double(21)\n",
    );
    write_package(
        &http,
        "acme.http",
        &[],
        "src/main.aipo",
        "export double\nfn double(value)\nreturn value * 2\nend\n",
    );
    let entry = app.join("src/main.aipo");

    assert_success("check", run_command("check", &entry, None));
    assert_success("run", run_command("run", &entry, None));
    assert_success("disasm", run_command("disasm", &entry, None));

    let output = tree.path().join("dist");
    assert_success("build", run_command("build", &entry, Some(&output)));
    for file in ["app.js", "aipo-runtime.js", "app.js.map"] {
        assert!(output.join(file).is_file(), "build emits {file}");
    }
    assert!(
        !app.join("aipo.lock").exists(),
        "resolution does not write a lockfile"
    );
}

#[test]
fn local_package_alias_keeps_the_coordinate_for_resolution() {
    let tree = TempTree::new("alias");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nimport acme.http as h\nlet default_answer = http.double(1)\nlet alias_answer = h.double(2)\n",
    );
    write_package(
        &http,
        "acme.http",
        &[],
        "src/main.aipo",
        "export double\nfn double(value)\nreturn value * 2\nend\n",
    );

    assert_success(
        "check aliased package",
        run_command("check", &app.join("src/main.aipo"), None),
    );
}

#[test]
fn package_diagnostics_keep_the_full_coordinate() {
    let tree = TempTree::new("diagnostic-coordinate");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nlet answer = http.secret()\n",
    );
    write_package(
        &http,
        "acme.http",
        &[],
        "src/main.aipo",
        "fn secret()\nreturn 1\nend\n",
    );

    let (code, _stdout, stderr) = run_command("check", &app.join("src/main.aipo"), None);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("module 'acme.http' does not export 'secret'"),
        "diagnostic retains package provenance: {stderr}"
    );
}

#[test]
fn package_dependencies_recurse_from_each_mapped_entry_directory() {
    let tree = TempTree::new("package-chain");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    let core = tree.path().join("core");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nlet answer = http.triple(4)\n",
    );
    write_package(
        &http,
        "acme.http",
        &[("acme.core", "../core")],
        "src/main.aipo",
        "import acme.core as c\nimport helper\nexport triple\nfn triple(value)\nreturn c.base(value) + helper.offset(0)\nend\n",
    );
    write_file(
        &http.join("src/helper.aipo"),
        "export offset\nfn offset(value)\nreturn value + 1\nend\n",
    );
    write_package(
        &core,
        "acme.core",
        &[],
        "src/main.aipo",
        "export base\nfn base(value)\nreturn value * 3\nend\n",
    );

    let entry = app.join("src/main.aipo");
    assert_success("check package chain", run_command("check", &entry, None));
    assert_success("run package chain", run_command("run", &entry, None));
}

#[test]
fn source_without_manifest_keeps_the_legacy_module_path() {
    let tree = TempTree::new("legacy");
    let entry = tree.path().join("main.aipo");
    write_file(&entry, "import math\nlet answer = math.double(2)\n");
    write_file(
        &tree.path().join("math.aipo"),
        "export double\nfn double(value)\nreturn value * 2\nend\n",
    );

    assert_success("check legacy import", run_command("check", &entry, None));
    assert_success("run legacy import", run_command("run", &entry, None));
}

#[test]
fn package_lock_writes_a_round_trippable_lockfile() {
    let tree = TempTree::new("lock-round-trip");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nlet answer = http.double(2)\n",
    );
    write_package(
        &http,
        "acme.http",
        &[],
        "src/main.aipo",
        "export double\nfn double(value)\nreturn value * 2\nend\n",
    );

    let (code, stdout, stderr) = run_package_command("lock", &app);
    assert_eq!(code, EXIT_SUCCESS, "package lock failed: {stderr}");
    assert!(stdout.contains("locked acme.app"), "{stdout}");

    let lock_path = app.join("aipo.lock");
    let contents = std::fs::read_to_string(&lock_path).expect("lockfile is readable");
    let lockfile = Lockfile::from_str(&contents).expect("lockfile round-trips");
    assert_eq!(lockfile.len(), 2);
    assert!(
        lockfile
            .package(&PackageId::parse("acme.app").expect("valid coordinate"))
            .is_some()
    );
    assert_eq!(
        lockfile.to_toml().expect("lockfile serializes"),
        contents,
        "lockfile is canonical after a TOML round-trip"
    );
    assert_success(
        "check with current lock",
        run_command("check", &app.join("src/main.aipo"), None),
    );
}

#[test]
fn cli_fs_is_denied_without_an_explicit_host_profile() {
    let tree = TempTree::new("fs-denial");
    let entry = tree.path().join("main.aipo");
    write_file(&entry, "let value = fs.read_text(\"config.txt\")\n");

    assert_success("fs module check", run_command("check", &entry, None));
    let (code, _stdout, stderr) = run_command("run", &entry, None);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE, "{stderr}");
    assert!(stderr.contains("AIPO_RT_CAPABILITY_DENIED"), "{stderr}");
}

#[test]
fn package_lock_preserves_declared_env_capability() {
    let tree = TempTree::new("env-capability");
    let app = tree.path().join("app");
    write_file(
        &app.join("aipo.toml"),
        "[package]\nname = \"acme.app\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\ncapabilities = [\"env.read\"]\n",
    );
    write_file(&app.join("src/main.aipo"), "let answer = 1\n");

    assert_success("package lock", run_package_command("lock", &app));
    let contents = std::fs::read_to_string(app.join("aipo.lock")).expect("lockfile is readable");
    let lockfile = Lockfile::from_str(&contents).expect("lockfile parses");
    let package = lockfile
        .package(&PackageId::parse("acme.app").expect("valid coordinate"))
        .expect("root package is locked");
    assert_eq!(package.capabilities, vec!["env.read"]);
}

#[test]
fn cli_env_is_denied_without_an_explicit_host_profile() {
    let tree = TempTree::new("env-denial");
    let entry = tree.path().join("main.aipo");
    write_file(&entry, "let value = env.get(\"AIPO_TEST\")\n");

    assert_success("env module check", run_command("check", &entry, None));
    let (code, _stdout, stderr) = run_command("run", &entry, None);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE, "{stderr}");
    assert!(stderr.contains("AIPO_RT_CAPABILITY_DENIED"), "{stderr}");
}

#[test]
fn package_lock_rejects_remote_dependency_without_explicit_fetch() {
    let tree = TempTree::new("remote-lock-denial");
    let app = tree.path().join("app");
    write_file(
        &app.join("aipo.toml"),
        "[package]\nname = \"acme.app\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n[dependencies]\n\"acme.http\" = { version = \"1.0.0\", type = \"github\", repository = \"acme/packages\", revision = \"0123456789abcdef0123456789abcdef01234567\" }\n",
    );
    write_file(
        &app.join("src/main.aipo"),
        "import acme.http\nlet answer = 1\n",
    );

    let (code, _stdout, stderr) = run_package_command("lock", &app);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE, "{stderr}");
    assert!(stderr.contains("AIPO_PKG_RESOLUTION"), "{stderr}");
    assert!(!app.join("aipo.lock").exists());
}

#[test]
fn package_audit_passes_for_a_current_lock_without_rewriting_it() {
    let tree = TempTree::new("audit");
    let app = tree.path().join("app");
    write_package(&app, "acme.app", &[], "src/main.aipo", "let answer = 1\n");
    let (missing_code, _missing_stdout, missing_stderr) = run_package_command("audit", &app);
    assert_eq!(missing_code, EXIT_LANGUAGE_FAILURE);
    assert!(
        missing_stderr.contains("AIPO_PKG_LOCK_STALE"),
        "{missing_stderr}"
    );

    assert_success("package lock", run_package_command("lock", &app));
    let lock_path = app.join("aipo.lock");
    let before = std::fs::read(&lock_path).expect("lockfile is readable");
    let (code, stdout, stderr) = run_package_command("audit", &app);
    assert_eq!(code, EXIT_SUCCESS, "package audit failed: {stderr}");
    assert!(stdout.contains("audit passed"), "{stdout}");
    assert_eq!(
        before,
        std::fs::read(&lock_path).expect("lockfile remains readable")
    );
}

#[test]
fn source_command_rejects_a_stale_lock_before_compiling() {
    let tree = TempTree::new("stale-lock");
    let app = tree.path().join("app");
    let http = tree.path().join("http");
    write_package(
        &app,
        "acme.app",
        &[("acme.http", "../http")],
        "src/main.aipo",
        "import acme.http\nlet answer = http.double(2)\n",
    );
    write_package(
        &http,
        "acme.http",
        &[],
        "src/main.aipo",
        "export double\nfn double(value)\nreturn value * 2\nend\n",
    );
    assert_success("initial package lock", run_package_command("lock", &app));

    write_file(
        &http.join("src/main.aipo"),
        "export double\nfn double(value)\nreturn value * 3\nend\n",
    );
    let (code, _stdout, stderr) = run_command("check", &app.join("src/main.aipo"), None);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE, "{stderr}");
    assert!(stderr.contains("AIPO_PKG_LOCK_STALE"), "{stderr}");
    assert!(
        !stderr.contains("AIPO_SEM_"),
        "stale lock is not a semantic diagnostic"
    );
}

#[test]
fn source_command_does_not_create_a_lockfile() {
    let tree = TempTree::new("no-write");
    let app = tree.path().join("app");
    write_package(&app, "acme.app", &[], "src/main.aipo", "let answer = 1\n");

    assert_success(
        "check without lock",
        run_command("check", &app.join("src/main.aipo"), None),
    );
    assert!(!app.join("aipo.lock").exists());
}

#[test]
fn invalid_lock_is_rejected_by_source_and_audit_commands() {
    let tree = TempTree::new("invalid-lock");
    let app = tree.path().join("app");
    write_package(&app, "acme.app", &[], "src/main.aipo", "let answer = 1\n");
    write_file(&app.join("aipo.lock"), "not a lockfile\n");

    let (source_code, _source_stdout, source_stderr) =
        run_command("check", &app.join("src/main.aipo"), None);
    assert_eq!(source_code, EXIT_LANGUAGE_FAILURE, "{source_stderr}");
    assert!(
        source_stderr.contains("AIPO_PKG_LOCK_STALE"),
        "{source_stderr}"
    );

    let (audit_code, _audit_stdout, audit_stderr) = run_package_command("audit", &app);
    assert_eq!(audit_code, EXIT_LANGUAGE_FAILURE, "{audit_stderr}");
    assert!(
        audit_stderr.contains("AIPO_PKG_LOCK_STALE"),
        "{audit_stderr}"
    );
}

#[test]
fn manifest_resolution_failures_use_package_resolution_code() {
    let tree = TempTree::new("manifest-resolution");
    let app = tree.path().join("app");
    write_file(
        &app.join("aipo.toml"),
        "name = \"acme.app\"\nversion = \"1.0.0\"\nentry = \"../outside.aipo\"\n",
    );
    write_file(&app.join("src/main.aipo"), "let answer = 1\n");

    let (code, _stdout, stderr) = run_command("check", &app.join("src/main.aipo"), None);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE, "{stderr}");
    assert!(stderr.contains("AIPO_PKG_RESOLUTION"), "{stderr}");
    assert!(
        !stderr.contains("AIPO_SEM_"),
        "manifest failure is not semantic"
    );
}

fn assert_success(label: &str, result: (u8, String, String)) {
    let (code, _stdout, stderr) = result;
    assert_eq!(code, EXIT_SUCCESS, "{label} failed: {stderr}");
    assert!(stderr.is_empty(), "{label} emitted diagnostics: {stderr}");
}

fn run_command(command: &str, entry: &Path, output: Option<&Path>) -> (u8, String, String) {
    let mut arguments = vec![command.to_string(), entry.to_string_lossy().into_owned()];
    if let Some(output) = output {
        arguments.push("--out".to_string());
        arguments.push(output.to_string_lossy().into_owned());
    }
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn run_package_command(operation: &str, package_dir: &Path) -> (u8, String, String) {
    let arguments = vec![
        "package".to_string(),
        operation.to_string(),
        package_dir.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    (
        code,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn write_package(
    directory: &Path,
    name: &str,
    dependencies: &[(&str, &str)],
    entry: &str,
    source: &str,
) {
    let mut manifest = format!("name = \"{name}\"\nversion = \"1.0.0\"\nentry = \"{entry}\"\n");
    if !dependencies.is_empty() {
        manifest.push_str("\n[dependencies]\n");
        for (coordinate, path) in dependencies {
            manifest.push_str(&format!("\"{coordinate}\" = {{ path = \"{path}\" }}\n"));
        }
    }
    write_file(&directory.join("aipo.toml"), &manifest);
    write_file(&directory.join(entry), source);
}

fn write_file(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("fixture path has a parent"))
        .expect("fixture directory creates");
    std::fs::write(path, contents).expect("fixture file writes");
}

struct TempTree {
    path: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "aipo-cli-package-test-{}-{label}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temporary package tree creates");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
