use spectrum::component::{ComponentRef, Kind, Witness};
use spectrum::register::{replay, Event, EventKind, Register};

fn event(id: &str) -> Event {
    Event {
        kind: EventKind::ComponentAdded,
        hook: "CONTEXT_ASSEMBLY".into(),
        component: ComponentRef {
            kind: Kind::RetrievedChunk,
            id: id.into(),
            content_digest: Some(format!("sha-256:{id}")),
            witness: Witness::Measured,
        },
    }
}

/// Like `event`, but with `kind` and `hook` as parameters so tests can vary
/// exactly one of `Event`'s three digest-contributing fields at a time.
fn event_with(kind: EventKind, hook: &str, id: &str) -> Event {
    Event {
        kind,
        hook: hook.into(),
        component: ComponentRef {
            kind: Kind::RetrievedChunk,
            id: id.into(),
            content_digest: Some(format!("sha-256:{id}")),
            witness: Witness::Measured,
        },
    }
}

#[test]
fn a_static_deployment_never_advances_its_register() {
    // OPEN FORK 3's recommendation rests on this: the register degenerates
    // exactly to the static root when nothing changes, so the field means the
    // same thing in a static and a dynamic deployment.
    let r = Register::genesis("build-manifest", "sha-256:aa");
    assert_eq!(
        r.value(),
        Register::genesis("build-manifest", "sha-256:aa").value()
    );
}

#[test]
fn genesis_depends_on_both_the_profile_and_the_static_root() {
    assert_ne!(
        Register::genesis("build-manifest", "sha-256:aa").value(),
        Register::genesis("runtime-tools", "sha-256:aa").value()
    );
    assert_ne!(
        Register::genesis("build-manifest", "sha-256:aa").value(),
        Register::genesis("build-manifest", "sha-256:bb").value()
    );
}

#[test]
fn extension_advances_the_register() {
    let mut r = Register::genesis("runtime-tools", "sha-256:aa");
    let before = r.value();
    r.extend(&event("chunk-1")).unwrap();
    assert_ne!(r.value(), before);
}

#[test]
fn extension_is_order_dependent() {
    // A register that ignored order would let two composition histories with
    // the same components collapse to one value, which is the whole point of
    // a chain rather than a set.
    let mut a = Register::genesis("runtime-tools", "sha-256:aa");
    a.extend(&event("chunk-1")).unwrap();
    a.extend(&event("chunk-2")).unwrap();

    let mut b = Register::genesis("runtime-tools", "sha-256:aa");
    b.extend(&event("chunk-2")).unwrap();
    b.extend(&event("chunk-1")).unwrap();

    assert_ne!(a.value(), b.value());
}

#[test]
fn the_register_replays_from_its_log() {
    // A register that cannot be replayed is a number, not a commitment.
    let events = vec![event("chunk-1"), event("chunk-2"), event("chunk-3")];
    let mut r = Register::genesis("runtime-tools", "sha-256:aa");
    for e in &events {
        r.extend(e).unwrap();
    }
    let replayed = replay(Register::genesis("runtime-tools", "sha-256:aa"), &events).unwrap();
    assert_eq!(replayed.value(), r.value());
}

#[test]
fn replaying_a_truncated_log_does_not_reproduce_the_register() {
    let events = vec![event("chunk-1"), event("chunk-2"), event("chunk-3")];
    let mut r = Register::genesis("runtime-tools", "sha-256:aa");
    for e in &events {
        r.extend(e).unwrap();
    }
    let short = replay(
        Register::genesis("runtime-tools", "sha-256:aa"),
        &events[..2],
    )
    .unwrap();
    assert_ne!(short.value(), r.value());
}

#[test]
fn the_hook_is_inside_the_event_digest() {
    // The same component arriving at CONTEXT_ASSEMBLY and at MEMORY_WRITE are
    // different composition changes, so the hook must move the register.
    let mut a = Register::genesis("runtime-tools", "sha-256:aa");
    a.extend(&event_with(
        EventKind::ComponentAdded,
        "CONTEXT_ASSEMBLY",
        "chunk-1",
    ))
    .unwrap();

    let mut b = Register::genesis("runtime-tools", "sha-256:aa");
    b.extend(&event_with(
        EventKind::ComponentAdded,
        "MEMORY_WRITE",
        "chunk-1",
    ))
    .unwrap();

    assert_ne!(a.value(), b.value());
}

#[test]
fn the_event_kind_is_inside_the_event_digest() {
    let mut a = Register::genesis("runtime-tools", "sha-256:aa");
    a.extend(&event_with(
        EventKind::ComponentAdded,
        "CONTEXT_ASSEMBLY",
        "chunk-1",
    ))
    .unwrap();

    let mut b = Register::genesis("runtime-tools", "sha-256:aa");
    b.extend(&event_with(
        EventKind::ComponentChanged,
        "CONTEXT_ASSEMBLY",
        "chunk-1",
    ))
    .unwrap();

    assert_ne!(a.value(), b.value());
}

#[test]
fn identical_events_extend_two_registers_identically() {
    // The equal direction, closing the loop with the two tests above: same
    // kind, same hook, same component must produce the same register value.
    let mut a = Register::genesis("runtime-tools", "sha-256:aa");
    a.extend(&event_with(
        EventKind::ComponentAdded,
        "CONTEXT_ASSEMBLY",
        "chunk-1",
    ))
    .unwrap();

    let mut b = Register::genesis("runtime-tools", "sha-256:aa");
    b.extend(&event_with(
        EventKind::ComponentAdded,
        "CONTEXT_ASSEMBLY",
        "chunk-1",
    ))
    .unwrap();

    assert_eq!(a.value(), b.value());
}

#[test]
fn a_register_value_never_repeats_within_a_run() {
    // The one-way property, asserted rather than asserted-about. There is no
    // API that lowers a register; this checks that extension does not produce
    // a collision that would let a history be replaced by a shorter one.
    let mut r = Register::genesis("runtime-tools", "sha-256:aa");
    let mut seen = std::collections::HashSet::new();
    seen.insert(r.value());
    for i in 0..64 {
        r.extend(&event(&format!("chunk-{i}"))).unwrap();
        assert!(
            seen.insert(r.value()),
            "register value repeated at step {i}"
        );
    }
}
