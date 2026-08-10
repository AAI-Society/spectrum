use spectrum::hostile_mcp::{tool_list, Scenario};

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
}

#[test]
fn the_added_tool_scenario_grows_the_list_after_approval() {
    let approved = tool_list(Scenario::AddedTool, 0);
    let called = tool_list(Scenario::AddedTool, 1);
    assert!(called.len() > approved.len());
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
