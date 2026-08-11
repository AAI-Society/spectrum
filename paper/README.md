# A Digest Is a Change Detector, Not a Claim: arXiv Preprint

LaTeX source for *A Digest Is a Change Detector, Not a Claim: Composition
Commitments for Agent Systems, and the Attack Class They Cannot See*.

**Status: working draft for co-author review — not yet submitted.** All sections
are written; there is no placeholder section. §5 reports the detection harness
and §6 the benchmark; §7 ("What We Still Don't Know") states what remains open
after both, including the fact that the number which would actually decide
P10's fork 5 — the network round trip — is not in this paper at all.

**Prose style.** The expository register shared with the sibling papers: first
person, concrete case before abstraction, ordinary words in place of jargon, and
ignorance stated loudly rather than hedged. Specifically —

- **The attack is demonstrated before it is formalised.** §1 walks the tool
  rug-pull through in English, with the two literal JSON bodies, and §3 opens by
  admitting the symbols "will not make anything truer than the English already
  did."
- **The central claim is stated in its blunt form first.** The claim that a
  build-time bill of materials is *a constant function* against the substitution
  it most needs to catch is meant to land hard, and §1 defends it rather than
  softening it to "may not capture."
- **The weakest result is labelled as weak by the section that reports it.**
  §5.3 exists to say that the content-poisoning row of Table 2 could not have
  come out the other way, because `Detected` is unreachable outside the
  MCP-server branch by construction of the predicate. The tidy version of that
  story would present the row as an experimental finding. It is not one.
- **The correction to our own numbers is reported as a correction.** §6.2 says
  that the previously committed 9.635 µs figure overstates the steady-state cost
  by about half, gives the sweep median that supersedes it, and says why this
  changed no argument — and would have changed one if the figure had been near
  the budget.

Section titles stay plain declaratives ("The Problem", "Does It Actually
Work?", "What We Still Don't Know").

**The ordering is load-bearing.** §1 opens with the timing problem (the
rug-pull) and only then reaches the harder one (nobody has agreed what belongs
on the bill). Do not promote §1.1 to the front in editing: the schema question
is the more important of the two and the less concrete, and leading with it
makes the paper read as position rather than result. The rug-pull earns the
right to raise it.

**Where the numbers come from.**

- **Table 2 (detection)** is real output from `spectrum detect --scenario
  <name>`, run against all four scenarios and reformatted into a matrix. No cell
  is asserted.
- **Table 3 (cost)** is read out of `../results/mcp-remeasurement-sweep.json`,
  produced by `spectrum bench-mcp` — seven repetitions per size, 10,000
  iterations each, sizes 1 to 256 by doubling. One host: Apple M5 Max, macOS,
  `rustc 1.97.1`, release build.
- `../results/mcp-remeasurement-sweep.md` is the authoritative reading of those
  numbers. **If §6 and that file ever disagree, that file wins.**

Three things §6 is required to keep saying:

- **The measurement is local only.** Canonicalization and hashing. It excludes
  the round trip to a remote MCP server. If an edit introduces "we measure the
  cost of re-measurement" without that qualifier, revert it.
- **Fork 5 is not resolved.** A negligible local cost is a *necessary* condition
  for affordability inside the per-action budget, never a *sufficient* one.
- **The linear fit's intercept is not a quotable figure.** The fit is dominated
  by the large sizes and under-predicts the small end by about 7%. Quote
  0.685–0.698 µs per tool; do not quote 0.27 µs as "the fixed cost."

## Build

```bash
cd paper
tectonic main.tex
```

Output: `main.pdf`, which is committed to the repository so it can be read
without building anything.

**It is not diff-gated in CI, and that is deliberate.** Tectonic stamps a
`CreationDate` into the PDF, so two builds of identical sources differ
byte-for-byte and a `git diff --exit-code` check would fail on every run.
Keeping the committed PDF current is a manual step: if you change `main.tex`,
rebuild and commit the PDF in the same commit.

`-Z shell-escape` is *not* required. This preamble omits `minted` and uses
`listings`, so there is no Python dependency.

## Before Submission — Required Steps

1. **Co-author consent.** The author list is deliberately short at this stage.
   Add co-authors only on explicit consent and substantive contribution; the
   title-page footnote states this.

2. **Citation verification.** Every entry in `references.bib` must be confirmed
   against the primary source before submission. **None has been confirmed
   yet**, and entries carrying a `[verify]` note name the specific field that
   needs checking. Check with particular care: the ISO/IEC 5962 designation on
   SPDX, the Federal Register citation for EO 14028, the Cyber Resilience Act's
   regulation number and the annex carrying the bill-of-materials obligation,
   the page ranges on Merkle (1980), in-toto, Ohm (2020) and Greshake (2023),
   and the TPM 2.0 part and revision for the PCR extend operation cited in §2.
   The CycloneDX, SLSA, MCP and OWASP entries need the version or edition
   current at submission time, not the one that happened to be current while
   drafting.

3. **The 15 ms budget and the 39.5 ms quote.** §6 takes both from the
   Proof-of-Control standard's C7.2.3 commentary, at second hand, via a comment
   in `tests/bench_bound.rs`. Before submission, cite the clause directly or
   downgrade both to "the figure the implementation's test suite records."
   Do not leave a budget the whole cost argument is scaled against resting on a
   code comment.

4. **The warm-up hypothesis in §6.2.** The paper says the first-run-slowest
   ordering is *consistent with* a warm-up effect and explicitly labels that a
   hypothesis. Keep it labelled. Testing it properly means pinning CPU frequency
   and affinity and re-running; if somebody does that, the result replaces the
   hypothesis. Absent that, the hedge stays.

5. **Re-derive every figure in §6 against `../results/*.json`.** The tables are
   typed into the `.tex`, not generated, so they can drift from the JSON
   silently. Re-check each cell of Table 3, the two per-tool slopes, the 7%
   under-prediction, the ~22,000-entry extrapolation, and the 1.55 factor in
   §6.2. Numbers that cannot be re-derived from the committed JSON must be cut,
   not rounded.

6. **Do not let §5.3 be tidied away.** It is the section an editor is most
   likely to cut for flow, because it argues against the paper's own table. Its
   claim — that the content row is a worked consequence of the design rather
   than an experiment — is the most important honest statement in the paper. If
   an edit softens it to "the harness confirms that content poisoning is not
   detected," revert it.

7. **Keep §7.4's distinction between written-down and enforced.** The paper says
   that the hosted-model and training-data rules are prose, not behaviour: no
   model collector exists, no collector constructs an `Opaque` component, and
   nothing stops a caller writing `Measured` against a hosted model. If a future
   build adds a model collector, that is a new result needing its own
   description — not a tense change in that subsection.

8. **Repeat the benchmark on a second machine.** One host is thin for a
   published figure even with seven repetitions per cell. Either add a second
   platform (quoting its CPU and toolchain) or keep §6's framing that these are
   one machine's numbers.

9. **arXiv metadata.** Suggested categories: cs.CR (primary), cs.SE and cs.AI
   (secondary). Licence: Apache-2.0, matching the implementation.

## Files

| File | Purpose |
| --- | --- |
| `main.tex` | The paper (compiles with tectonic, XeTeX engine) |
| `references.bib` | Bibliography: the SBOM formats and their policy drivers, the provenance line, the commitment literature, and the agent-attack work |
| `main.pdf` | Compiled output. **Committed**, so it can be read in the browser without a LaTeX toolchain — regenerate and commit whenever `main.tex` changes |
