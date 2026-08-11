# MCP re-measurement: cost against list size, with repetitions

**This file is the authoritative reading of `mcp-remeasurement-sweep.json`.** If
it and the paper's Table 3 ever disagree, this file wins.

**Host:** Apple M5 Max, macOS (Darwin 25.6.0), `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
release build. The same host as `mcp-remeasurement.md`.

**Method:** `target/release/spectrum bench-mcp --iterations 10000 --tools N`, seven
repetitions per size, sizes 1 through 256 by doubling. Each repetition is 10,000
calls to `spectrum::collect::mcp::measure` against a synthetic tool list built
before the timing loop, with the binary's own 100-iteration warm-up outside it.
Spread is `(max - min) / min` across the seven repetitions.

| tools | min (µs) | median (µs) | max (µs) | spread | median as % of 15 ms |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1.250 | 1.277 | 1.386 | 10.9% | 0.0085% |
| 2 | 1.983 | 2.045 | 2.149 | 8.4% | 0.0136% |
| 4 | 3.281 | 3.346 | 3.430 | 4.5% | 0.0223% |
| 8 | 6.047 | 6.197 | 6.364 | 5.2% | 0.0413% |
| 16 | 11.404 | 11.587 | 11.861 | 4.0% | 0.0772% |
| 32 | 22.006 | 22.494 | 22.613 | 2.8% | 0.1500% |
| 64 | 43.083 | 43.310 | 43.921 | 1.9% | 0.2887% |
| 128 | 84.491 | 84.945 | 87.629 | 3.7% | 0.5663% |
| 256 | 176.164 | 177.259 | 182.886 | 3.8% | 1.1817% |

## What it shows

Cost is linear in list size. A least-squares fit over the nine sizes gives
**0.685 µs per tool** with a fixed cost of about 0.27 µs; the marginal cost
between the 64- and 256-tool rows is 0.698 µs per tool. The fit under-predicts
the small end by about 7%, so quote the per-tool figure and not the intercept.

Extrapolated, a tool list would need roughly **22,000 entries** before local
re-measurement consumed the whole 15 ms per-action budget. At 256 tools it
consumes 1.2% of it.

## Why this file exists: one run is not a benchmark

`mcp-remeasurement.md` reports **9.635 µs** at 8 tools from a single run. The
median of seven repetitions on the same host, same toolchain, same binary, is
**6.197 µs**. A single standalone run taken while preparing the paper landed at
7.021 µs, between the two.

That is a factor of 1.55 between two measurements of the same quantity on the
same machine. *Within* a sweep the spread is 1.9%–10.9% and tightens as the work
per iteration grows, which is ordinary timing noise. *Across* sweeps, at the
small end, it is not small.

The ordering — first-run slowest, sweep-median fastest — is consistent with a
warm-up effect the in-process 100-iteration warm-up does not cover: that loop
warms the allocator and the branch predictors and does nothing about CPU
frequency or scheduler placement. **That is a hypothesis, not a finding.** It was
not tested, and testing it means pinning frequency and affinity, which was not
done.

`mcp-remeasurement.md` is not wrong as a record of one run. It overstates the
steady-state cost by about half, and both figures support the same conclusion —
the local cost is two to three orders of magnitude below the budget — which is
why this changed no argument. It would have changed one if the true figure had
been near the budget.

## What this still does not settle

The same limit as `mcp-remeasurement.md`, and it is not weakened by having more
repetitions: **this is the local half only.** Canonicalization and hashing. It
excludes the round trip to a remote MCP server, which is the operator's network
and is expected to dominate the total. A negligible local cost is a *necessary*
condition for re-measurement being affordable inside the per-action budget, not a
*sufficient* one. **Fork 5 is not resolved by this file.**
