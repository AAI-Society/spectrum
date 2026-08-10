use crate::component::{self, ComponentError, ComponentRef};
use crate::profile::Profile;
use sha2::{Digest, Sha256};

/// A static composition manifest under one profile.
#[derive(Clone, Debug)]
pub struct Manifest {
    pub profile_id: String,
    /// Always sorted and deduplicated on construction, so two manifests over
    /// the same components are equal whatever order they were assembled in.
    pub components: Vec<ComponentRef>,
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

    /// `H("spectrum-manifest-v1" ‖ profile_id ‖ component_digest*)`.
    ///
    /// The profile id is inside, so a digest is a commitment *under a stated
    /// theory of what composition means*. `diff` refuses across profiles for
    /// the same reason.
    ///
    /// The domain tag is present so a manifest digest can never collide with
    /// a component digest or a register value, all three of which are
    /// SHA-256 over structured input in this crate.
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
