use std::process::Command;

/// `spectrum compose --profile <p> --bom <b>`, run against the built binary.
///
/// The design's fail-loud table says `compose` should refuse when a declared
/// kind has no collector. It warns instead — four of the eight kinds have no
/// collector until step 5, so refusing would make step 1 unusable. These tests
/// pin the warning, because a warning nobody asserts is a warning that gets
/// deleted, and a digest printed clean over a half-empty manifest is the
/// confident wrong answer this tool exists to avoid.
fn run(profile: &str, bom: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spectrum"))
        .args(["compose", "--profile", profile, "--bom", bom])
        .output()
        .expect("spectrum binary runs")
}

fn stdout(profile: &str, bom: &str) -> String {
    let out = run(profile, bom);
    assert!(out.status.success(), "compose exited non-zero");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn compose_names_the_declared_kinds_no_collector_filled() {
    // `build-manifest` declares four kinds. The CycloneDX projection supplies
    // model and code_dependency; there is no system-prompt or tool-definition
    // collector in this crate, so the digest is over half the profile and
    // must say so.
    let s = stdout(
        "profiles/build-manifest.toml",
        "tests/fixtures/cyclonedx-a.json",
    );
    assert!(s.contains("DECLARED BUT ABSENT"));
    assert!(s.contains("system_prompt"));
    assert!(s.contains("tool_definition"));
    // The two that *were* collected must not be listed as absent.
    let absent = s.split("DECLARED BUT ABSENT").nth(1).unwrap();
    let absent = absent.split("The digest commits only").next().unwrap();
    assert!(!absent.contains("- model"));
    assert!(!absent.contains("- code_dependency"));
}

#[test]
fn compose_reports_unmapped_cyclonedx_types_with_counts() {
    let s = stdout(
        "profiles/build-manifest.toml",
        "tests/fixtures/cyclonedx-unmapped.json",
    );
    assert!(s.contains("UNMAPPED CycloneDX TYPES"));
    assert!(s.contains("container (2)"));
    assert!(s.contains("operating-system (1)"));
    assert!(s.contains("firmware (1)"));
    // A mapped type is projected, not dropped, so it must not be reported as
    // unmapped.
    assert!(!s.contains("machine-learning-model ("));
}

#[test]
fn compose_says_the_digest_covers_only_what_was_collected() {
    let s = stdout(
        "profiles/build-manifest.toml",
        "tests/fixtures/cyclonedx-a.json",
    );
    assert!(s.contains("The digest commits only to what was actually collected"));
    // The closing line comes after the gaps, so a reader who reaches it has
    // already read them.
    let gap = s.find("DECLARED BUT ABSENT").unwrap();
    let closing = s.find("The digest commits only").unwrap();
    assert!(gap < closing);
}

#[test]
fn compose_does_not_print_an_unmapped_section_when_there_is_nothing_to_report() {
    // The heading must mean something. A document with no unmapped types must
    // not print an empty "UNMAPPED" block, or the section becomes furniture a
    // reader learns to skip.
    let s = stdout(
        "profiles/build-manifest.toml",
        "tests/fixtures/cyclonedx-a.json",
    );
    assert!(!s.contains("UNMAPPED CycloneDX TYPES"));
}

#[test]
fn compose_still_prints_a_digest_alongside_the_warnings() {
    // The human ruling was warn, not refuse. The digest must still be there
    // and the exit code must still be zero, or this became a refusal.
    let out = run(
        "profiles/build-manifest.toml",
        "tests/fixtures/cyclonedx-unmapped.json",
    );
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("digest    sha-256:"));
}
