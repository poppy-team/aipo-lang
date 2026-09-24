//! Integration coverage for local package resolution in the source commands.

#![forbid(unsafe_code)]

use aipo_package::{
    GitHubArtifact, GitHubCache, Lockfile, PackageId, PackageInput, PackageSource,
    resolve_cached_github_graph,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const EXIT_SUCCESS: u8 = 0;
const EXIT_LANGUAGE_FAILURE: u8 = 1;
static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

#[test]
fn mixed_lock_flags_are_validated_at_the_cli_boundary() {
    let base = vec!["package".to_string(), "lock".to_string(), ".".to_string()];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut missing_cache = base.clone();
    missing_cache.push("--fetch-github".to_string());
    stdout.clear();
    stderr.clear();
    assert_eq!(
        aipo_cli::run_with(&missing_cache, &mut stdout, &mut stderr),
        2
    );
    assert!(String::from_utf8_lossy(&stderr).contains("must be provided together"));

    let mut unexpected_flag = base.clone();
    unexpected_flag.push("--cache".to_string());
    unexpected_flag.push(".cache".to_string());
    stdout.clear();
    stderr.clear();
    assert_eq!(
        aipo_cli::run_with(&unexpected_flag, &mut stdout, &mut stderr),
        2
    );
    assert!(String::from_utf8_lossy(&stderr).contains("must be provided together"));

    let audit_flag = vec![
        "package".to_string(),
        "audit".to_string(),
        ".".to_string(),
        "--fetch-github".to_string(),
        "--cache".to_string(),
        ".cache".to_string(),
    ];
    stdout.clear();
    stderr.clear();
    assert_eq!(aipo_cli::run_with(&audit_flag, &mut stdout, &mut stderr), 2);
    assert!(String::from_utf8_lossy(&stderr).contains("only valid"));
}

#[cfg(not(feature = "github-http"))]
#[test]
fn mixed_lock_fetch_requires_the_http_feature() {
    let arguments = vec![
        "package".to_string(),
        "lock".to_string(),
        ".".to_string(),
        "--fetch-github".to_string(),
        "--cache".to_string(),
        ".cache".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, 2);
    assert!(String::from_utf8_lossy(&stderr).contains("github-http"));
}

#[cfg(feature = "github-http")]
#[test]
fn github_token_flag_rejects_invalid_environment_names() {
    for flag in ["--github-token-env", "--github-token-env=bad-name"] {
        let mut arguments = vec![
            "package".to_string(),
            "lock".to_string(),
            ".".to_string(),
            "--fetch-github".to_string(),
            "--cache".to_string(),
            ".cache".to_string(),
            flag.to_string(),
        ];
        if flag == "--github-token-env" {
            arguments.push("bad-name".to_string());
        }
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(aipo_cli::run_with(&arguments, &mut stdout, &mut stderr), 2);
        assert!(String::from_utf8_lossy(&stderr).contains("valid variable name"));
    }
}

#[cfg(feature = "github-http")]
#[test]
fn github_token_env_is_explicit_for_direct_fetches() {
    let name = format!(
        "AIPO_CLI_TEST_MISSING_GITHUB_TOKEN_{}_{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    );
    assert!(std::env::var_os(&name).is_none());
    let arguments = vec![
        "package".to_string(),
        "fetch-github".to_string(),
        "acme/packages".to_string(),
        "0123456789abcdef0123456789abcdef01234567".to_string(),
        "--cache".to_string(),
        ".cache".to_string(),
        "--out".to_string(),
        ".out".to_string(),
        "--github-token-env".to_string(),
        name.clone(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    let stderr = String::from_utf8_lossy(&stderr);
    assert!(stderr.contains("AIPO_PKG_FETCH"));
    assert!(stderr.contains(&name));
    assert!(stderr.contains("is not set"));
}

#[cfg(feature = "github-http")]
#[test]
fn github_token_env_is_explicit_for_mixed_locks() {
    let tree = TempTree::new("missing-token");
    let package = tree.path().join("app");
    write_package(
        &package,
        "acme.app",
        &[],
        "src/main.aipo",
        "export value\nfn value()\nreturn 1\nend\n",
    );
    let name = format!(
        "AIPO_CLI_TEST_MISSING_GITHUB_TOKEN_{}_{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    );
    assert!(std::env::var_os(&name).is_none());
    let arguments = vec![
        "package".to_string(),
        "lock".to_string(),
        package.to_string_lossy().into_owned(),
        "--fetch-github".to_string(),
        "--cache".to_string(),
        tree.path().join("cache").to_string_lossy().into_owned(),
        "--github-token-env".to_string(),
        name.clone(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    let stderr = String::from_utf8_lossy(&stderr);
    assert!(stderr.contains("AIPO_PKG_FETCH"));
    assert!(stderr.contains(&name));
    assert!(stderr.contains("is not set"));
}

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

#[cfg(feature = "github-http")]
#[test]
fn mixed_local_root_lock_and_offline_source_commands_work() {
    let tree = TempTree::new("mixed-local-remote");
    let app = tree.path().join("app");
    let local = tree.path().join("local");
    let entry = app.join("src/main.aipo");
    write_package(
        &local,
        "acme.local",
        &[],
        "src/main.aipo",
        "export local_answer\nfn local_answer()\nreturn 1\nend\n",
    );
    let revision = "0123456789abcdef0123456789abcdef01234567";
    let remote_source = PackageSource::github_at("acme/packages", revision, "packages/remote")
        .expect("remote source");
    write_file(
        &app.join("aipo.toml"),
        &format!(
            "[package]\nname = \"acme.app\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n[dependencies]\n\"acme.local\" = {{ path = \"../local\" }}\n\"acme.remote\" = {{ version = \"1.0.0\", type = \"github\", repository = \"acme/packages\", revision = \"{revision}\", subpath = \"packages/remote\" }}\n"
        ),
    );
    write_file(
        &entry,
        "import acme.local\nimport acme.remote\nlet result = local.local_answer() + remote.remote_answer()\n",
    );
    let cache_path = tree.path().join("cache");
    let cache = GitHubCache::new(&cache_path);
    cache
        .store(
            &remote_source,
            GitHubArtifact::new(
                b"[package]\nname = \"acme.remote\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n",
                b"export remote_answer\nfn remote_answer()\nreturn 42\nend\n",
            ),
        )
        .expect("remote dependency stores in cache");

    let lock_arguments = vec![
        "package".to_string(),
        "lock".to_string(),
        app.to_string_lossy().into_owned(),
        "--fetch-github".to_string(),
        "--cache".to_string(),
        cache_path.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&lock_arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_SUCCESS, "mixed lock failed: {stderr:?}");
    let lock = std::fs::read_to_string(app.join("aipo.lock")).expect("mixed lock reads");
    assert!(lock.contains("acme.remote"));
    assert!(lock.contains("type = \"github\""));

    for command in ["check", "run", "disasm"] {
        let arguments = vec![
            command.to_string(),
            entry.to_string_lossy().into_owned(),
            "--package-cache".to_string(),
            cache_path.to_string_lossy().into_owned(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
        assert_eq!(code, EXIT_SUCCESS, "mixed {command} failed: {stderr:?}");
    }

    let output = tree.path().join("dist");
    let build_arguments = vec![
        "build".to_string(),
        entry.to_string_lossy().into_owned(),
        "--out".to_string(),
        output.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache_path.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&build_arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_SUCCESS, "mixed build failed: {stderr:?}");
    assert!(output.join("app.js").is_file());
}

#[test]
fn source_commands_consume_cached_remote_snapshot_without_http_feature() {
    let tree = TempTree::new("cached-remote-commands");
    let (entry, cache) = write_cached_remote_snapshot(&tree);

    for command in ["check", "run", "disasm"] {
        let arguments = vec![
            command.to_string(),
            entry.to_string_lossy().into_owned(),
            "--package-cache".to_string(),
            cache.to_string_lossy().into_owned(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
        assert_eq!(code, EXIT_SUCCESS, "{command} failed: {stderr:?}");
    }

    let output = tree.path().join("dist");
    let arguments = vec![
        "build".to_string(),
        entry.to_string_lossy().into_owned(),
        "--out".to_string(),
        output.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_SUCCESS, "build failed: {stderr:?}");
    assert!(output.join("app.js").is_file());
}

#[test]
fn cached_remote_commands_fail_without_fetching_when_cache_entry_is_missing() {
    let tree = TempTree::new("cached-remote-missing");
    let (entry, cache) = write_cached_remote_snapshot(&tree);
    std::fs::remove_dir_all(&cache).expect("cache removal");

    let arguments = vec![
        "check".to_string(),
        entry.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(String::from_utf8_lossy(&stderr).contains("AIPO_PKG_FETCH"));
    assert!(
        !cache.exists(),
        "cache reads must not recreate a missing root"
    );
}

#[test]
fn cached_remote_commands_reject_a_stale_lock_without_network() {
    let tree = TempTree::new("cached-remote-stale-lock");
    let (entry, cache) = write_cached_remote_snapshot(&tree);
    let lock_path = entry
        .parent()
        .expect("entry parent")
        .parent()
        .expect("package root")
        .join("aipo.lock");
    let lock = std::fs::read_to_string(&lock_path).expect("lockfile reads");
    let stale_lock = lock.replacen("digest = \"", "digest = \"0", 1);
    std::fs::write(&lock_path, stale_lock).expect("stale lock writes");

    let arguments = vec![
        "check".to_string(),
        entry.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(String::from_utf8_lossy(&stderr).contains("AIPO_PKG_LOCK_STALE"));
}

#[test]
fn package_cache_flag_requires_a_value() {
    let tree = TempTree::new("cache-flag-usage");
    let entry = tree.path().join("main.aipo");
    write_file(&entry, "let answer = 1\n");
    let arguments = vec![
        "check".to_string(),
        entry.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = aipo_cli::run_with(&arguments, &mut stdout, &mut stderr);
    assert_eq!(code, 2);
    assert!(String::from_utf8_lossy(&stderr).contains("--package-cache requires"));
}

#[test]
fn cache_prune_dry_run_and_apply_only_remove_unreferenced_entries() {
    let tree = TempTree::new("cache-prune");
    let (entry, cache) = write_cached_remote_snapshot(&tree);
    let lock_path = entry
        .parent()
        .expect("entry parent")
        .parent()
        .expect("package root")
        .join("aipo.lock");
    let candidate_source = PackageSource::github_at(
        "acme/other",
        "0123456789abcdef0123456789abcdef01234567",
        "packages/other",
    )
    .expect("candidate source");
    GitHubCache::new(&cache)
        .store(
            &candidate_source,
            GitHubArtifact::new(
                b"[package]\nname = \"acme.other\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n",
                b"export answer\nfn answer()\nreturn 7\nend\n",
            ),
        )
        .expect("candidate stores");
    assert_eq!(cache_entry_count(&cache), 2);

    let (code, stdout, stderr) = run_cache_prune(&cache, &lock_path, false);
    assert_eq!(code, EXIT_SUCCESS, "{stderr}");
    assert!(stdout.contains(&candidate_source.github_source().expect("source").label()));
    assert!(stdout.contains("dry-run"));
    assert_eq!(cache_entry_count(&cache), 2);

    let (code, stdout, stderr) = run_cache_prune(&cache, &lock_path, true);
    assert_eq!(code, EXIT_SUCCESS, "{stderr}");
    assert!(stdout.contains("removed 1 candidates"));
    assert_eq!(cache_entry_count(&cache), 1);
}

#[test]
fn cache_prune_refuses_corrupt_entries_without_deleting() {
    let tree = TempTree::new("cache-prune-corrupt");
    let (entry, cache) = write_cached_remote_snapshot(&tree);
    let lock_path = entry
        .parent()
        .expect("entry parent")
        .parent()
        .expect("package root")
        .join("aipo.lock");
    let candidate_source = PackageSource::github_at(
        "acme/other",
        "0123456789abcdef0123456789abcdef01234567",
        "packages/other",
    )
    .expect("candidate source");
    let cache_handle = GitHubCache::new(&cache);
    cache_handle
        .store(
            &candidate_source,
            GitHubArtifact::new(
                b"[package]\nname = \"acme.other\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n",
                b"export answer\nfn answer()\nreturn 7\nend\n",
            ),
        )
        .expect("candidate stores");
    let candidate_dir = cache_entry_dir_for_source(&cache, &candidate_source);
    std::fs::write(candidate_dir.join("entry.bin"), b"corrupt").expect("corrupt entry writes");

    let (code, _stdout, stderr) = run_cache_prune(&cache, &lock_path, true);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(stderr.contains("AIPO_PKG_FETCH"), "{stderr}");
    assert_eq!(cache_entry_count(&cache), 2);
}

#[test]
fn cache_prune_requires_an_existing_lockfile() {
    let tree = TempTree::new("cache-prune-missing-lock");
    let (_entry, cache) = write_cached_remote_snapshot(&tree);
    let missing_lock = tree.path().join("missing.lock");
    let (code, _stdout, stderr) = run_cache_prune(&cache, &missing_lock, true);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(stderr.contains("AIPO_PKG_LOCK_STALE"), "{stderr}");
    assert_eq!(cache_entry_count(&cache), 1);
}

#[test]
fn cache_verify_is_read_only_and_reports_entries() {
    let tree = TempTree::new("cache-verify-valid");
    let (_entry, cache) = write_cached_remote_snapshot(&tree);
    let metadata_path = cache_entry_dir(&cache).join("metadata.json");
    let before = std::fs::read(&metadata_path).expect("metadata reads");

    let (code, stdout, stderr) = run_cache_verify(&cache);
    assert_eq!(code, EXIT_SUCCESS, "{stderr}");
    assert!(stdout.contains("cache verified: 1 entries"));
    assert!(stderr.is_empty());
    assert_eq!(
        std::fs::read(&metadata_path).expect("metadata reads after verify"),
        before
    );
}

#[test]
fn cache_verify_missing_root_succeeds_without_creation() {
    let tree = TempTree::new("cache-verify-missing");
    let cache = tree.path().join("missing-cache");
    let (code, stdout, stderr) = run_cache_verify(&cache);
    assert_eq!(code, EXIT_SUCCESS, "{stderr}");
    assert!(stdout.contains("cache verified: 0 entries"));
    assert!(stderr.is_empty());
    assert!(!cache.exists());
}

#[test]
fn cache_verify_reports_corruption_without_repairing() {
    let tree = TempTree::new("cache-verify-corrupt");
    let (_entry, cache) = write_cached_remote_snapshot(&tree);
    let entry_path = cache_entry_dir(&cache).join("entry.bin");
    std::fs::write(&entry_path, b"corrupt").expect("corrupt entry writes");

    let (code, _stdout, stderr) = run_cache_verify(&cache);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(stderr.contains("AIPO_PKG_FETCH"), "{stderr}");
    assert_eq!(
        std::fs::read(&entry_path).expect("corrupt entry remains"),
        b"corrupt"
    );
}

#[test]
fn cache_verify_cli_arguments_are_strict() {
    let tree = TempTree::new("cache-verify-arguments");
    let cache = tree.path().join("cache");
    for arguments in [
        vec!["package".to_string(), "cache".to_string()],
        vec![
            "package".to_string(),
            "cache".to_string(),
            "clean".to_string(),
            cache.to_string_lossy().into_owned(),
        ],
        vec![
            "package".to_string(),
            "cache".to_string(),
            "verify".to_string(),
        ],
    ] {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(aipo_cli::run_with(&arguments, &mut stdout, &mut stderr), 2);
        assert!(!stderr.is_empty());
    }
}

#[test]
fn package_cache_flag_accepts_equals_form_and_rejects_duplicates() {
    let tree = TempTree::new("cache-flag-forms");
    let (entry, cache) = write_cached_remote_snapshot(&tree);
    let valid = vec![
        "check".to_string(),
        entry.to_string_lossy().into_owned(),
        format!("--package-cache={}", cache.display()),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        aipo_cli::run_with(&valid, &mut stdout, &mut stderr),
        EXIT_SUCCESS
    );

    let duplicate = vec![
        "check".to_string(),
        entry.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache.to_string_lossy().into_owned(),
        "--package-cache".to_string(),
        cache.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(aipo_cli::run_with(&duplicate, &mut stdout, &mut stderr), 2);
    assert!(String::from_utf8_lossy(&stderr).contains("provided more than once"));
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

fn run_cache_verify(cache_dir: &Path) -> (u8, String, String) {
    let arguments = vec![
        "package".to_string(),
        "cache".to_string(),
        "verify".to_string(),
        cache_dir.to_string_lossy().into_owned(),
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

fn run_cache_prune(cache_dir: &Path, lock_path: &Path, apply: bool) -> (u8, String, String) {
    let mut arguments = vec![
        "package".to_string(),
        "cache".to_string(),
        "prune".to_string(),
        cache_dir.to_string_lossy().into_owned(),
        "--lock".to_string(),
        lock_path.to_string_lossy().into_owned(),
    ];
    if apply {
        arguments.push("--apply".to_string());
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

fn cache_entry_dir(cache_dir: &Path) -> PathBuf {
    std::fs::read_dir(cache_dir.join("github-v1"))
        .expect("cache namespace reads")
        .next()
        .expect("cache entry exists")
        .expect("cache entry reads")
        .path()
}

fn cache_entry_count(cache_dir: &Path) -> usize {
    std::fs::read_dir(cache_dir.join("github-v1"))
        .expect("cache namespace reads")
        .count()
}

fn cache_entry_dir_for_source(cache_dir: &Path, source: &PackageSource) -> PathBuf {
    let label = source.github_source().expect("GitHub source").label();
    std::fs::read_dir(cache_dir.join("github-v1"))
        .expect("cache namespace reads")
        .map(|entry| entry.expect("cache entry reads").path())
        .find(|path| {
            std::fs::read_to_string(path.join("metadata.json"))
                .is_ok_and(|metadata| metadata.contains(&label))
        })
        .expect("cache entry for source")
}

fn write_cached_remote_snapshot(tree: &TempTree) -> (PathBuf, PathBuf) {
    let revision = "0123456789abcdef0123456789abcdef01234567";
    let root_source = PackageSource::github_at("acme/root", revision, ".").expect("root source");
    let dependency_source = PackageSource::github_at("acme/packages", revision, "packages/http")
        .expect("dependency source");
    let root = tree.path().join("fetched");
    let entry_path = root.join("src/main.aipo");
    let manifest = format!(
        "[package]\nname = \"acme.app\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n[dependencies]\n\"acme.http\" = {{ version = \"1.0.0\", type = \"github\", repository = \"acme/packages\", revision = \"{revision}\", subpath = \"packages/http\" }}\n"
    );
    write_file(&root.join("aipo.toml"), &manifest);
    write_file(
        &entry_path,
        "import acme.http\nlet result = http.answer()\n",
    );
    let entry_bytes = std::fs::read(&entry_path).expect("root entry reads");
    let cache_path = tree.path().join("cache");
    let cache = GitHubCache::new(&cache_path);
    cache
        .store(
            &dependency_source,
            GitHubArtifact::new(
                b"[package]\nname = \"acme.http\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n",
                b"export answer\nfn answer()\nreturn 42\nend\n",
            ),
        )
        .expect("dependency cache stores");
    let root_input =
        PackageInput::from_manifest_bytes(manifest.as_bytes(), entry_bytes, root_source)
            .expect("root input parses");
    let resolved = resolve_cached_github_graph(root_input, &entry_path, &cache)
        .expect("cached graph resolves");
    let lock = resolved
        .graph
        .to_lockfile()
        .expect("graph lock builds")
        .to_toml()
        .expect("graph lock serializes");
    write_file(&root.join("aipo.lock"), &lock);
    (entry_path, cache_path)
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
