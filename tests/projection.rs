use spectrum::component::{digest, Kind, Witness};
use spectrum::profile::Profile;
use spectrum::project::from_cyclonedx;

fn profile() -> Profile {
    Profile::load(std::path::Path::new("profiles/build-manifest.toml")).unwrap()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap()
}

#[test]
fn two_boms_of_the_same_system_project_identically() {
    // The whole point of projecting rather than digesting. `serialNumber` is
    // a fresh UUID per generation and `metadata.timestamp` is generation
    // time, so digesting the document would report a system as diverging
    // from itself — `parallax`'s `12h`-versus-`720m` defect at the input.
    let p = profile();
    let a = from_cyclonedx(&fixture("cyclonedx-a.json"), &p).unwrap();
    let b = from_cyclonedx(&fixture("cyclonedx-b.json"), &p).unwrap();
    assert_eq!(a, b);
}

#[test]
fn the_projection_is_ordered_independently_of_the_document() {
    // The fixtures list their components in opposite orders. A projection
    // that preserved document order would make the digest depend on the
    // generator's iteration, which is exactly the positional-identity defect
    // `parallax` shipped and had to fix.
    let p = profile();
    let a = from_cyclonedx(&fixture("cyclonedx-a.json"), &p).unwrap();
    let mut sorted = a.clone();
    sorted.sort();
    assert_eq!(a, sorted);
}

#[test]
fn a_component_with_a_hash_is_measured_and_one_without_is_declared() {
    let p = profile();
    let cs = from_cyclonedx(&fixture("cyclonedx-a.json"), &p).unwrap();
    let model = cs.iter().find(|c| c.kind == Kind::Model).unwrap();
    assert_eq!(model.witness, Witness::Measured);
    assert_eq!(model.content_digest.as_deref(), Some("sha-256:aa11"));

    let dep = cs.iter().find(|c| c.kind == Kind::CodeDependency).unwrap();
    assert_eq!(dep.witness, Witness::Declared);
    assert_eq!(dep.content_digest, None);
}

#[test]
fn kinds_outside_the_profile_are_dropped() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("narrow.toml");
    std::fs::write(
        &path,
        "id = \"narrow\"\nkinds = [\"model\"]\ncoverage = [\"everything else\"]\n",
    )
    .unwrap();
    let narrow = Profile::load(&path).unwrap();
    let cs = from_cyclonedx(&fixture("cyclonedx-a.json"), &narrow).unwrap();
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].kind, Kind::Model);
}

#[test]
fn malformed_documents_error_rather_than_panic() {
    let p = profile();
    for bad in [
        "",
        "{",
        "null",
        "[]",
        r#"{"components": 3}"#,
        r#"{"components":[{"type":"library"}]}"#,
    ] {
        let _ = from_cyclonedx(bad, &p);
    }
}

#[test]
fn a_projected_component_has_a_stable_digest() {
    let p = profile();
    let a = from_cyclonedx(&fixture("cyclonedx-a.json"), &p).unwrap();
    let b = from_cyclonedx(&fixture("cyclonedx-b.json"), &p).unwrap();
    let da: Vec<String> = a.iter().map(|c| digest(c).unwrap()).collect();
    let db: Vec<String> = b.iter().map(|c| digest(c).unwrap()).collect();
    assert_eq!(da, db);
}
