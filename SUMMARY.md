# adler2 2.0.1 with Verus: summary (Updated: 2026-11-08)

adler2 is the Adler-32 checksum that std ships inside `miniz_oxide` (593M downloads). The rules
are those used for hex: no `assume`, the crate passes `--no-cheating`, and anything Verus lacks a
model for goes into a declared trusted base. Two additions this time: every number in
[REPORT.md](REPORT.md) points to the file that produced it, and the spec was checked against a
gold spec written blind.

## Headline

13 of 14 functions verified, including the vectorised deferred-modulo core, against the per-byte
definition in RFC 1950. The unverified function is `adler32<R: BufRead>`, a generic reader loop
for which vstd has no model. The trusted base consists of three std specifications for
`chunks_exact`. No upstream bug was found. One documentation gap was found, described below.

Source of the count: [FN_INVENTORY.md](evidence/FN_INVENTORY.md), generated from Verus's
`--output-json` over every `fn` in the upstream source; row 14 is the unverified function.
Verus result: 68 verified, 0 errors ([verus_std.log](evidence/verus_std.log)).

## What "verified" is measured against

Claude's spec is the first 50 lines of [adler2/src/spec.rs](adler2/src/spec.rs): `adler_step`,
`adler_from`, `pack`, `adler32_spec`, the RFC definition per byte modulo 65521. Chunking, lane
sums and overflow belong to the proof and are absent from the spec. The gold spec,
[human_written_spec.v1.rs](spec_compare/human_written_spec.v1.rs), was written from the RFC and
the crate docs before the Claude spec was opened, and has the same shape.

A verified crate meets its spec; the spec itself is supported by two kinds of evidence: published
test vectors evaluated inside Verus (`by (compute)` in both spec files), and the two-way comparison
below.

## Spec comparison

Each cell in [SPEC_COMPARE.md](evidence/SPEC_COMPARE.md) is a Verus proof obligation from
[links.rs](spec_compare/links.rs), run with `--no-cheating`. Claude's spec implies the gold spec
for 6 of 7 functions, and the gold spec implies Claude's for 6 of 7. These are proofs over all
inputs in both directions ([spec_compare.log](evidence/spec_compare.log): 62 verified, 2 errors,
the two being the row below).

The failing row is `write_slice`, in one situation only: a state restored with `from_checksum`
from a `u32` outside the range of real checksums, followed by an empty write. The gold spec leaves
such a state unchanged; the code reduces both fields modulo 65521 on every write. Two further rows
show the specs agree for every canonical state and for every non-empty write. The disagreement is
computed in the last proof function of `links.rs` (`counterexample_write_slice_empty`): from
`0xFFFF_FFFF`, the crate's spec gives `(14, 14)` and the gold spec gives `(65535, 65535)`. The real
code gives the crate's answer ([doc_findings.rs::d1](adler2/tests/doc_findings.rs)).

The docs describe the argument of `from_checksum` as "a precomputed Adler-32 checksum" and say
nothing about other values. Two readers of the docs specified different behaviour and the crate
implements one of them. This is a documentation gap surfaced by the comparison method; such an
input rarely appears in a test. Suggested upstream wording: `sum` must be a value returned by
`checksum()`. All documentation claims and their checks are in
[DOC_REVIEW.md](evidence/DOC_REVIEW.md).

## Effect of annotation on the compiled code

A script expands both the upstream and the annotated crate with rustc, which erases ghost code,
and diffs the result: [CODE_CHANGES.md](evidence/code_changes/CODE_CHANGES.md). There are 10
hunks; 9 are residue of erased proof blocks. The one executable change rewrites three `for` loops
as the equivalent `while let`, required because vstd has no spec for `chunks_exact`. The release
assembly of `write_slice`, which contains the whole core inlined, is identical to upstream across
268 instructions; throughput is 16.4 GB/s for both ([bench.log](evidence/bench.log)). Every hunk
and every trusted marker has a line in [SIGNOFF.md](evidence/code_changes/SIGNOFF.md), with
proposed classifications in [REVIEW_NOTES.md](evidence/code_changes/REVIEW_NOTES.md); the
sign-off is still open.

## Trusted base

The header table of [trusted.rs](adler2/src/trusted.rs) lists the three trusted items, all part
of the `ChunksExact` model, each with its reason; `--no-cheating` flags exactly these
([verus_nocheating.log](evidence/verus_nocheating.log)). They are checked against std at runtime
for lengths 0–199 and the chunk boundaries ([trusted_model.rs](adler2/tests/trusted_model.rs));
a proof of them is unavailable from outside vstd. If this model were wrong, the proof of
`compute` would be unsound. vstd makes the same assumption for `chunks` on its main branch. A
vstd pull request adding `chunks_exact` would bring the trusted base to zero and allow the loops
to return to `for`.

## Robustness checks

Fourteen injected bugs were all rejected by Verus, including an `assert(false)` vacuity check and
a chunk one group too long, which really overflows
([negative_controls.log](evidence/negative_controls.log)). Results are identical under six SMT
random seeds; verification takes 1.6 s wall clock. The upstream tests pass in std, no_std and
release builds. Every command behind these statements, with its exit code and output, is listed
in [INDEX.md](evidence/INDEX.md).

## Cost

From the transcript timestamps in [TIMELINE.md](evidence/TIMELINE.md): about 12 minutes of
spec-and-proof work over roughly 20 verify-fix rounds, and 19 minutes in total including a
toolchain bug investigation. The blind gold spec took longer than that. The rest of the pipeline
is scripted: [WORKFLOW.md](../verus-tools/WORKFLOW.md) is the eight-step procedure, `new_crate.sh`
sets up a new crate in one command, and [GOLD_SPEC_GUIDE.md](../verus-tools/GOLD_SPEC_GUIDE.md)
covers writing the gold spec, including the pitfalls hit this time.

## Toolchain findings

Two Verus issues worth filing, with error text in [VERIFICATION.md](VERIFICATION.md):
`cargo verus` fails on any crate whose name coincides with a private dependency of std (adler2
among them), so the bare binary was used ([cargo_verus_std.log](evidence/cargo_verus_std.log));
and `--no-cheating` passed through cargo breaks the vstd build.

## Open items

- A decision on the `write_slice` disagreement: documentation note upstream, or an issue; and
  whether the gold spec is revised (the frozen `.v1` copy stays either way).

## Note on the gold spec file

Claude's only edit to the working gold spec is inside the `vectors()` proof function: `let`-bound
values were inlined into the asserts, because `by (compute)` treats such a variable as symbolic
and the residue kept the SMT solver running without end. Definitions are untouched;
`diff spec_compare/human_written_spec.v1.rs spec_compare/human_written_spec.rs` shows the change.
