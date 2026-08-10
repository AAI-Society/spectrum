use crate::component::{ComponentRef, Kind, Witness};
use crate::profile::Profile;
use serde::Deserialize;

/// The subset of CycloneDX this projection reads.
///
/// Deliberately **not** `deny_unknown_fields`: a CycloneDX document carries
/// far more than this, and refusing documents that contain fields we ignore
/// would make the tool unusable. The `deny_unknown_fields` rule applies to
/// *our own* configuration surfaces, where an unknown key is a typo; here an
/// unknown key is the rest of a standard we do not consume.
///
/// Note what is absent: `serialNumber` and `metadata` are not read at all.
/// They cannot influence the projection because they are not in this struct.
#[derive(Debug, Deserialize)]
struct Bom {
    #[serde(default)]
    components: Vec<Comp>,
}

#[derive(Debug, Deserialize)]
struct Comp {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    purl: Option<String>,
    #[serde(default)]
    hashes: Vec<Hash>,
}

#[derive(Debug, Deserialize)]
struct Hash {
    alg: String,
    content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("could not parse the CycloneDX document: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Map CycloneDX component types onto our kinds. Types we have no kind for
/// are dropped, which is safe because the profile's `coverage` says what is
/// not captured.
fn kind_of(t: &str) -> Option<Kind> {
    match t {
        "machine-learning-model" => Some(Kind::Model),
        "library" | "framework" | "application" | "file" => Some(Kind::CodeDependency),
        "data" => Some(Kind::RetrievedChunk),
        _ => None,
    }
}

/// Project a CycloneDX document into a normalized, **sorted** component set.
///
/// Sorting is not cosmetic. It is the identity: a projection that preserved
/// document order would make the manifest digest depend on the generator's
/// iteration order, which is the positional-identity defect `parallax`
/// shipped and had to fix — a file that compared `Incomparable` to a
/// reordering of itself.
pub fn from_cyclonedx(json: &str, p: &Profile) -> Result<Vec<ComponentRef>, ProjectError> {
    let bom: Bom = serde_json::from_str(json)?;
    let mut out = Vec::new();
    for c in bom.components {
        let Some(kind) = kind_of(&c.kind) else {
            continue;
        };
        if !p.includes(kind) {
            continue;
        }
        let sha = c
            .hashes
            .iter()
            .find(|h| h.alg.eq_ignore_ascii_case("SHA-256"))
            .map(|h| format!("sha-256:{}", h.content));
        let id = match (&c.purl, &c.version) {
            (Some(purl), _) => purl.clone(),
            (None, Some(v)) => format!("{}@{}", c.name, v),
            (None, None) => c.name.clone(),
        };
        let witness = if sha.is_some() {
            Witness::Measured
        } else {
            Witness::Declared
        };
        out.push(ComponentRef {
            kind,
            id,
            content_digest: sha,
            witness,
        });
    }
    out.sort();
    out.dedup();
    Ok(out)
}
