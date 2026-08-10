use spectrum::component::Kind;
use spectrum::profile::{Profile, ProfileError};

fn write(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn a_profile_loads_and_reports_its_kinds() {
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "x.toml",
        r#"
id = "build-manifest"
kinds = ["model", "system_prompt", "tool_definition"]
coverage = ["everything that changes after task start"]
"#,
    );
    let prof = Profile::load(&p).unwrap();
    assert_eq!(prof.id, "build-manifest");
    assert!(prof.includes(Kind::Model));
    assert!(!prof.includes(Kind::RetrievedChunk));
}

#[test]
fn a_profile_with_empty_coverage_refuses_to_load() {
    // A profile claiming to capture everything does not load. This is the
    // mechanism by which the tool cannot assert an AgBOM schema: every
    // theory of composition must say what it misses.
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "x.toml",
        r#"
id = "everything"
kinds = ["model"]
coverage = []
"#,
    );
    assert!(matches!(
        Profile::load(&p),
        Err(ProfileError::EmptyCoverage { .. })
    ));
}

#[test]
fn a_profile_missing_coverage_entirely_refuses_to_load() {
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "x.toml",
        r#"
id = "everything"
kinds = ["model"]
"#,
    );
    assert!(Profile::load(&p).is_err());
}

#[test]
fn a_typod_kind_is_an_error_not_a_narrowed_manifest() {
    // `parallax`'s fifth critical: a one-letter typo silently disabled a
    // gate. Here it would silently drop a component kind from the manifest.
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "x.toml",
        r#"
id = "typo"
kinds = ["model", "sytem_prompt"]
coverage = ["a lot"]
"#,
    );
    assert!(Profile::load(&p).is_err());
}

#[test]
fn an_unknown_key_in_a_profile_is_an_error() {
    let d = tempfile::tempdir().unwrap();
    let p = write(
        d.path(),
        "x.toml",
        r#"
id = "x"
kinds = ["model"]
coverage = ["a lot"]
covrage = ["typo of the above"]
"#,
    );
    assert!(Profile::load(&p).is_err());
}

#[test]
fn no_shipped_profile_is_named_default() {
    // There is no default theory of what composition means, so there is no
    // default profile. Every digest can be traced to a named claim.
    for f in std::fs::read_dir("profiles").unwrap() {
        let path = f.unwrap().path();
        let prof = Profile::load(&path).unwrap();
        assert_ne!(prof.id, "default", "{} is named default", path.display());
        assert!(!prof.coverage.is_empty());
    }
}

#[test]
fn all_three_shipped_profiles_load() {
    for name in ["build-manifest", "runtime-tools", "runtime-with-context"] {
        let p = std::path::Path::new("profiles").join(format!("{name}.toml"));
        let prof = Profile::load(&p).unwrap();
        assert_eq!(prof.id, name);
    }
}
