use spectrum::component::{digest, ComponentRef, Kind, Witness};
use std::collections::HashSet;

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
fn every_kind_tag_is_distinct() {
    // `Kind::tag`'s own doc comment says the strings are hand-written because
    // they are inside a digest. Nothing checked that two of them were not the
    // same string. A copy-paste — `Kind::MemoryRead => "retrieved_chunk"` —
    // would make a memory read and a retrieved chunk hash identically,
    // silently merging two composition kinds, and every other test in this
    // repository would still pass.
    //
    // The variant list is written out literally rather than derived, so
    // adding a `Kind` without adding a tag is a compile error here.
    let all = [
        Kind::Model,
        Kind::SystemPrompt,
        Kind::ToolDefinition,
        Kind::McpServer,
        Kind::CodeDependency,
        Kind::RetrievedChunk,
        Kind::MemoryRead,
        Kind::SubAgent,
    ];
    let tags: HashSet<&str> = all.iter().map(|k| k.tag()).collect();
    assert_eq!(
        tags.len(),
        all.len(),
        "two `Kind` variants share a tag, so two composition kinds digest alike"
    );
}

#[test]
fn every_witness_tag_is_distinct() {
    // Same hazard, and worse here: the witness class is inside the digest
    // (fork 2) precisely so a `Declared` component cannot hash like a
    // `Measured` one. Two witness classes sharing a tag would undo that
    // resolution without touching the code that implements it.
    let all = [
        Witness::Measured,
        Witness::Attested,
        Witness::Declared,
        Witness::Opaque,
    ];
    let tags: HashSet<&str> = all.iter().map(|w| w.tag()).collect();
    assert_eq!(
        tags.len(),
        all.len(),
        "two `Witness` variants share a tag, so two evidence classes digest alike"
    );
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
