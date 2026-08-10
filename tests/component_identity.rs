use spectrum::component::{digest, ComponentRef, Kind, Witness};

fn base() -> ComponentRef {
    ComponentRef {
        kind: Kind::ToolDefinition,
        id: "pkg:generic/charge-tool@1.0.0".into(),
        content_digest: Some("sha-256:aa".into()),
        witness: Witness::Measured,
    }
}

#[test]
fn the_same_component_always_digests_the_same() {
    assert_eq!(digest(&base()).unwrap(), digest(&base()).unwrap());
}

#[test]
fn every_field_contributes_to_the_component_digest() {
    let d = digest(&base()).unwrap();

    let mut kind = base();
    kind.kind = Kind::McpServer;
    assert_ne!(digest(&kind).unwrap(), d, "kind did not contribute");

    let mut id = base();
    id.id = "pkg:generic/charge-tool@1.0.1".into();
    assert_ne!(digest(&id).unwrap(), d, "id did not contribute");

    let mut cd = base();
    cd.content_digest = Some("sha-256:bb".into());
    assert_ne!(digest(&cd).unwrap(), d, "content_digest did not contribute");

    let mut absent = base();
    absent.content_digest = None;
    assert_ne!(
        digest(&absent).unwrap(),
        d,
        "an absent digest did not contribute"
    );
}

#[test]
fn the_witness_class_is_inside_the_digest() {
    // OPEN FORK 2, resolved as recommended and pinned here so reversing it is
    // a visible test change rather than a silent behaviour change.
    //
    // A component the operator merely typed must not hash like one whose bytes
    // were read. A digest that cannot tell those apart is the confident wrong
    // answer this whole tool exists to avoid.
    let mut declared = base();
    declared.witness = Witness::Declared;
    assert_ne!(digest(&declared).unwrap(), digest(&base()).unwrap());
}

#[test]
fn a_component_id_is_compared_as_opaque_bytes() {
    // Two spellings that a human reads as the same version are different
    // components here, deliberately. Normalizing would mean deciding which
    // textual differences a consumer's parser ignores, which this tool cannot
    // know; guessing wrong merges two distinguishable components.
    let mut a = base();
    a.id = "pkg:generic/x@1.0".into();
    let mut b = base();
    b.id = "pkg:generic/x@1.0.0".into();
    assert_ne!(digest(&a).unwrap(), digest(&b).unwrap());
}

#[test]
fn opaque_components_still_digest() {
    // `Opaque` is a positive statement that something is present and
    // unmeasurable, not the absence of a component, so it must have an
    // identity like any other.
    let c = ComponentRef {
        kind: Kind::Model,
        id: "training-data-behind-hosted-model".into(),
        content_digest: None,
        witness: Witness::Opaque,
    };
    assert!(digest(&c).is_ok());
}
