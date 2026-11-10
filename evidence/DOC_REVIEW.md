# Documentation review: adler2 2.0.1

Every doc-comment claim in `upstream/src/lib.rs` that says something about behaviour, with the
check that backs the verdict. Line numbers refer to `upstream/src/lib.rs`. Runtime checks are in
`adler2/tests/doc_findings.rs` (run: `cargo test --test doc_findings`, log: `cargo_test.log`).

| # | lines | doc claim | verdict | evidence |
|---|---|---|---|---|
| 1 | 7 | "Zero `unsafe`" | holds for executable code; the annotated crate relaxes `forbid` to `deny` so that Verus-only `trusted.rs` can hold `assume_specification` | `code_changes/CODE_CHANGES.md` (hunk C1, markers T1) |
| 2 | 8 | "Decent performance (3-4 GB/s)" | understated on this machine (16.4 GB/s); not a correctness claim | `bench.log` row of SUMMARY.md |
| 3 | 9 | `#![no_std]` support with `default-features = false` | holds | `verus_nostd.log`, `cargo_test_nostd.log` |
| 4 | 31-33 | "An instance of this type is equivalent to an Adler-32 checksum ... created ... from a precalculated checksum via `from_checksum`" | **D1 (doc gap):** `from_checksum` also accepts values that are not Adler-32 checksums (either 16-bit half ≥ 65521). Such an instance reports the raw value from `checksum()` until the first `write_slice`, which reduces it mod 65521 even if the slice is empty: `0xFFFF_FFFF` → `0x000E_000E`. Nothing in the docs says so | `doc_findings.rs::d1_from_checksum_non_canonical_state`; spec `adler_from(a % 65521, b % 65521, …)` in `adler2/src/spec.rs` |
| 5 | 35-38 | `Hasher` impl; "hash value depends on architecture endianness" (doc test, lines 73-79) | holds; the `Hasher` integer methods are std defaults over `to_ne_bytes` and are not part of the verified surface | doc test passes (`cargo_test.log`); VERIFICATION.md intent decision 3 |
| 6 | 99-102 | `from_checksum` "allows resuming checksum calculation" | holds for every checksum the crate itself produces | `doc_findings.rs::d2_from_checksum_roundtrip_on_canonical_states`; Verus: `from_checksum` ensures `r@ == (sum % 65536, sum / 65536)` and `checksum` ensures `pack(self@)` |
| 7 | 132 | `checksum` "Returns the calculated checksum at this point in time" | holds after any write; for a D1 instance before its first write it returns the raw constructor input, which is not a checksum | same as 4 |
| 8 | 138-141 | `write_slice` "Adds `bytes` to the checksum calculation" | holds; verified contract `final(self)@ == adler_from(old.a % 65521, old.b % 65521, bytes@)` | `verus_std.log`; `FN_INVENTORY.md` row 9 |
| 9 | 175-186 | `adler32<R: BufRead>` reads to EOF, bubbles up reader errors | holds at runtime; **not verified** (generic reader has no vstd model) | `doc_findings.rs::d3_adler32_bufread_error_and_chunking`; `FN_INVENTORY.md` row 14 |
| 10 | 168-170 | `adler32_slice` is "a convenience function around the `Adler32` type" | holds; verified `r == adler32_spec(data@)` | `verus_std.json` (`lib::adler32_slice`) |

## Upstream bugs found

None. The one behavioural surprise (D1) is reachable only by passing a non-checksum `u32` to
`from_checksum`, which the docs neither allow nor forbid. Worth a doc note upstream ("`sum`
must be a value previously returned by `checksum`") or a debug assertion; not a panic, not a
wrong result for any documented input.

## Source-comment claim checked by proof

`algo.rs` comment (upstream lines 26-30): the deferred-modulo chunk size "n = 5552.18..., so 5552".
Machine-checked by `lemma_b_vec_bound` in `adler2/src/spec.rs`: with 5552 groups of 4 per chunk
no `u32` overflows, for every starting state the type allows (including D1 states, which the
comment's own bound of "a and b always in range 0..MOD" does not cover). Negative control N1
(`negative_controls.log`) shows 5553 is rejected.
