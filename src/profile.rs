use crate::component::Kind;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A declarative statement of which component kinds are in scope, and what
/// this theory of composition does not capture.
///
/// **The profile is how this tool avoids asserting an AgBOM schema.** Its id
/// is inside every digest, so two deployments under different profiles produce
/// digests that cannot be compared — a digest is a commitment under a stated
/// theory of what composition means, and comparing across theories is a
/// category error. This is `parallax`'s rule that trust sets for different
/// claims are not ranked, applied to composition.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub kinds: Vec<Kind>,
    /// What this profile cannot capture. **Required and non-empty**: a profile
    /// claiming complete capture does not load.
    pub coverage: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("could not read the profile at {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("could not parse the profile at {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error(
        "the profile `{id}` declares an empty `coverage`. Every profile must state what it \
         does not capture; a profile implying complete capture is the confident wrong answer \
         this tool exists to avoid"
    )]
    EmptyCoverage { id: String },
    #[error("the profile `{id}` declares no component kinds")]
    NoKinds { id: String },
}

impl Profile {
    pub fn load(path: &Path) -> Result<Self, ProfileError> {
        let text = std::fs::read_to_string(path).map_err(|source| ProfileError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let p: Profile = toml::from_str(&text).map_err(|source| ProfileError::Parse {
            path: path.display().to_string(),
            source,
        })?;
        if p.coverage.is_empty() {
            return Err(ProfileError::EmptyCoverage { id: p.id });
        }
        if p.kinds.is_empty() {
            return Err(ProfileError::NoKinds { id: p.id });
        }
        Ok(p)
    }

    pub fn includes(&self, k: Kind) -> bool {
        self.kinds.contains(&k)
    }
}
