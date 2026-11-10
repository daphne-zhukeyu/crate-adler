# Verifying `adler2` with Verus: results (2026-10-18)

Crate: `adler2` 2.0.1 from crates.io (oyvindln/adler2; 593M downloads; also a dependency of `std` itself, through `miniz_oxide`). Verus **0.2026.10.04.426d8b0** (stable) on rustc 1.98.1, with vstd `=0.0.0-2026-10-04-0306`. The annotated crate is in [`adler2/`](adler2/), the pristine crates.io sources are in [`upstream/`](upstream/), and the diff between them is [`upstream.diff`](upstream.diff). Claude (Opus 5.5) wrote the specifications and proofs from RFC 1950 §8.2 and the crate docs; the human spec they were checked against is in [`spec_compare/`](spec_compare/) (see "Spec comparison" at the end).

## Headline

| | |
|---|---|
| Verus result (default features) | **68 verified, 0 errors**, ~1.8 s wall clock (0.6 s SMT) |
| no_std (`--no-default-features`) | 68 verified, 0 errors |
| Proof stability | identical results for Z3 seeds 1–6 |
| Trusted items (`--no-cheating`) | **3** std specifications for `ChunksExact` (plus their 3 uninterpreted model functions), all in `src/trusted.rs`. No `assume`, `admit` or `external_body` in crate code |
| Not verified | `adler32<R: BufRead>` (generic `io` reader; left outside `verus!`) |
| Plain `cargo build`/`cargo test` | pass: 6/6 unit tests, 4/4 doc tests, plus the new trusted-model test. `--no-default-features`: 5 + 3 pass. Also in `--release` |
| Performance | unchanged: 16.4 GB/s for both upstream and annotated on a 64 MiB buffer (identical results) |
| Negative controls | 14/14 injected bugs rejected (`negative_controls.py`) |

## Coverage by public item

| Public item | Status | Contract (`src/spec.rs`) |
|---|---|---|
| `Adler32::new`, `Default::default` | verified | `r@ == (1, 0)` |
| `Adler32::from_checksum(sum)` (const fn) | verified | `r@ == (sum % 65536, sum / 65536)` |
| `Adler32::checksum` | verified | `r == pack(self@)` = `b * 65536 + a` |
| `Adler32::write_slice(bytes)`, `Hasher::write` | verified, uses T1–T3 | `final(self)@ == adler_from(old.a % 65521, old.b % 65521, bytes@)` |
| `Hasher::finish` | verified | `r == pack(self@)` |
| `adler32_slice(data)` | verified, uses T1–T3 | `r == adler32_spec(data@)` |
| `adler32<R: BufRead>(reader)` | **not verified** | generic `BufRead`/`io::Result` has no vstd model. The function is 12 lines: a loop over `fill_buf`/`write_slice`/`consume` |

The private helpers `Adler32::compute` (the vectorised core), `U32X4::from` and the `+=`/`%=`/`*=` impls on `U32X4` are all verified. `adler_from` is the textbook per-byte recurrence `a = (a + d) % 65521; b = (b + a) % 65521`. It is not the vectorised algorithm, so the proof connects two independent formulations.

### What the proof of `compute` establishes

1. **No `u32` overflow anywhere.** This holds for every input length and every starting state, including non-canonical states from `from_checksum` (a, b up to 65535), which the upstream comment's bound doesn't cover. The key fact is `lemma_b_vec_bound`: within one 5552-group chunk, `b_vec ≤ 65520·(t+1) + 255·t(t+1)/2`, which is at most 4,294,690,200 < 2³² at t = 5552. That machine-checks the "n = 5552.18…, so 5552" calculation in the source comment. Control N1 shows that 5553 groups would overflow.
2. **Deferred modulo is sound** (`lemma_adler_from_unreduced`): per-byte reduction equals one final reduction of the unreduced sums `a + Σd` and `b + n·a + Σ prefix sums`.
3. **The 4-lane decomposition is correct** (`lemma_lanes`): the weighted sum splits as `4·Σ lane_wsum_j − Σ j·lane_sum_j`. This is exactly the combine step `b_vec *= 4; b_vec[j] += j·(MOD − a_vec[j])`, where `j·MOD` is added to keep the term non-negative.

## What the specification commits to (intent decisions)

