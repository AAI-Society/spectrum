use crate::component::{self, ComponentError, ComponentRef};
use crate::profile::Profile;
use sha2::{Digest, Sha256};

/// A static composition manifest under one profile.
///
/// **Both fields are private, and that is the invariant.** `components` is
/// sorted and deduplicated by [`Manifest::new`], and [`Manifest::digest`]
/// concatenates in that order while [`crate::diff::compare`] indexes into a
/// `BTreeMap` and is order-insensitive. A public `Vec` that a caller could
/// push onto after construction would let those two disagree: two manifests
/// holding the same components in different orders would `compare()` as
/// identical and digest differently — a producer and a verifier inside one
/// crate reaching opposite conclusions about the same pair of manifests.
/// There is deliberately no `push`, no `components_mut`, and no `From<Vec<_>>`;
/// [`Manifest::new`] is the only way to obtain one, so the invariant holds by
/// construction rather than by promise.
#[derive(Clone, Debug)]
pub struct Manifest {
    profile_id: String,
    /// Always sorted and deduplicated on construction, so two manifests over
    /// the same components are equal whatever order they were assembled in.
    components: Vec<ComponentRef>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error(transparent)]
    Component(#[from] ComponentError),
}

impl Manifest {
    pub fn new(profile: &Profile, mut components: Vec<ComponentRef>) -> Self {
        components.sort();
        components.dedup();
        Self {
            profile_id: profile.id.clone(),
            components,
        }
    }

    /// The profile this manifest was taken under. Read-only: the profile id is
    /// inside the digest, so a manifest whose profile could be reassigned
    /// after construction would be a manifest whose digest no longer means
    /// what its `profile_id` says.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// The manifest's components, sorted and deduplicated. Returned as a slice
    /// rather than a `&Vec` so there is no route to mutation.
    pub fn components(&self) -> &[ComponentRef] {
        &self.components
    }

    /// `H("spectrum-manifest-v1" ‖ profile_id ‖ component_digest*)`.
    ///
    /// The profile id is inside, so a digest is a commitment *under a stated
    /// theory of what composition means*. `diff` refuses across profiles for
    /// the same reason.
    ///
    /// The domain tag is present so a manifest digest can never collide with
    /// any other SHA-256 this crate takes over structured input. There are
    /// six such derivations: [`component::digest`]
    /// (`spectrum-component-v1`), this one (`spectrum-manifest-v1`),
    /// [`Register::genesis`](crate::register::Register::genesis)
    /// (`spectrum-register-v1`), `event_digest` (`spectrum-event-v1`),
    /// [`Register::extend`](crate::register::Register::extend), and
    /// [`collect::mcp::measure`](crate::collect::mcp::measure). The first four
    /// carry a tag. `extend` needs none: it hashes exactly two fixed-width
    /// 32-byte values, so there is no length ambiguity to exploit and no
    /// structured input to reinterpret. `measure` needs none either: its
    /// SHA-256 is over an MCP tool list and becomes a `content_digest`, a
    /// position no component, manifest, event or register digest ever
    /// occupies.
    ///
    /// Each component digest is length-prefixed, so a set of two components
    /// cannot be made to hash like one component whose id happens to contain
    /// the concatenation.
    pub fn digest(&self) -> Result<String, ManifestError> {
        let mut h = Sha256::new();
        h.update(b"spectrum-manifest-v1");
        h.update((self.profile_id.len() as u64).to_be_bytes());
        h.update(self.profile_id.as_bytes());
        h.update((self.components.len() as u64).to_be_bytes());
        for c in &self.components {
            let d = component::digest(c)?;
            h.update((d.len() as u64).to_be_bytes());
            h.update(d.as_bytes());
        }
        Ok(format!("sha-256:{}", hex::encode(h.finalize())))
    }
}
