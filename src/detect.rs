use crate::collect::mcp::{measure, McpError};
use crate::component::{ComponentRef, Kind, Witness};
use crate::hostile_mcp::{tool_list, Scenario};
use crate::manifest::{Manifest, ManifestError};
use crate::profile::Profile;
use crate::register::{Event, EventKind, Register, RegisterError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// The control: nothing changes.
    NoAttack,
    /// An MCP server widens a tool's parameters between approval and call.
    ToolRugPull,
    /// An MCP server gains a tool between approval and call.
    AddedTool,
    /// A validly sourced, correctly digested, faithfully recorded,
    /// adversarial document enters the context.
    ContentPoisoning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Detected { component: String },
    NotDetected,
}

#[derive(Debug, thiserror::Error)]
pub enum DetectError {
    #[error(transparent)]
    Mcp(#[from] McpError),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Register(#[from] RegisterError),
}

const SERVER: &str = "mcp://payments";

/// Run one attack against one profile and report whether the profile's
/// commitment moved.
///
/// "Detected" here has a precise and narrow meaning: **the register value
/// after the attack differs from the value a verifier would expect**, and the
/// event log names which component moved. It does not mean the attack was
/// prevented, and it does not mean anything was judged malicious.
pub fn run(attack: Attack, p: &Profile) -> Result<Outcome, DetectError> {
    let scenario = match attack {
        Attack::NoAttack | Attack::ContentPoisoning => Scenario::Stable,
        Attack::ToolRugPull => Scenario::RugPull,
        Attack::AddedTool => Scenario::AddedTool,
    };

    // Approval time: the operator reviews, and the static manifest is taken.
    let mut approved = Vec::new();
    if p.includes(Kind::McpServer) {
        approved.push(measure(SERVER, &tool_list(scenario, 0))?);
    }
    let static_manifest = Manifest::new(p, approved.clone());
    let genesis = Register::genesis(&p.id, &static_manifest.digest()?);

    // Call time. A profile that does not include MCP servers re-measures
    // nothing, which is precisely why `build-manifest` cannot see a rug-pull.
    let mut live = Register::genesis(&p.id, &static_manifest.digest()?);
    let mut moved: Option<String> = None;

    if p.includes(Kind::McpServer) {
        let called = measure(SERVER, &tool_list(scenario, 1))?;
        if Some(&called) != approved.first() {
            live.extend(&Event {
                kind: EventKind::ComponentChanged,
                hook: "PRE_CALL_TOOL_INVOCATION".into(),
                component: called.clone(),
            })?;
            moved = Some(called.id.clone());
        }
    }

    if attack == Attack::ContentPoisoning && p.includes(Kind::RetrievedChunk) {
        // The poisoned document is recorded faithfully. Recording it is not
        // detecting it: the register commits that this chunk was present, and
        // a verifier comparing against an expected manifest has no expected
        // value for a chunk retrieved at runtime. Nothing here moves the
        // outcome, and that is the finding.
        live.extend(&Event {
            kind: EventKind::ComponentAdded,
            hook: "CONTEXT_ASSEMBLY".into(),
            component: ComponentRef {
                kind: Kind::RetrievedChunk,
                id: "corpus://policies/refunds#chunk-7".into(),
                content_digest: Some("sha-256:poisoned".into()),
                witness: Witness::Measured,
            },
        })?;
    }

    match moved {
        Some(component) if live.value() != genesis.value() => Ok(Outcome::Detected { component }),
        _ => Ok(Outcome::NotDetected),
    }
}