1. **`from_checksum` does not validate.** Any `u32` is accepted, including states with `a` or `b` ≥ 65521 that no real Adler-32 stream produces. The next `write` reduces them mod 65521, even when it is empty: `from_checksum(0xFFFF_FFFF).checksum() == 0xFFFF_FFFF`, but after `write_slice(&[])` it becomes `0x000E000E` (observed on the real code). The spec states this precisely as `adler_from(a % 65521, b % 65521, …)`. This is a candidate doc note upstream.
2. **Packing.** `checksum()` is `b << 16 | a`, the RFC 1950 byte order. `from_checksum` is its inverse on canonical states.
3. **Only `write` is specified.** `write_u8`, `write_u32` and the other `Hasher` methods use std's defaults (`to_ne_bytes`). The crate docs already warn that the result is endianness-dependent.
4. **Contracts reach concrete callers only.** As in fnv, vstd gives the `Hasher` trait no spec, so code that is generic over `H: Hasher` does not see these contracts.

## Trusted base

| id | item | why trusted | evidence |
|---|---|---|---|
| T1 | `<[T]>::chunks_exact(n)`: full chunks are `s[..len - len % n]`, remainder is `s[len - len % n..]` | vstd 0.2026.10.04 has no spec for `chunks_exact`. Verus main (10-18) has one only for `chunks` | `tests/trusted_model.rs` checks the model against std for lengths 0–199 and boundary lengths × 9 chunk sizes |
| T2 | `ChunksExact::remainder()` is that remainder, independent of iteration | same | same test |
| T3 | `<ChunksExact as Iterator>::next` returns the next `n` items, then `None` forever | Implementing vstd's `IteratorSpec` for a core type is only possible inside vstd (orphan rule E0117), so a crate cannot make `for` over `ChunksExact` verifiable | same test |

T1–T3 mirror vstd's own (equally trusted) spec of `chunks` on Verus main. They would make a small vstd PR, after which this trusted base drops to zero and the three `while let` loops below could go back to `for`.

## Changes to the crate (audit of `upstream.diff`)

| Change | Effect on compiled code |
|---|---|
| `for chunk in chunk_iter`, `for byte_vec in chunk.chunks_exact(4)` and `for byte_vec in remainder_chunk.chunks_exact(4)` → `while let Some(x) = it.next()` (and `let chunk_iter` → `let mut chunk_iter`) | **Executable change**, the only one. It is the `for` desugaring, needed because of T3. Throughput and results are identical (see Headline) |
| `#![forbid(unsafe_code)]` → `#![deny(unsafe_code)]` | none. `assume_specification` expands to unsafe code, allowed only in `trusted.rs`, which is compiled only under Verus (same as hex) |
| Wrapped everything except `adler32<R>` and the tests in `verus! { }`. Added `requires`/`ensures`, loop invariants, `proof { }` blocks and ghost variables, and named 3 array/slice iterators (`for x in it: …`) | none (ghost, erased) |
| `impl View for Adler32` (closed). `AddAssignSpecImpl`/`RemAssignSpecImpl`/`MulAssignSpecImpl` for `U32X4` under `cfg(verus_keep_ghost)` | none (ghost) |
| Ghost copy plus extensionality assert in the three `U32X4` operator impls | none (ghost) |
| New `src/spec.rs` (ghost), `src/trusted.rs` (Verus-only), `tests/trusted_model.rs` | none to the library |
| Cargo.toml: `vstd` dependency, `[package.metadata.verus]`, `rust-toolchain.toml`. Removed cargo-release metadata | build-time only |

## Negative controls (all rejected)

`VERUS=/path/to/verus python3 -I negative_controls.py` injects each bug into a scratch copy.

| id | injected bug | rejected by |
|---|---|---|
| N0 | `assert(false)` in `compute` (vacuity check) | assertion |
| N1 | chunk of 5553 groups instead of 5552 | overflow budget (`lemma_b_vec_bound` precondition) |
| N2 | modulus 65519 | proof of postcondition |
| N3 | combine step drops lane-1 correction | proof of postcondition |
| N4 | lane-3 weight 2 instead of 3 | proof of postcondition |
| N5 | `b_vec *= 3` | proof of postcondition |
| N6 | full chunks forget `b += CHUNK_SIZE * a` | proof of postcondition |
| N7 | final `a_vec %= MOD` dropped | `MOD - a_vec` underflow |
| N8 | serial tail updates `b` before `a` | loop invariant |
| N9 | `U32X4 +=` mixes lanes | overflow / impl spec |
| N10 | initial state `(0, 1)` | postcondition |
| N11 | `checksum` swaps `a` and `b` | postcondition |
| N12 | `from_checksum` uses `>> 15` | postcondition |
| N13 | spec typo: modulus 65519 in `spec.rs` | `by (compute)` test vectors |

