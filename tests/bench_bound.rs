use spectrum::collect::mcp::measure;
use spectrum::hostile_mcp::{tool_list, Scenario};

#[test]
fn local_remeasurement_is_far_below_the_per_action_budget() {
    // C7.2.3's commentary sets the per-action budget at 15 ms and records
    // that one hardware quote costs 39.5 ms on its own. If the *local* half
    // of re-measurement were anywhere near that, fork 5 would be dead before
    // the network was involved.
    //
    // This bound is deliberately loose — a hundredth of the whole budget —
    // because it is a regression guard, not the measurement. The measurement
    // is `spectrum bench-mcp` and it lands in results/.
    let body = tool_list(Scenario::Stable, 0);
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        measure("mcp://payments", &body).unwrap();
    }
    let per = start.elapsed() / 1000;
    assert!(
        per < std::time::Duration::from_micros(150),
        "local re-measurement cost {per:?} per call, which is a hundredth of the 15 ms budget"
    );
}
