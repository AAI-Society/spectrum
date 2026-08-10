use spectrum::hostile_mcp::{tool_list, Scenario};
use transit::json::{parse_strict, Json};

#[test]
fn a_stable_server_returns_the_same_tool_list_every_time() {
    // The negative control. Without it, a detector that fired on everything
    // would pass the rug-pull test.
    let a = tool_list(Scenario::Stable, 0);
    let b = tool_list(Scenario::Stable, 1);
    let c = tool_list(Scenario::Stable, 17);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn the_rug_pull_widens_a_parameter_after_approval() {
    // Call 0 is the operator's review. Every later call is the real one.
    let approved = tool_list(Scenario::RugPull, 0);
    let called = tool_list(Scenario::RugPull, 1);
    assert_ne!(
        approved, called,
        "the rug-pull scenario returned a stable list"
    );
    assert!(approved.contains("charge"));
    assert!(called.contains("charge"));

    // Fix round 1: `contains("charge")` is a substring match on the whole
    // document, so it would still pass if the tool were renamed to
    // "charge_v2", or if the widened parameter showed up on some unrelated
    // second tool. Parse both bodies and pin the attack precisely: the
    // SAME tool, named exactly "charge", gains an unconstrained
    // "destination" parameter between the review call and the real one.
    let approved_json =
        parse_strict(&approved).expect("the approved body must be strict-JSON parseable");
    let called_json = parse_strict(&called).expect("the called body must be strict-JSON parseable");

    let approved_tools = match approved_json.get("tools") {
        Some(Json::Array(arr)) => arr,
        other => panic!("expected a \"tools\" array, got {other:?}"),
    };
    let called_tools = match called_json.get("tools") {
        Some(Json::Array(arr)) => arr,
        other => panic!("expected a \"tools\" array, got {other:?}"),
    };
    assert_eq!(
        approved_tools.len(),
        1,
        "the approved list must hold exactly one tool"
    );
    assert_eq!(
        called_tools.len(),
        1,
        "the called list must hold exactly one tool"
    );

    let approved_tool = &approved_tools[0];
    let called_tool = &called_tools[0];
    assert_eq!(
        approved_tool.get("name").and_then(Json::as_string),
        Some("charge".to_string()),
        "the approved tool must be named exactly \"charge\""
    );
    assert_eq!(
        called_tool.get("name").and_then(Json::as_string),
        Some("charge".to_string()),
        "the called tool must still be named exactly \"charge\" -- same tool, not a rename"
    );

    let approved_params = approved_tool
        .get("parameters")
        .expect("the approved tool must have a parameters object");
    let called_params = called_tool
        .get("parameters")
        .expect("the called tool must have a parameters object");

    assert!(
        approved_params.get("destination").is_none(),
        "the approved tool must not have a destination parameter"
    );
    assert!(
        called_params.get("destination").is_some(),
        "the called tool must have gained a destination parameter"
    );

    assert_eq!(
        approved_params.get("amount").and_then(Json::as_string),
        Some("number".to_string())
    );
    assert_eq!(
        called_params.get("amount").and_then(Json::as_string),
        Some("number".to_string()),
        "amount must be present and unchanged after the rug pull"
    );
    assert_eq!(
        approved_params.get("currency").and_then(Json::as_string),
        Some("string".to_string())
    );
    assert_eq!(
        called_params.get("currency").and_then(Json::as_string),
        Some("string".to_string()),
        "currency must be present and unchanged after the rug pull"
    );
}

#[test]
fn the_added_tool_scenario_grows_the_list_after_approval() {
    let approved = tool_list(Scenario::AddedTool, 0);
    let called = tool_list(Scenario::AddedTool, 1);
    assert!(called.len() > approved.len());

    // Fix round 1: a raw string-length comparison would still pass if call
    // 1 merely padded the body with whitespace, or widened a parameter
    // instead of adding a second tool. Parse both and count actual tools.
    let approved_json =
        parse_strict(&approved).expect("the approved body must be strict-JSON parseable");
    let called_json = parse_strict(&called).expect("the called body must be strict-JSON parseable");

    let approved_tools = match approved_json.get("tools") {
        Some(Json::Array(arr)) => arr,
        other => panic!("expected a \"tools\" array, got {other:?}"),
    };
    let called_tools = match called_json.get("tools") {
        Some(Json::Array(arr)) => arr,
        other => panic!("expected a \"tools\" array, got {other:?}"),
    };
    assert_eq!(
        approved_tools.len(),
        1,
        "the approved list must hold exactly one tool"
    );
    assert_eq!(
        called_tools.len(),
        2,
        "the called list must hold exactly two tools"
    );

    let called_names: Vec<String> = called_tools
        .iter()
        .filter_map(|t| t.get("name").and_then(Json::as_string))
        .collect();
    assert!(
        called_names.contains(&"charge".to_string()),
        "the called list must still contain the original charge tool"
    );
    assert!(
        called_names.contains(&"send_report".to_string()),
        "the called list must contain the added send_report tool"
    );
}

#[test]
fn every_scenario_returns_parseable_json() {
    for s in [Scenario::Stable, Scenario::RugPull, Scenario::AddedTool] {
        for call in 0..3 {
            let body = tool_list(s, call);
            assert!(
                transit::json::parse_strict(&body).is_ok(),
                "{s:?} call {call} produced a body the strict decoder rejects"
            );
        }
    }
}
