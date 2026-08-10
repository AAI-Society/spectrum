# spectrum

A composition commitment for agent systems, and the empirical answer to
[P10](https://github.com/AAI-Society/poc-research-roadmap/blob/master/papers/P10-agbom.md)'s
question: can an Agent Bill of Materials be a running, verifiable commitment,
and does it catch anything a static one does not?

## What this tool refuses to do

- **`spectrum` does not define an Agent Bill of Materials.** It defines a
  digest over a *declared* component set, ships three such sets as named
  falsifiable hypotheses (`build-manifest`, `runtime-tools`,
  `runtime-with-context`), and reports what each fails to capture. Which set
  is right is [P10](https://github.com/AAI-Society/poc-research-roadmap/blob/master/papers/P10-agbom.md)'s
  question, not this tool's answer.
- **There is no default profile.** `--profile` is required on every command
  that composes a manifest, so no invocation of this tool can produce an
  unmarked "the AgBOM." A digest only ever exists under a stated theory of
  what composition means, and `spectrum diff` refuses to compare digests
  taken under different profiles rather than reporting a difference.
- **What is real:** blob and file digests, JCS canonicalization
  (`transit::jcs`, not a second implementation), the projection from
  CycloneDX, the composition register and its one-way replay, MCP tool-list
  re-measurement against the local hostile server, and the detection
  harness — which executes real substitutions against real profiles, not a
  simulation of them.
- **What is modelled:** hosted-model weight identity is `Declared` and
  **cannot** be `Measured` — no measurement of a hosted model's weights
  exists from outside the provider. Training-data lineage is permanently
  `Opaque`. **Influence is absent, not stubbed.** There is no `influence`
  field and no API that could produce one, because a stub here would be
  mistaken for the C1.2.2 link — which retrieved content influenced a
  decision — that nobody can compute.
- **What it detects and does not:** the tool-definition rug-pull, yes.
  Content poisoning, **no** — and `tests/detect.rs` asserts that negative on
  purpose, against every shipped profile, so that no future change can make
  the negative quietly pass.
- **Nothing verifies a digest.** No infrastructure publishes expected
  AgBOMs anywhere in this programme, so `spectrum diff` against a prior
  digest of the same deployment is the only operation available. A digest is
  a change detector between two records of the same system, not a checkable
  claim about one record in isolation.
- **`runtime-with-context` is a privacy regression.** A content-derived
  chunk digest is a cross-session, cross-agent fingerprint of what was read:
  two evidence records carrying the same chunk digest prove both agents read
  the same document. `spectrum coverage --profile profiles/runtime-with-context.toml`
  prints this rather than burying it in prose. OPEN FORK 7 is unresolved.

## What it is

Three profiles, each a named, falsifiable hypothesis about what "composition"
means for an agent system:

| profile | includes | the claim it makes | what it cannot capture |
| --- | --- | --- | --- |
| `build-manifest` | model identity, system prompt, tool definitions, code and dependency SBOM | composition is fixed at task start | everything that changes after |
| `runtime-tools` | the above, plus MCP server tool-list re-measurement and sub-agent roster | the *capability surface* changes at runtime and is small enough to re-measure | retrieved context, memory, whether a component is adversarial |
| `runtime-with-context` | the above, plus retrieved-chunk and memory-read references | everything that enters the context is part of composition | which of it mattered, and it costs a linkability leak (fork 7) |

`build-manifest` reproduces the reference implementation's behaviour. It is
shipped so the cost and detection power of the other two are measured
*against* it, not asserted over it.

A **profile** cannot claim complete coverage: `coverage` is a required,
non-empty field, and a profile that omits it fails to load. Every
`ComponentRef` carries a **witness class** — `Measured`, `Attested`,
`Declared`, or `Opaque` — inside its digest, so an operator's typed version
string and a hash of real bytes can never render identically.

## The running commitment

```
C₀ = H(profile_id ‖ static_manifest_digest)          genesis
Cₙ = H(Cₙ₋₁ ‖ H(event))                              extension, one-way
```

The register is a PCR-style running commitment over composition changes:
32 bytes regardless of how much composition has moved. There is no setter,
no `reset`, and no constructor other than `genesis` — a "reset composition"
call would be a rewrite of history. It solves the *record size* problem
completely and the *verification* problem not at all: checking a register
requires its event log, so somebody must store that log. This tool does not;
that is `ephemeris`'s job.

## The detection harness, and the honest finding

P10's method step 4 asks whether the design lets a verifier detect
composition attacks from evidence alone. `spectrum detect --scenario <name>`
runs one attack against every shipped profile and reports, per profile,
`DETECTED` (naming the component that moved) or `not detected`. Exit code is
`0` if any profile detected the attack, `1` if none did.

**Detected: the tool-definition rug-pull.** An MCP server presents a narrow
tool schema at operator approval and a widened one at call time. Under
`build-manifest` the digest is identical before and after, because the
manifest was taken once, at task start. Under `runtime-tools` the register
diverges at the call, and the event names the component that changed. This
is detectable because a tool definition is small, local, and re-readable.

**Not detected: content poisoning.** A retrieved document that is validly
sourced, correctly digested, faithfully recorded, and malicious. The
register commits to the fact that the chunk entered the context. It says
nothing about whether the chunk was adversarial, and nothing about whether
it steered the decision. `tests/detect.rs::content_poisoning_is_not_detected_by_any_profile`
asserts this negative against all three profiles, on purpose, and the test's
own comment says it is the row most likely to be deleted by someone tidying
the suite. It should not be.

If that finding holds — and the harness in this repository says it does —
the honest conclusion for P10 is narrower than "AgBOM is inventory" and more
useful than "AgBOM is a security control": **a running composition
commitment is a change-detection control over the capability surface, and an
inventory control over content.** That moves the threat model on
tool-and-plugin substitution, not on the memory- and context-poisoning rows
P10 hopes for.

## Measured cost of re-measurement

`results/mcp-remeasurement.md` is a committed artifact of one real, measured
run on the host and toolchain it names — not regenerated by CI, because a
wall-clock timing is never byte-stable and a diff guard on it would fail on
every run for reasons that have nothing to do with a code change.

| tools | per measurement | share of the 15 ms per-action budget |
| --- | --- | --- |
| 8 | 9.635 µs | 0.0642% |
| 64 | 47.106 µs | 0.3140% |

Local only, Apple M5 Max. This is canonicalization and hashing —
`spectrum::collect::mcp::measure` run in-process against a synthetic tool
list. It excludes the round trip to a remote MCP server, which is the
operator's network and is not measured here. **Fork 5 is not resolved by
this number.** A negligible local cost is a *necessary* condition for
re-measurement being affordable inside the per-action budget, not a
*sufficient* one; the network round trip is expected to dominate the total
cost and requires a real deployment to observe.

## Command reference

| command | purpose |
| --- | --- |
| `spectrum bench-mcp [--iterations N] [--tools N]` | local cost of re-measuring an MCP tool list |
| `spectrum compose --profile <p> --bom <b>` | project a CycloneDX document under a profile and print the manifest digest |
| `spectrum coverage --profile <p>` | print what this profile does not capture |
| `spectrum detect --scenario <name>` | run one attack against every profile and report what each catches |
| `spectrum serve-hostile --scenario <name> [--addr <addr>]` | serve one hostile-MCP scenario over HTTP, for manual exploration only |

`detect` and `serve-hostile` accept `none`/`tool-rug-pull`/`added-tool`/`content-poisoning`
and `stable`/`rug-pull`/`added-tool` respectively; an unrecognized name exits
`2` rather than panicking, since both are untrusted input.

`serve-hostile` binds a real socket and is exercised by a human running the
binary, never by the test suite — every test in this repository runs
offline, with no network, no credentials, and no socket.

## The eight open forks

`spectrum` implements the design at
[`../poc-research-roadmap/docs/superpowers/specs/2026-08-10-spectrum-design.md`](../poc-research-roadmap/docs/superpowers/specs/2026-08-10-spectrum-design.md),
which leaves eight forks open. This tool takes the design's recommended
branch on each and does not resolve any of them further; read the design for
the full argument on each side.

1. **SPDX support** — read CycloneDX only, read both and emit CycloneDX, or
   read and emit both. The design recommends reading both, emitting only
   CycloneDX.
2. **Does the witness class contribute to the digest?** Inside (upgrading a
   component's evidence changes the digest) or outside (a `Declared` and a
   `Measured` component with the same claimed identity would hash
   identically). The design recommends inside.
3. **What value goes in `agbom_digest`?** The static root only, the register
   value with the static root as genesis, or a combination of both. The
   design recommends the register value.
4. **Prompt identity** — the rendered instance, the template plus a digest
   of variable names, or both as separate components. The design recommends
   both.
5. **MCP server identity** — endpoint URL plus declared version, a JCS
   digest of the tool list as returned and re-measured, or refuse and mark
   `Opaque`. The design recommends re-measurement, and calls it the design's
   central bet; this is what `runtime-tools` implements and what the
   detection harness above tests.
6. **Retrieved-context granularity** — per-document, per-chunk-as-retrieved,
   or per-token-span (which requires instrumenting the model and is not
   obtainable from outside it). The design recommends per-chunk.
7. **Context digests are a cross-session fingerprint** — unlinkable by
   default (salted, kills cross-session `diff`), linkable by default with
   the leak documented, or reference-only (leaks less, detects no
   tampering). The design recommends linkable by default, with the leak
   printed by `spectrum coverage` rather than buried in a README. This fork
   is unresolved and the benchmark above does not resolve it either.
8. **Cross-agent composition** — does a delegator's manifest link to a
   delegate's manifest root, or recursively include the delegate's full
   composition? The design recommends linking only, since a delegate's
   composition is not frozen at delegation time and a transitive digest
   would be stale the moment it is computed.

## Building and testing

```
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Rust edition 2021, rust-version 1.90, `#![forbid(unsafe_code)]`. Library
errors are `thiserror`; the binary's are `anyhow`. Every test in this
repository runs offline.
