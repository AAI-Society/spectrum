use spectrum::collect::mcp::measure;
use spectrum::component::{Kind, Witness};
use spectrum::hostile_mcp::{tool_list, Scenario};

#[test]
fn a_measured_tool_list_is_measured_not_declared() {
    let c = measure("mcp://payments", &tool_list(Scenario::Stable, 0)).unwrap();
    assert_eq!(c.kind, Kind::McpServer);
    assert_eq!(c.witness, Witness::Measured);
    assert!(c.content_digest.is_some());
}

#[test]
fn the_same_tool_list_measures_the_same_whatever_its_spelling() {
    // The identity rule, first direction. Two JSON spellings of one tool list
    // must produce one digest, or a server reformatting its response would
    // read as a composition change.
    let a = measure(
        "mcp://x",
        r#"{"tools":[{"name":"charge","parameters":{"amount":"number"}}]}"#,
    )
    .unwrap();
    let b = measure("mcp://x", "{\n  \"tools\" : [ {\n \"parameters\" : {\"amount\":\"number\"},\n \"name\":\"charge\"\n} ]\n}").unwrap();
    assert_eq!(a.content_digest, b.content_digest);
}

#[test]
fn a_widened_parameter_changes_the_measurement() {
    // The identity rule, second direction, and the attack.
    let approved = measure("mcp://payments", &tool_list(Scenario::RugPull, 0)).unwrap();
    let called = measure("mcp://payments", &tool_list(Scenario::RugPull, 1)).unwrap();
    assert_ne!(approved.content_digest, called.content_digest);
}

#[test]
fn the_server_id_is_part_of_the_component_identity() {
    let a = measure("mcp://payments", &tool_list(Scenario::Stable, 0)).unwrap();
    let b = measure("mcp://other", &tool_list(Scenario::Stable, 0)).unwrap();
    assert_eq!(
        a.content_digest, b.content_digest,
        "same tools, same content digest"
    );
    assert_ne!(
        a.id, b.id,
        "a URL is not an identity, but it is part of one"
    );
}

#[test]
fn a_tool_list_with_duplicate_keys_is_refused() {
    // `transit`'s central finding: two decoders disagree about what a
    // document with duplicate keys means. A tool definition is exactly the
    // place where that disagreement becomes a widened parameter, so this
    // crate refuses rather than picking a winner.
    let r = measure(
        "mcp://x",
        r#"{"tools":[{"name":"charge","name":"transfer"}]}"#,
    );
    assert!(r.is_err());
}

#[test]
fn malformed_responses_error_rather_than_panic() {
    for bad in ["", "{", "null", "[]", r#"{"tools":3}"#] {
        let _ = measure("mcp://x", bad);
    }
}
