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

#[test]
fn bench_mcp_with_zero_iterations_errors_instead_of_panicking() {
    // `Duration::div(0)` panics, and `--iterations` is a plain u32 with no
    // lower bound, so `bench-mcp --iterations 0` divides by iterations after
    // the timing loop. That must be caught before the divide and routed
    // through the ordinary exit-code-2 error path, not left to panic (exit
    // code 101). This drives the built binary rather than the library so it
    // observes the same failure mode a real invocation would.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_spectrum"))
        .args(["bench-mcp", "--iterations", "0"])
        .output()
        .expect("failed to run spectrum binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 (bad input), got {:?}; stderr: {stderr}",
        output.status.code()
    );
    assert!(!stderr.is_empty(), "expected a non-empty stderr message");
    assert!(
        stderr.to_lowercase().contains("iterations"),
        "expected stderr to mention iterations, got: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "binary panicked instead of erroring cleanly: {stderr}"
    );
}
