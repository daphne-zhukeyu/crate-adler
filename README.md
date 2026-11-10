# adler2 2.0.1 verified with Verus

[adler2](https://crates.io/crates/adler2) is the Adler-32 checksum crate that the Rust standard
library ships inside `miniz_oxide`. This repository holds the crate annotated with Verus
specifications and proofs, the evidence behind every reported number, and the hand-written
human spec the Claude-written spec was checked against.

Workflow (details in [verus-tools/WORKFLOW.md](../verus-tools/WORKFLOW.md)): the human writes a
spec from the crate source, docs and RFC 1950 and checks that it compiles; Claude writes its spec
and proof; given the human spec, Claude amends them; the human runs the equivalence script in
both directions and analyses every gap; if Claude's amended spec is wrong, Claude regenerates the
proof for the correct spec. Spec and proof generation and amendment use Opus 5.5 at thinking
level max.

**Results:** 13 of 14 functions verified against the per-byte definition in RFC 1950, including the
vectorised deferred-modulo core; 3 trusted std specifications (`chunks_exact`); 0 `assume`; the
compiled code of the hot path is byte-identical to upstream; Claude's spec implies the human spec
for 6 of 7 functions and the human spec implies Claude's for 6 of 7, and the one gap is a
documentation gap in the crate.

| read this | for |
|---|---|
| [SUMMARY.md](SUMMARY.md) | the report: results, a claim-to-evidence table, pending decisions, caveats |
| [VERIFICATION.md](VERIFICATION.md) | spec rationale, intent decisions, trusted base, Verus toolchain findings |
| [PRESENTATION.md](PRESENTATION.md) | ten-minute walkthrough script |

## Layout

```
upstream/        the crates.io tarball, unpacked, never edited
adler2/          the annotated crate: src/lib.rs, src/algo.rs (contracts and proofs in place),
                 src/spec.rs (specification + proof lemmas), src/trusted.rs (Verus-only trusted seams),
                 tests/trusted_model.rs (runtime check of the trusted seams), tests/doc_findings.rs
upstream.diff    annotated crate vs upstream
spec_compare/    human spec (human_written_spec.v1.rs is the frozen version), linking
                 proofs (links.rs), and the sheet used to write the human spec
evidence/        every log, inventory, diff and comparison the reports cite (INDEX.md lists them)
bench_compare/   upstream vs annotated throughput, same process, equal results asserted
negative_controls.py   14 injected bugs, each rejected by Verus
```

## Reproduce

Toolchain: Verus `0.2026.10.04.426d8b0` (stable release, bare binary), rustc 1.98.1 via rustup
(pinned in `adler2/rust-toolchain.toml`), vstd `=0.0.0-2026-10-04-0306` from crates.io.

```sh
cd adler2
verus --crate-type=lib --edition 2021 --cfg 'feature="std"' src/lib.rs                 # 68 verified, 0 errors
verus --crate-type=lib --edition 2021 src/lib.rs                                       # no_std: 68 verified
verus --crate-type=lib --edition 2021 --no-cheating --cfg 'feature="std"' src/lib.rs   # flags only src/trusted.rs
cargo build && cargo test && cargo test --no-default-features && cargo test --release
cd .. && VERUS=$(which verus) python3 -I negative_controls.py                          # 14/14 rejected
```

Spec equivalence check (human spec vs Claude spec, both directions, `--no-cheating`):

```sh
cd spec_compare && verus --crate-type=lib --edition 2021 --no-cheating compare.rs      # 62 verified, 2 errors
```

The two errors are the two unconditional `write_slice` obligations; see
[evidence/SPEC_COMPARE.md](evidence/SPEC_COMPARE.md) for why they are expected and what they show.

`cargo verus verify --no-default-features` also works. With the `std` feature, `cargo verus` fails
on this crate because std has a private dependency of the same name (a Verus issue, documented in
VERIFICATION.md); the bare binary is the supported path.

## Trusted base

Three `assume_specification`s in `adler2/src/trusted.rs`, modelling `<[T]>::chunks_exact`,
`ChunksExact::remainder` and `ChunksExact::next`. vstd has no specification for these, and a
crate cannot implement vstd's iterator specification for a core type. The model is checked
against the real standard library in `adler2/tests/trusted_model.rs`. A vstd addition would bring
the trusted base to zero.

## Status

Pending: a decision on the `from_checksum` documentation gap (recommended upstream wording:
`sum` must be a value returned by `checksum()`); see the Pending section of
[SUMMARY.md](SUMMARY.md).

## License

The crate is `0BSD OR MIT OR Apache-2.0` (see `upstream/`). The annotations, specifications,
proofs and tooling in this repository are offered under the same terms.
