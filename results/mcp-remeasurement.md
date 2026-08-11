# MCP re-measurement: local cost

> **Superseded as the headline figure by
> [`mcp-remeasurement-sweep.md`](mcp-remeasurement-sweep.md).** Both runs below
> are one run per cell. A seven-repetition sweep on the same host puts the
> 8-tool figure at a median of 6.197 µs against the 9.635 µs recorded here — a
> factor of 1.55. This file is kept as the record of what was actually run and
> when; the sweep is the number to quote. Neither changes the conclusion, and
> the sweep file says why.

**Host:** Darwin Jims-MacBook-Pro-2.local 25.6.0 Darwin Kernel Version 25.6.0: Sat Jul 11
15:27:04 PDT 2026; root:xnu-12377.161.13~4/RELEASE_ARM64_T6050 arm64 (`uname -a`), CPU
Apple M5 Max (`sysctl -n machdep.cpu.brand_string`).

**Toolchain:** `rustc 1.97.1 (8bab26f4f 2026-07-14)`. Built and run with `cargo run
--release`, i.e. an optimized build, not a debug build.

**What this measures:** the *local* cost of re-measuring an MCP tool list —
canonicalization and hashing via `spectrum::collect::mcp::measure`, run in-process
against a synthetic tool list of the given size. This is the part of re-measurement
that is ours to control.

**What this does not measure:** the round trip to a remote MCP server. That round trip
is the operator's network, not this binary's concern, and it is not measured here.
Reporting the number below as an end-to-end re-measurement cost would be exactly the
extrapolation the design forbids.

**What this does and does not settle:** a negligible local cost, as measured below, is
a *necessary* condition for re-measurement being affordable inside the per-action
budget — it is not a *sufficient* one. Fork 5 is not resolved by this file, because the
network round trip is expected to dominate the total cost and is outside what is
measured here. Whether re-measurement is affordable end-to-end depends on that round
trip, which requires a real deployment to observe, not this benchmark.

Both runs below are real output from `target/release/spectrum bench-mcp` on the host
described above; no figure here is fabricated or interpolated.

## `--tools 8`

```
tools           8
iterations      10000
per measurement 9.635µs
as a share of the 15 ms per-action budget: 0.0642%

LOCAL ONLY. This is canonicalization and hashing. It excludes the round trip to a remote MCP server, which is the operator's network and is not measured here. Do not report this as an end-to-end re-measurement cost.
```

## `--tools 64`

```
tools           64
iterations      10000
per measurement 47.106µs
as a share of the 15 ms per-action budget: 0.3140%

LOCAL ONLY. This is canonicalization and hashing. It excludes the round trip to a remote MCP server, which is the operator's network and is not measured here. Do not report this as an end-to-end re-measurement cost.
```
