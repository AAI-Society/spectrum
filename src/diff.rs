use crate::component::{ComponentRef, Kind, Witness};
use crate::manifest::Manifest;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Change {
    Added,
    Removed,
    ContentChanged {
        from: Option<String>,
        to: Option<String>,
    },
    /// The system did not change; what we know about it did. Reported
    /// separately because conflating the two is how a fleet-wide
    /// instrumentation rollout reads as a fleet-wide composition change.
    WitnessChanged {
        from: Witness,
        to: Witness,
    },
}

#[derive(Clone, Debug)]
pub struct Difference {
    pub kind: Kind,
    pub id: String,
    pub change: Change,
}

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error(
        "refusing to compare a `{left}` manifest with a `{right}` one: a digest is a \
         commitment under a stated theory of composition, and a difference between two \
         theories is not a difference between two systems"
    )]
    ProfileMismatch { left: String, right: String },
}

/// Components are keyed by `(kind, id)` — their identity as *things*, not
/// their digest, since a changed digest is the difference we want to report
/// rather than a different component.
fn index(m: &Manifest) -> BTreeMap<(Kind, &str), &ComponentRef> {
    m.components
        .iter()
        .map(|c| ((c.kind, c.id.as_str()), c))
        .collect()
}

pub fn compare(a: &Manifest, b: &Manifest) -> Result<Vec<Difference>, DiffError> {
    if a.profile_id != b.profile_id {
        return Err(DiffError::ProfileMismatch {
            left: a.profile_id.clone(),
            right: b.profile_id.clone(),
        });
    }

    let (ia, ib) = (index(a), index(b));
    let mut out = Vec::new();

    for (key, left) in &ia {
        match ib.get(key) {
            None => out.push(Difference {
                kind: key.0,
                id: key.1.to_string(),
                change: Change::Removed,
            }),
            Some(right) => {
                if left.witness != right.witness {
                    out.push(Difference {
                        kind: key.0,
                        id: key.1.to_string(),
                        change: Change::WitnessChanged {
                            from: left.witness,
                            to: right.witness,
                        },
                    });
                }
                if left.content_digest != right.content_digest {
                    out.push(Difference {
                        kind: key.0,
                        id: key.1.to_string(),
                        change: Change::ContentChanged {
                            from: left.content_digest.clone(),
                            to: right.content_digest.clone(),
                        },
                    });
                }
            }
        }
    }

    for key in ib.keys() {
        if !ia.contains_key(key) {
            out.push(Difference {
                kind: key.0,
                id: key.1.to_string(),
                change: Change::Added,
            });
        }
    }

    Ok(out)
}
