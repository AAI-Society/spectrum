use spectrum::component::{ComponentRef, Kind, Witness};
use spectrum::diff::{compare, Change, DiffError};
use spectrum::manifest::Manifest;
use spectrum::profile::Profile;

fn profile(name: &str) -> Profile {
    Profile::load(&std::path::Path::new("profiles").join(format!("{name}.toml"))).unwrap()
}

fn model(d: &str, w: Witness) -> ComponentRef {
    ComponentRef {
        kind: Kind::Model,
        id: "pkg:generic/classifier@3.1.0".into(),
        content_digest: Some(d.into()),
        witness: w,
    }
}

#[test]
fn identical_manifests_differ_in_nothing() {
    let p = profile("build-manifest");
    let a = Manifest::new(&p, vec![model("sha-256:aa", Witness::Measured)]);
    let b = Manifest::new(&p, vec![model("sha-256:aa", Witness::Measured)]);
    assert!(compare(&a, &b).unwrap().is_empty());
}

#[test]
fn a_changed_digest_is_reported_as_a_content_change() {
    let p = profile("build-manifest");
    let a = Manifest::new(&p, vec![model("sha-256:aa", Witness::Measured)]);
    let b = Manifest::new(&p, vec![model("sha-256:bb", Witness::Measured)]);
    let d = compare(&a, &b).unwrap();
    assert_eq!(d.len(), 1);
    assert!(matches!(d[0].change, Change::ContentChanged { .. }));
}

#[test]
fn a_witness_class_change_is_a_distinct_kind_of_difference() {
    // OPEN FORK 2's accepted cost: improving instrumentation changes the
    // digest even though the system did not change. `diff` is the mitigation,
    // and it only mitigates if the two are distinguishable in the output.
    let p = profile("build-manifest");
    let a = Manifest::new(&p, vec![model("sha-256:aa", Witness::Declared)]);
    let b = Manifest::new(&p, vec![model("sha-256:aa", Witness::Measured)]);
    let d = compare(&a, &b).unwrap();
    assert_eq!(d.len(), 1);
    assert!(
        matches!(d[0].change, Change::WitnessChanged { .. }),
        "a witness-class change must not be reported as a content change"
    );
}

#[test]
fn added_and_removed_components_are_reported() {
    let p = profile("build-manifest");
    let a = Manifest::new(&p, vec![model("sha-256:aa", Witness::Measured)]);
    let b = Manifest::new(&p, vec![]);
    let d = compare(&a, &b).unwrap();
    assert!(matches!(d[0].change, Change::Removed));
    let d2 = compare(&b, &a).unwrap();
    assert!(matches!(d2[0].change, Change::Added));
}

#[test]
fn comparing_across_profiles_refuses_and_names_both() {
    // A digest is a commitment under a stated theory of composition.
    // Reporting a difference between two theories would be a category error
    // dressed as a finding.
    let a = Manifest::new(
        &profile("build-manifest"),
        vec![model("sha-256:aa", Witness::Measured)],
    );
    let b = Manifest::new(
        &profile("runtime-tools"),
        vec![model("sha-256:aa", Witness::Measured)],
    );
    match compare(&a, &b) {
        Err(DiffError::ProfileMismatch { left, right }) => {
            assert_eq!(left, "build-manifest");
            assert_eq!(right, "runtime-tools");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}