Most `compute` mutants surface as a failed precondition of a proof lemma (for example `lemma_combine`), meaning the chain of facts leading to the postcondition breaks at that point.

## Verus findings from this pass (toolchain notes)

- **`cargo verus` cannot verify adler2 with `std` (bug, also on main 10-18).** `handle_transitive_imports` in `rust_verify/src/cargo_verus.rs` matches loaded crates by *name*. `std` privately depends on a crate called `adler2`, so verifying the real `adler2` makes Verus look for a `.vir` for std's copy and fail with `could not find .vir file for 'adler2'`. `--no-default-features` works because `core` has no such dependency. Workaround: the bare `verus` binary (commands below). The same would affect any crate named like a std-internal dependency (`hashbrown`, `memchr`, `object`, `gimli`, `addr2line`, `miniz_oxide`, …), so it's worth an upstream issue.
- **No `chunks_exact` spec, and a crate cannot add one usable by `for`** (see T3). Suggested vstd addition: `ChunksExact`/`chunks_exact`/`remainder`, mirroring `Chunks`.
- **Operator-trait specs work well** (`AddAssignSpecImpl` etc.). One gotcha: calling the trait's own spec fn (`AddAssignSpec::add_assign_spec`) inside the impl being verified is rejected as a cyclic definition, so the proof asserts against the explicit array instead. The spec fns on a private type must be `closed`.
- **`while let` loops need an `ensures` clause** to keep the exit condition (`rest.len() < n`). Plain invariants don't carry the `None` branch out of the loop.
- **A function-local `const MOD` shadows a crate-level `spec fn MOD()`, even in `ensures`.** The spec constant was renamed to `base()` (RFC 1950's `BASE`).
- **Solver sensitivity between drivers.** One arithmetic step in `lemma_combine` passed under the bare binary but failed under `cargo verus`. Rewriting it with explicit quotients and the literal 65521 made it pass under both drivers and all seeds tried.

## Reproduce

```sh
cd adler2
verus --crate-type=lib --edition 2021 --cfg 'feature="std"' src/lib.rs          # 68 verified, 0 errors
verus --crate-type=lib --edition 2021 src/lib.rs                                # no_std
cargo verus verify --no-default-features                                        # no_std via cargo
verus --crate-type=lib --edition 2021 --no-cheating --cfg 'feature="std"' src/lib.rs   # lists only trusted.rs items
cargo build && cargo test && cargo test --no-default-features && cargo test --release
cd .. && VERUS=$(which verus) python3 -I negative_controls.py                   # 14/14 rejected
```

If `cargo build` has run in `adler2/`, run `cargo clean` before `cargo verus` (the two share `target/`, and a non-Verus vstd build there breaks `cargo verus`).

## Spec comparison

Only `adler_from`, `pack` and `adler32_spec` (the first ~50 lines of `spec.rs`) are the specification; the rest is proof machinery. The human spec of the public API is `spec_compare/human_written_spec.rs` (frozen as `.v1.rs`), written from RFC 1950 §8.2 and the crate docs. Given it, Claude's spec needed no amendment; Claude wrote the linking proofs in `spec_compare/links.rs`, and the equivalence script gives 6 of 7 functions in each direction (`evidence/SPEC_COMPARE.md`). The gap is the reduction of non-canonical `from_checksum` states on an empty write (intent decision 1): the human spec says no-op, the code and this spec reduce. `adler32<R: BufRead>` is outside both specs.

### Amendments

None to Claude's spec or proof. The only edit to the working copy of the human spec is inside its `vectors()` harness (`let`-bound values inlined so `by (compute)` can evaluate); `diff spec_compare/human_written_spec.v1.rs spec_compare/human_written_spec.rs` shows it.
