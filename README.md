<p align="center">
  <img src="assets/banner.svg" alt="spectrum — a digest is a change detector, not a checkable claim" width="100%">
</p>

<p align="center">
  <img alt="Rust 2021" src="https://img.shields.io/badge/rust-2021%20%C2%B7%201.90%2B-CFFF04?style=flat-square&labelColor=0A0A0A">
  <img alt="76 tests passing" src="https://img.shields.io/badge/tests-76%20passing-CFFF04?style=flat-square&labelColor=0A0A0A">
  <img alt="Apache 2.0" src="https://img.shields.io/badge/licence-Apache--2.0-CFFF04?style=flat-square&labelColor=0A0A0A">
  <img alt="paper included" src="https://img.shields.io/badge/paper-included-CFFF04?style=flat-square&labelColor=0A0A0A">
</p>

---

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
  what composition means, and the library's `diff` refuses to compare
  digests taken under different profiles rather than reporting a difference.
- **What is real:** the component, manifest, event and register digests, each
  domain-tagged and length-prefixed; JCS canonicalization (`transit::jcs`, not
  a second implementation); the projection from CycloneDX; the composition
  register and its one-way replay; MCP tool-list re-measurement against the
  local hostile server; and the detection harness — which executes real
  substitutions against real profiles. **Nothing here digests a file or a
  blob.** Every digest in this crate is over a `ComponentRef`, a manifest, an
  event, or an MCP tool list; a `content_digest` that names file bytes arrives
  already computed, from the CycloneDX document, and this tool copies it.
