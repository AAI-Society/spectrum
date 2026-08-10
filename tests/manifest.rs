use spectrum::component::{ComponentRef, Kind, Witness};
use spectrum::manifest::Manifest;
use spectrum::profile::Profile;

fn profile(name: &str) -> Profile {
    Profile::load(&std::path::Path::new("profiles").join(format!("{name}.toml"))).unwrap()
}

fn comps() -> Vec<ComponentRef> {
    vec![
        ComponentRef {
            kind: Kind::Model,
            id: "pkg:generic/classifier@3.1.0".into(),
            content_digest: Some("sha-256:aa11".into()),
            witness: Witness::Measured,
        },
        ComponentRef {
            kind: Kind::CodeDependency,
            id: "pkg:cargo/serde@1.0.0".into(),
            content_digest: None,
            witness: Witness::Declared,
        },
    ]
}

#[test]
fn the_manifest_digest_is_stable() {
    let p = profile("build-manifest");
    assert_eq!(
        Manifest::new(&p, comps()).digest().unwrap(),
        Manifest::new(&p, comps()).digest().unwrap()
    );
}

#[test]
fn component_order_does_not_change_the_manifest_digest() {
    let p = profile("build-manifest");
    let a = Manifest::new(&p, comps()).digest().unwrap();
    let mut reversed = comps();
    reversed.reverse();
    assert_eq!(Manifest::new(&p, reversed).digest().unwrap(), a);
}

#[test]
fn the_profile_id_is_inside_the_manifest_digest() {
    // Two theories of what composition means must produce digests that
    // cannot be compared. This is `parallax`'s rule that trust sets for
    // different claims are not ranked, applied to composition.
    let a = Manifest::new(&profile("build-manifest"), comps())
        .digest()
        .unwrap();
    let b = Manifest::new(&profile("runtime-tools"), comps())
        .digest()
        .unwrap();
    assert_ne!(
        a, b,
        "the same components under two profiles produced one digest"
    );
}

#[test]
fn changing_any_component_changes_the_manifest_digest() {
    let p = profile("build-manifest");
    let base = Manifest::new(&p, comps()).digest().unwrap();
    let mut changed = comps();
    changed[0].content_digest = Some("sha-256:bb22".into());
    assert_ne!(Manifest::new(&p, changed).digest().unwrap(), base);
}

#[test]
fn adding_a_component_changes_the_manifest_digest() {
    let p = profile("build-manifest");
    let base = Manifest::new(&p, comps()).digest().unwrap();
    let mut more = comps();
    more.push(ComponentRef {
        kind: Kind::SystemPrompt,
        id: "prompts/main.txt".into(),
        content_digest: Some("sha-256:cc33".into()),
        witness: Witness::Measured,
    });
    assert_ne!(Manifest::new(&p, more).digest().unwrap(), base);
}

#[test]
fn an_empty_manifest_still_digests_and_is_distinct_per_profile() {
    let a = Manifest::new(&profile("build-manifest"), vec![])
        .digest()
        .unwrap();
    let b = Manifest::new(&profile("runtime-tools"), vec![])
        .digest()
        .unwrap();
    assert_ne!(a, b);
}
