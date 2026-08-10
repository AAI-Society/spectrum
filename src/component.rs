use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The component kinds a profile may include.
///
/// `deny_unknown_fields` on the profile plus this closed enum is what makes a
/// typo'd kind an error rather than a silently narrowed manifest.
///
/// **The declaration order of these variants is load-bearing.** `ComponentRef`
/// derives `Ord` with `kind` first, so the derived `Ord` on this enum — which
/// is variant declaration order — decides the order
/// [`Manifest::new`](crate::manifest::Manifest::new) sorts into and therefore
/// the order [`Manifest::digest`](crate::manifest::Manifest::digest)
/// concatenates component digests in. Reordering these variants looks
/// cosmetic and changes the manifest digest of every manifest holding more
/// than one kind. Pinning the [`Kind::tag`] strings does not protect against
/// this; the strings are what each variant hashes as, not what order it
/// hashes in. `tests/manifest.rs::the_manifest_digest_of_a_two_kind_manifest_is_pinned`
/// exists to turn a reorder into a test failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Model,
    SystemPrompt,
    ToolDefinition,
    McpServer,
    CodeDependency,
    RetrievedChunk,
    MemoryRead,
    SubAgent,
}

impl Kind {
    /// The wire spelling, written here rather than derived, because this
    /// string is inside a digest and a `rename_all` change would silently
    /// alter every digest this tool has ever produced.
    pub fn tag(&self) -> &'static str {
        match self {
            Kind::Model => "model",
            Kind::SystemPrompt => "system_prompt",
            Kind::ToolDefinition => "tool_definition",
            Kind::McpServer => "mcp_server",
            Kind::CodeDependency => "code_dependency",
            Kind::RetrievedChunk => "retrieved_chunk",
            Kind::MemoryRead => "memory_read",
            Kind::SubAgent => "sub_agent",
        }
    }
}

/// How we know what we claim about a component.
///
/// This is inside the digest (OPEN FORK 2). The cost is that improving
/// instrumentation — moving a component from `Declared` to `Measured` —
/// changes the digest even though the system did not change. That cost is
/// accepted because the alternative is a digest in which an operator's typed
/// version string is indistinguishable from measured bytes.
///
/// **The declaration order of these variants is load-bearing**, for the same
/// reason as [`Kind`]'s: `witness` is the last field of `ComponentRef`'s
/// derived `Ord`, so it breaks ties between two components sharing a kind, id
/// and content digest, and that tie-break is the concatenation order
/// [`Manifest::digest`](crate::manifest::Manifest::digest) commits to.
/// Reordering these variants changes those manifests' digests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Witness {
    /// We read the bytes and hashed them.
    Measured,
    /// A third party signed a statement about it.
    Attested,
    /// The operator asserted it and nothing checked.
    Declared,
    /// A component of this kind is present and cannot be named. This is a
    /// positive statement, never the result of a failure.
    Opaque,
}

impl Witness {
    pub fn tag(&self) -> &'static str {
        match self {
            Witness::Measured => "measured",
            Witness::Attested => "attested",
            Witness::Declared => "declared",
            Witness::Opaque => "opaque",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRef {
    pub kind: Kind,
    /// A `purl` where one exists, otherwise a stable local name. Compared as
    /// opaque bytes; see the test.
    pub id: String,
    /// Tagged digest of the component's bytes, e.g. `sha-256:…`. `None` where
    /// the witness class is `Declared` or `Opaque`.
    pub content_digest: Option<String>,
    pub witness: Witness,
}

#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("could not canonicalize the component: {0}")]
    Canonicalize(#[from] transit::jcs::JcsError),
    #[error("the component did not survive the strict decoder: {0}")]
    Decode(#[from] transit::json::JsonError),
    #[error("could not serialize the component: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// The identity of one component.
///
/// **Every field of `ComponentRef` contributes, and the derivation is spelled
/// out here rather than left to `derive`.** Anything feeding this digest is
/// load-bearing for every comparison downstream — `diff`, the manifest root,
/// and the register — and specifying it casually produces answers that look
/// right. That was the root cause of four of `parallax`'s five criticals.
///
/// An absent `content_digest` is rendered as an explicit `null` rather than
/// omitted, so "we have no digest" and "we did not mention a digest" cannot
/// collide.
///
/// **`H("spectrum-component-v1" ‖ len(canonical) ‖ canonical)`.** The domain
/// tag is here so a component digest cannot collide with a SHA-256 taken over
/// the JCS form of anything else — most immediately
/// [`crate::collect::mcp::measure`], which is the other untagged JCS hash in
/// this crate and shares this namespace. The tag is hashed *before* the
/// canonical bytes rather than carried as a `_domain` key inside the JSON
/// object, so it can never be shadowed by a future `ComponentRef` field that
/// happens to be named the same thing; a tag inside the JCS input is just
/// another field, and another field is exactly what it must be distinguishable
/// from.
pub fn digest(c: &ComponentRef) -> Result<String, ComponentError> {
    let v = serde_json::json!({
        "kind": c.kind.tag(),
        "id": c.id,
        "content_digest": c.content_digest,
        "witness": c.witness.tag(),
    });
    let text = serde_json::to_string(&v)?;
    let parsed = transit::json::parse_strict(&text)?;
    let canonical = transit::jcs::canonicalize(&parsed)?;
    let mut h = Sha256::new();
    h.update(b"spectrum-component-v1");
    h.update((canonical.len() as u64).to_be_bytes());
    h.update(canonical.as_bytes());
    Ok(format!("sha-256:{}", hex::encode(h.finalize())))
}
