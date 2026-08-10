use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The component kinds a profile may include.
///
/// `deny_unknown_fields` on the profile plus this closed enum is what makes a
/// typo'd kind an error rather than a silently narrowed manifest.
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
    Ok(format!(
        "sha-256:{}",
        hex::encode(Sha256::digest(canonical.as_bytes()))
    ))
}
