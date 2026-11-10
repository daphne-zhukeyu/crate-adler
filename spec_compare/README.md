# Spec comparison for `adler2`

Files:

| file | what |
|---|---|
| `human_written_spec.rs` | the human spec of the public API, written from RFC 1950 and the crate (workflow step 1) |
| `human_written_spec.v1.rs` | the frozen version of that file; the working copy differs only in the `vectors()` harness |
| `compare.rs` | the comparison crate: includes `../adler2/src/spec.rs` (Claude's spec), the human spec and `links.rs` |
| `links.rs` | one pair of proof fns per public function, `claude_implies_human_<f>` and `human_implies_claude_<f>`, written by Claude (workflow step 3); no `assume`/`admit` |
| `GOLD_SPEC_adler2.md` | the sheet used to write the human spec: files and line ranges, functions, open decisions, vectors |

Procedure (`/home/zhukeyu/verus-tools/WORKFLOW.md`):

1. You write `human_written_spec.rs` from the crate source, its docs and RFC 1950, make sure it
   compiles, and freeze it as `.v1.rs`.
2. Claude writes its spec and proof in `../adler2/`.
3. Given the human spec, Claude amends its spec and proof and fills `links.rs`.
4. You run `bash /home/zhukeyu/verus-tools/spec_compare_run.sh /home/zhukeyu/crate-adler2 2021`.
   It writes `../evidence/SPEC_COMPARE.md` (per-function table, counts per direction) and
   `../evidence/spec_compare.log` (raw Verus output; a FAILED cell's error is the discrepancy).
5. You analyse every gap and decide whether Claude's amended spec is correct.
6. If it is not, Claude regenerates the proof for the correct spec.

Result for adler2: `../evidence/SPEC_COMPARE.md`. Claude's spec implies the human spec for 6 of
7 functions and the human spec implies Claude's for 6 of 7; the one gap, `write_slice` from a
non-canonical `from_checksum` state with an empty write, is analysed in `../SUMMARY.md`.
