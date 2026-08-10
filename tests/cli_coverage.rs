use std::process::Command;

/// `spectrum coverage --profile <p>`, run against the built binary. Every
/// invocation here reads a real profile file and captures real stdout, so
/// this exercises the same path a human running the CLI does — not the
/// library function `coverage` calls internally.
fn run(profile: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spectrum"))
        .arg("coverage")
        .arg("--profile")
        .arg(profile)
        .output()
        .expect("spectrum binary runs")
}

fn write(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn runtime_with_context_warns_and_lists_its_coverage() {
    let out = run(std::path::Path::new("profiles/runtime-with-context.toml"));
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("PRIVACY"));
    // A string unique to this profile's `coverage`, proving the list itself
    // reaches the user and not just the warning.
    assert!(stdout.contains("token-level provenance within a retrieved chunk"));
}

#[test]
fn build_manifest_does_not_warn_and_lists_its_coverage() {
    let out = run(std::path::Path::new("profiles/build-manifest.toml"));
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("PRIVACY"));
    assert!(stdout.contains("sub-agents spawned during the task"));
}

#[test]
fn runtime_tools_does_not_warn_and_lists_its_coverage() {
    let out = run(std::path::Path::new("profiles/runtime-tools.toml"));
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("PRIVACY"));
    assert!(stdout.contains("whether a validly sourced component is adversarial"));
}

#[test]
fn any_profile_that_includes_retrieved_chunk_warns_whatever_its_id() {
    // The case the old `if p.id == "runtime-with-context"` check got wrong:
    // the leak comes from the *property* (digesting retrieved content), not
    // from a specific profile name. A differently named profile that
    // includes `retrieved_chunk` must still warn.
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "custom.toml",
        r#"
id = "some-other-profile"
kinds = ["model", "retrieved_chunk"]
coverage = ["everything this made-up profile does not claim to capture"]
"#,
    );
    let out = run(&p);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("PRIVACY"));
    assert!(stdout.contains("everything this made-up profile does not claim to capture"));
}
