use spectrum::detect::{run, Attack, Outcome};
use spectrum::profile::Profile;

fn profile(name: &str) -> Profile {
    Profile::load(&std::path::Path::new("profiles").join(format!("{name}.toml"))).unwrap()
}

#[test]
fn the_rug_pull_is_invisible_to_a_build_time_manifest() {
    // The baseline's failure, which is the reference implementation's
    // behaviour and therefore the current state of the art.
    assert_eq!(
        run(Attack::ToolRugPull, &profile("build-manifest")).unwrap(),
        Outcome::NotDetected
    );
}

#[test]
fn the_rug_pull_is_detected_by_runtime_tools() {
    // The design's central bet, and the one thing it predicts it catches.
    let o = run(Attack::ToolRugPull, &profile("runtime-tools")).unwrap();
    match o {
        Outcome::Detected { ref component } => assert!(component.contains("payments")),
        other => panic!("expected detection, got {other:?}"),
    }
}

#[test]
fn an_added_tool_is_detected_by_runtime_tools() {
    assert!(matches!(
        run(Attack::AddedTool, &profile("runtime-tools")).unwrap(),
        Outcome::Detected { .. }
    ));
}

#[test]
fn content_poisoning_is_not_detected_by_any_profile() {
    // **This test asserts a negative on purpose and is the row most likely
    // to be deleted by someone tidying the suite.**
    //
    // A retrieved document can be validly sourced, correctly digested,
    // faithfully recorded, and malicious. The register commits to the fact
    // that the chunk entered the context. It says nothing about whether the
    // chunk was adversarial and nothing about whether it steered the
    // decision — that is finding 4, and no mechanism in this programme
    // closes it. Claiming detection here would be exactly the confident
    // wrong answer this tool exists to avoid.
    for p in ["build-manifest", "runtime-tools", "runtime-with-context"] {
        assert_eq!(
            run(Attack::ContentPoisoning, &profile(p)).unwrap(),
            Outcome::NotDetected,
            "{p} claimed to detect content poisoning, which the mechanism cannot do"
        );
    }
}

#[test]
fn a_stable_server_triggers_no_detection_under_any_profile() {
    // The negative control for the detector itself. Without it, a detector
    // that fired unconditionally would pass every test above.
    for p in ["build-manifest", "runtime-tools", "runtime-with-context"] {
        assert_eq!(
            run(Attack::NoAttack, &profile(p)).unwrap(),
            Outcome::NotDetected
        );
    }
}
