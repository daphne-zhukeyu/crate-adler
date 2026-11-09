# Blind spec comparison for `adler2`

Protocol (so the comparison stays independent):

1. Write `human_written_spec.rs` from the reference (FNV page / RFC 1950) and the crate docs
   **before** opening `../adler2/src/spec.rs`. Put it inside `verus! { }`; name and shape are yours.
2. Then ask Claude to fill `links.rs`: for each public function `f`, two proof fns
   `claude_implies_human_f` and `human_implies_claude_f`, each stating one direction of
   refinement between the two contracts. No `assume`/`admit` (the runner uses `--no-cheating`).
   Obligations not yet attempted are listed as `// TODO <name>` lines.
3. Run `bash /home/zhukeyu/verus-tools/spec_compare_run.sh /home/zhukeyu/crate-adler2 2021`.
   It writes `../evidence/SPEC_COMPARE.md` (per-function table, counts per direction) and
   `../evidence/spec_compare.log` (raw Verus output; a FAILED cell's error is the discrepancy).

Start with `GOLD_SPEC_adler2.md` in this directory (what to read, what to avoid, decisions) and
`/home/zhukeyu/verus-tools/GOLD_SPEC_GUIDE.md` (Verus spec syntax, self-checks, common errors).
