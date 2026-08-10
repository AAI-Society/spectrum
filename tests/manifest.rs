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
fn a_duplicate_component_does_not_change_the_manifest_digest() {
    // Dedup is part of a manifest's identity: a manifest that listed the
    // same component twice would otherwise digest differently from one that
    // listed it once, because the component-count length prefix makes that
    // difference real, not incidental.
    let p = profile("build-manifest");
    let c = comps()[0].clone();
    let once = Manifest::new(&p, vec![c.clone()]).digest().unwrap();
    let twice = Manifest::new(&p, vec![c.clone(), c]).digest().unwrap();
    assert_eq!(twice, once);
}

#[test]
fn the_manifest_digest_of_a_two_kind_manifest_is_pinned() {
    // This test exists so that reordering the `Kind` variants is a visible
    // test failure rather than a silent change to every multi-kind digest.
    //
    // `ComponentRef` derives `Ord` with `kind` first, so the *declaration
    // order* of `Kind` decides the order `Manifest::new` sorts into and
    // therefore the order `Manifest::digest` concatenates in. Swapping two
    // variants is a diff that looks cosmetic — the tags are pinned, the
    // components are unchanged, the count is unchanged — and it moves the
    // digest of every manifest holding more than one kind.
    //
    // `component_order_does_not_change_the_manifest_digest` cannot catch it,
    // because it re-sorts both sides and so compares two digests that moved
    // together. Only a hardcoded expected value catches it.
    //
    // `comps()` is deliberately a *two-kind* manifest (`Model` and
    // `CodeDependency`), because a one-kind manifest's digest is invariant
    // under a `Kind` reorder and would pin nothing.
    //
    // The value below was produced by running this code, not derived by hand.
    // If it fails, do not update it without establishing which input moved:
    // the tags, the component fields, the domain tags, or the enum order.
    let p = profile("build-manifest");
    assert_eq!(
        Manifest::new(&p, comps()).digest().unwrap(),
        "sha-256:748251dd0aea6a90dcf19fa909f4d6478fb75062d2ac154b2bb33ce0d1afe9fa"
    );
}

#[test]
fn a_manifest_exposes_its_components_sorted_however_it_was_built() {
    // The invariant `Manifest`'s doc comment promises, asserted through the
    // only surface a caller has. `components` is a private field behind a
    // `&[ComponentRef]` accessor, so there is no `push`, no `components_mut`
    // and no way to append after construction — which is the half of this
    // that cannot be tested, only prevented. What is testable is that the
    // accessor's output is sorted whatever order the input arrived in.
    //
    // It matters because `Manifest::digest` concatenates in slice order while
    // `diff::compare` indexes into a `BTreeMap` and does not. If a caller
    // could disturb the order, two manifests could `compare()` as identical
    // and digest differently: a producer and a verifier in one crate
    // disagreeing about the same pair.
    let p = profile("build-manifest");
    let mut unsorted = comps();
    unsorted.reverse();
    assert_ne!(unsorted, comps(), "the fixture is already in sorted order");

    let m = Manifest::new(&p, unsorted);
    let mut expected = m.components().to_vec();
    expected.sort();
    assert_eq!(m.components(), expected.as_slice());
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
