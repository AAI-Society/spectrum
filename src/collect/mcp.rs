use crate::component::{ComponentRef, Kind, Witness};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error(
        "the MCP server's tool list was not accepted by the strict decoder: {0}. A tool \
         definition two decoders disagree about is a widened parameter waiting to happen, \
         so this is refused rather than resolved"
    )]
    Decode(#[from] transit::json::JsonError),
    #[error("could not canonicalize the tool list: {0}")]
    Canonicalize(#[from] transit::jcs::JcsError),
}

/// Measure an MCP server by the tool list it actually returned.
///
/// **A URL is not an identity.** The server behind one can change its tool
/// list between two calls with no local signal, which is the attack this
/// whole tool is built to catch. So the component's `content_digest` is over
/// the canonicalized tool list, and the URL is only its name.
///
/// The list is canonicalized with `transit::jcs` and parsed strictly, so a
/// server reformatting its JSON does not read as a composition change, and a
/// server sending duplicate keys is refused rather than silently resolved to
/// whichever key the decoder happens to prefer.
pub fn measure(server_id: &str, tool_list_json: &str) -> Result<ComponentRef, McpError> {
    let parsed = transit::json::parse_strict(tool_list_json)?;
    let canonical = transit::jcs::canonicalize(&parsed)?;
    let d = Sha256::digest(canonical.as_bytes());
    Ok(ComponentRef {
        kind: Kind::McpServer,
        id: server_id.to_string(),
        content_digest: Some(format!("sha-256:{}", hex::encode(d))),
        witness: Witness::Measured,
    })
}