- **What is modelled, and what is only intended:** **influence is absent, not
  stubbed** — there is no `influence` field and no API that could produce one,
  because a stub here would be mistaken for the C1.2.2 link (which retrieved
  content influenced a decision) that nobody can compute. That absence is
  enforced today, by there being no such field. Two further properties are
  *intended discipline, not enforced behaviour*: that hosted-model weight
  identity is `Declared` and never `Measured` — no measurement of a hosted
  model's weights exists from outside the provider — and that training-data
  lineage is `Opaque`. **The code does not enforce either.** There is no model
  collector (`src/collect/mod.rs` declares only `mcp`), `Witness::Opaque` is
  constructed nowhere in the crate, no test asserts the property, and
  `ComponentRef` is a plain public struct, so a caller can write `Measured`
  against a hosted model today and nothing objects. These are the rules the
  model collector of [build step 5](#the-eight-open-forks) must be built to,
  written down before it exists rather than after.
- **What it detects and does not:** the tool-definition rug-pull, yes.
  Content poisoning, **no** — and `tests/detect.rs` asserts that negative on
  purpose, against every shipped profile, so that no future change can make
  the negative quietly pass.
- **Nothing verifies a digest.** No infrastructure publishes expected
  AgBOMs anywhere in this programme, so the library's `diff` against a prior
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

**What the harness is, and is not.** It encodes the argument rather than
discovering it: in `detect::run`, `Outcome::Detected` is reachable only when
`moved` is set, and `moved` is set only inside the `p.includes(Kind::McpServer)`
branch, so a content attack cannot reach `Detected` by construction of the
predicate. That is deliberate and the code's own doc comment gives the reason —
a verifier has no expected value for a chunk retrieved at runtime, so a
register that moved because a chunk arrived is a record of arrival and not a
detection. The negative is correct and the argument for it is sound; what the
harness does is execute that argument against real profiles and real
substitutions, not independently measure its way to it. Read the result as a
worked consequence of the design, not as an experiment that could have come
out the other way on the content row.

If that finding holds — and the harness in this repository says it does —
the honest conclusion for P10 is narrower than "AgBOM is inventory" and more
useful than "AgBOM is a security control": **a running composition
commitment is a change-detection control over the capability surface, and an
inventory control over content.** That moves the threat model on
tool-and-plugin substitution, not on the memory- and context-poisoning rows
P10 hopes for.

## Measured cost of re-measurement

`results/mcp-remeasurement-sweep.md` is the authoritative reading — seven
repetitions per size, 10,000 iterations each, sizes 1 to 256 by doubling.
Neither results file is regenerated by CI, because a wall-clock timing is never
byte-stable and a diff guard on it would fail on every run for reasons that
have nothing to do with a code change.

| tools | per measurement (median of 7) | share of the 15 ms per-action budget |
| --- | --- | --- |
| 8 | 6.197 µs | 0.0413% |
| 64 | 43.310 µs | 0.2887% |
| 256 | 177.259 µs | 1.1817% |

Cost is linear in list size at roughly **0.69 µs per tool**; a list would need
around 22,000 entries before the local half consumed the whole budget.

**One run is not a benchmark, and we got this wrong first time.**
`results/mcp-remeasurement.md` reports 9.635 µs at 8 tools from a single run.
The median of seven repetitions on the same host, same toolchain, same binary,
is 6.197 µs — a factor of 1.55. Within a sweep the spread is 1.9%–10.9%;
across sweeps at the small end it is not small. The older file is kept as the
record of what was actually run, with a pointer to this one. Both support the
same conclusion, which is why nothing downstream changed.

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
| `spectrum compose --profile <p> --bom <b>` | project a CycloneDX document under a profile and print the manifest digest, plus the gaps in it |
| `spectrum coverage --profile <p>` | print what this profile does not capture |
| `spectrum detect --scenario <name>` | run one attack against every profile and report what each catches |
| `spectrum serve-hostile --scenario <name> [--addr <addr>]` | serve one hostile-MCP scenario over HTTP, for manual exploration only |

`compose` **warns rather than refuses** when a profile declares a kind no
collector fills. `build-manifest` declares four kinds and the CycloneDX
projection supplies two of them, so every run prints the declared-but-absent
kinds under the digest, and the CycloneDX `type` values the projection had no
kind for, with counts. The design's fail-loud table asks for a refusal here;
refusing would make this step unusable, since four of the eight kinds have no
collector until step 5. The compromise is that the digest still prints and
never prints alone.

`detect` and `serve-hostile` accept `none`/`tool-rug-pull`/`added-tool`/`content-poisoning`
and `stable`/`rug-pull`/`added-tool` respectively; an unrecognized name exits
`2` rather than panicking, since both are untrusted input.

`serve-hostile` binds a real socket and is exercised by a human running the
binary, never by the test suite — every test in this repository runs
offline, with no network, no credentials, and no socket.

## The paper

[**A Digest Is a Change Detector, Not a Claim: Composition Commitments for
Agent Systems, and the Attack Class They Cannot See**](paper/main.pdf) —
readable in the browser, no toolchain needed.

**Draft, circulated for co-author review.** It argues that a bill of materials
taken at task start is a *constant function* against the substitution it most
needs to catch, that the fix — a running commitment extended at
composition-change points — works for the capability surface and does nothing
for content, and that the honest answer for P10 is therefore narrower than
"AgBOM is inventory" and more useful than "AgBOM is a security control."

It also argues against its own table. §5.3 says plainly that the
content-poisoning row could not have come out the other way, because
`Outcome::Detected` is unreachable outside the `Kind::McpServer` branch by
construction of the predicate — so that row is a worked consequence of the
design and not an experiment. That section is the one most likely to be cut for
flow, and `paper/README.md` records that it must not be.

Its detection figures are real output from `spectrum detect`; its cost figures
come from `results/mcp-remeasurement-sweep.json`, one machine, seven
repetitions per cell. The number that would actually decide fork 5 — the
network round trip — is not in the paper, and §6.3 says so.

## The eight open forks

`spectrum` implements the design at
[`../poc-research-roadmap/docs/superpowers/specs/2026-08-10-spectrum-design.md`](../poc-research-roadmap/docs/superpowers/specs/2026-08-10-spectrum-design.md),
which leaves eight forks open. **This work resolves one of them, and it is
not the case that the tool takes the recommended branch on each.** The design
is a draft with its forks deliberately unresolved, and nothing here closes one
that its own build order puts after this point. Read the design for the full
argument on each side; what this repository does to each is:

| fork | what this work does |
| --- | --- |
| 1 SPDX support | **Diverges from the recommendation.** The design recommends **(b)**, read both and emit CycloneDX. This tool reads CycloneDX only, which is branch **(a)**. That is what the plan says and what the code does; a second reader is a second identity derivation to keep in agreement, and it is not built. |
| 2 witness class in the digest | **Resolved, as recommended: (a), inside.** Pinned in both directions by `tests/component_identity.rs` — `the_witness_class_is_inside_the_digest` asserts a `Declared` component does not hash like a `Measured` one, and `every_witness_tag_is_distinct` asserts no two witness classes can hash alike by a copy-paste. Reversing this fork is a visible test change. |
| 5 MCP server identity | **Instrumented, not resolved.** `runtime-tools` implements (b), re-measurement, and the detection harness and the benchmark exist to *measure* what (b) costs and catches. A negligible local cost is a necessary and not a sufficient condition; the network round trip decides this fork and requires a real deployment. |
| 7 context linkability | **Half built, and see below.** |
| 3, 4, 6, 8 | **Untouched.** No branch is taken because nothing here reaches the decision: no field of this tool is emitted as an `agbom_digest` (3), there is no prompt collector (4), no retriever (6), and no delegation (8). The collectors and the `ephemeris` integration that would decide them are the design's build step 5, which has not begun. Saying this tool "takes the recommended branch" on them would be an empty claim about code that does not exist. |

One thing bearing on fork 3 is worth naming without closing it: `register.rs`'s
`a_static_deployment_never_advances_its_register` pins the property the
recommendation rests on — that (b) degenerates exactly to (a) when nothing
changes. That is evidence for the recommendation, not a resolution of it,
because nothing in this tool fills the field the fork is about.

1. **SPDX support** — read CycloneDX only, read both and emit CycloneDX, or
   read and emit both. The design recommends reading both, emitting only
   CycloneDX. **This tool reads CycloneDX only** — the first branch, not the
   recommended one. An operator with an SPDX document cannot use `compose`.
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
   the leak documented *and* a `--profile unlinkable` that salts, or
   reference-only (leaks less, detects no tampering). The design recommends
   the second, with the leak printed by `spectrum coverage` rather than
   buried in a README. **Only half of that branch is built.**
   `runtime-with-context` is linkable and `spectrum coverage` prints the
   leak, keyed on the `retrieved_chunk` kind rather than on a profile name.
   **The salting half does not exist**: there is no `unlinkable` profile, no
   salt, and no code that could produce one, so an operator who reads the
   warning has nothing to switch to. This fork is unresolved and the
   benchmark above does not resolve it either.
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
