# Speaking script: fnv + adler2 with Verus (~10 min)

Each step: what to open, what to say. Numbers are the ones in REPORT.md / evidence/.

---

**1. Framing (30 s)** — slide or nothing open

"Two more crates after hex: fnv, a one-function hash, as the toolchain sanity check, and adler2,
the Adler-32 checksum that std itself ships inside miniz_oxide. Same rules as hex: no `assume`,
must pass `--no-cheating`, anything Verus can't model goes in a declared trusted base. Two things
are new this time: every number in the report points at a file that produced it, and the spec
was checked against a gold spec I wrote blind."

**2. Headline (1 min)** — `crate-adler2/REPORT.md`, the Summary paragraph

"adler2: 13 of 14 functions verified, including the vectorised deferred-modulo core, against the
textbook RFC 1950 definition. The one unverified function is the generic `BufRead` reader; vstd
has no model for it. Trusted base: three std specifications for `chunks_exact`, and nothing
else. No upstream bug. One doc gap, which I'll come back to, because it's the interesting part.
fnv: 4 of 4, empty trusted base, nothing to report beyond a doc gap about which FNV variant it is."

**3. What 'verified against' means (1.5 min)** — `adler2/src/spec.rs`, lines 1–50, side by side
with `spec_compare/human_written_spec.v1.rs`

"Left: Claude's spec. `adler_step`, `adler_from`, `pack`. That's the RFC, per byte, modulo 65521.
Nothing about chunks, lanes or overflow; that's all in the proof, not the spec. Right: my spec,
written from the RFC and the docs before I opened the left file. Same shape, independently.
The point of showing both: 'verified' only means 'meets the spec', so the spec needs its own
evidence. It has two kinds: published test vectors evaluated inside Verus, and this comparison."

**4. The comparison result (2 min)** — `evidence/SPEC_COMPARE.md`

"Each cell is a Verus proof. Claude's spec implies mine for 6 of 7 functions, mine implies Claude's
for 6 of 7. Not a test: a proof over all inputs, both directions. fnv is 4 of 4 both ways.

The one failed row is `write_slice`, and only in one situation: a state restored with
`from_checksum` from a `u32` that isn't a real checksum, followed by an empty write. I wrote that
an empty write changes nothing; the code reduces both fields mod 65521 on every write. The two
rows below show the specs agree whenever the state is canonical or the slice is non-empty."

Open `spec_compare/links.rs`, the last proof fn `counterexample_write_slice_empty`:

"The disagreement as a computed fact: from `0xFFFF_FFFF`, crate gives (14, 14), my spec gives
(65535, 65535). And `tests/doc_findings.rs::d1` runs the real code and gets the crate's answer."

**5. Why that row matters (45 s)** — stay on the same screen

"The docs say `from_checksum` takes 'a precomputed Adler-32 checksum' and nothing about anything
else. Two readers of the docs specified different behaviour; the crate does one of them. That's a
documentation gap found by the method, and it's harder to find by testing because the input is
one nobody writes a test for. Recommendation upstream: say `sum` must come from `checksum()`."

**6. Did annotating change the code? (1 min)** — `evidence/code_changes/CODE_CHANGES.md`, last table

"A script expands both crates with rustc, which erases the ghost code, and diffs. adler2: 10 hunks,
9 of them erased-proof residue. One executable change: three `for` loops became the equivalent
`while let`, forced because vstd has no spec for `chunks_exact`. Bottom table: release assembly of
`write_slice`, which has the whole core inlined, is identical to upstream, 268 instructions.
Throughput 16.4 GB/s on both. Every hunk has a sign-off line; that review is mine to finish."

**7. The honest part (45 s)** — `adler2/src/trusted.rs`, header table

"Three trusted items, all the `ChunksExact` model, each with the reason. These are tested against
std across lengths 0–199 and the chunk boundary, not proven. If this model is wrong, the proof of
`compute` is unsound. It's the same assumption vstd itself makes for `chunks` on main; the right
fix is a vstd PR, after which the trusted base is zero and the loops go back to `for`."

**8. Process and cost (45 s)** — `evidence/TIMELINE.md` or just say it

"Claude Code time, from the transcript timestamps: fnv about 2 minutes of spec-and-proof work,
6 minutes total. adler2 about 12 minutes spec-and-proof, roughly 20 verify-fix rounds, 19 minutes
total including the toolchain bug investigation. My blind spec took longer than that.
Everything else is scripts: `verus-tools/WORKFLOW.md` is the eight-step procedure, and one command
sets up a new crate."

**9. Toolchain findings (30 s)**

"Two Verus issues worth filing: `cargo verus` can't verify any crate whose name collides with a
private std dependency, adler2 included, so I used the bare binary; and `--no-cheating` through
cargo breaks the vstd build. Both are in the report with the error text."

**10. Close (20 s)**

"Open items: my sign-off on the 23 code-change lines, and a decision on the `write_slice`
disagreement: doc note or upstream issue. Next crates: itoa is Shawn's; I'm looking at base16ct
and percent-encoding."

---

## Likely questions

- *"Could the gold spec just be wrong?"* Yes; that's why it carries the same published vectors.
  Where it disagrees with Claude's spec, the code decides, and here the code sides with Claude's.
  The disagreement is still a finding about the docs.
- *"13 of 14: what is the 14th?"* `adler32<R: BufRead>`, 12 lines, a loop over `fill_buf` /
  `write_slice` / `consume`. Tested at runtime (`doc_findings.rs::d3`), not verified.
- *"How many bugs would the spec catch?"* 14 injected, 14 rejected, including an `assert(false)`
  vacuity check and a chunk one group too long, which really overflows. `negative_controls.log`.
- *"Is the proof fragile?"* Same result under 6 SMT seeds; 1.6 s wall clock.
- *"What did Claude change in my spec?"* Only the vector harness: `let`-bound values inlined so
  `by (compute)` can evaluate. `diff human_written_spec.v1.rs human_written_spec.rs`.
