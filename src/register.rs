use crate::component::{self, ComponentError, ComponentRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ComponentAdded,
    ComponentChanged,
    ComponentRemoved,
}

impl EventKind {
    pub fn tag(&self) -> &'static str {
        match self {
            EventKind::ComponentAdded => "component_added",
            EventKind::ComponentChanged => "component_changed",
            EventKind::ComponentRemoved => "component_removed",
        }
    }
}

/// One composition change.
///
/// `hook` is the `interception_point` it occurred at. Three of the standard's
/// eight hooks are composition-change points — `CONTEXT_ASSEMBLY`,
/// `MEMORY_WRITE`, `SUBAGENT_DELEGATION` — which is evidence that the
/// runtime reading of `agbom_digest` is the intended one.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub kind: EventKind,
    pub hook: String,
    pub component: ComponentRef,
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error(transparent)]
    Component(#[from] ComponentError),
}

/// A PCR-style running commitment over composition changes.
///
/// ```text
/// C₀ = H("spectrum-register-v1" ‖ profile_id ‖ static_manifest_digest)
/// Cₙ = H(Cₙ₋₁ ‖ H(event))
/// ```
///
/// **One-way by construction.** The inner value is private, there is no
/// setter, no `reset`, and no constructor other than `genesis`. A "reset
/// composition" convenience would be a rewrite of history, so there is not
/// one. The 32-byte value is what makes a per-step field affordable however
/// much composition has changed.
///
/// The honest half: this solves the *record size* problem completely and the
/// *verification* problem not at all. Checking a register requires its
/// events, so somebody must store them — and that somebody is `ephemeris`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register {
    value: [u8; 32],
}

impl Register {
    pub fn genesis(profile_id: &str, static_manifest_digest: &str) -> Self {
        let mut h = Sha256::new();
        h.update(b"spectrum-register-v1");
        h.update((profile_id.len() as u64).to_be_bytes());
        h.update(profile_id.as_bytes());
        h.update((static_manifest_digest.len() as u64).to_be_bytes());
        h.update(static_manifest_digest.as_bytes());
        Self {
            value: h.finalize().into(),
        }
    }

    pub fn extend(&mut self, e: &Event) -> Result<(), RegisterError> {
        let ed = event_digest(e)?;
        let mut h = Sha256::new();
        h.update(self.value);
        h.update(ed);
        self.value = h.finalize().into();
        Ok(())
    }

    pub fn value(&self) -> String {
        format!("sha-256:{}", hex::encode(self.value))
    }
}

/// The identity of one event. Every field contributes, including the hook:
/// the same component arriving at `CONTEXT_ASSEMBLY` and at `MEMORY_WRITE`
/// are different composition changes.
fn event_digest(e: &Event) -> Result<[u8; 32], RegisterError> {
    let cd = component::digest(&e.component)?;
    let mut h = Sha256::new();
    h.update(b"spectrum-event-v1");
    h.update((e.kind.tag().len() as u64).to_be_bytes());
    h.update(e.kind.tag().as_bytes());
    h.update((e.hook.len() as u64).to_be_bytes());
    h.update(e.hook.as_bytes());
    h.update((cd.len() as u64).to_be_bytes());
    h.update(cd.as_bytes());
    Ok(h.finalize().into())
}

/// Recompute a register from its genesis and its log.
pub fn replay(genesis: Register, events: &[Event]) -> Result<Register, RegisterError> {
    let mut r = genesis;
    for e in events {
        r.extend(e)?;
    }
    Ok(r)
}
