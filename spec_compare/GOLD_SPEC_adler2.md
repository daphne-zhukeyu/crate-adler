# Gold spec sheet: adler2 2.0.1

General guide: `/home/zhukeyu/verus-tools/GOLD_SPEC_GUIDE.md`. Output file:
`human_written_spec.rs` in this directory.

## Read

| file | why | what to take from it |
|---|---|---|
| `../upstream/src/lib.rs` lines 1–9 | crate docs | promises: zero unsafe, no_std |
| `../upstream/src/lib.rs` lines 29–90 | `Adler32` docs and struct | "equivalent to an Adler-32 checksum", `new`/`from_checksum`/`checksum`, the `Hasher` note, the doc examples (`[0,1,2]` then `[3,4,5]` gives `0x00290010`) |
| `../upstream/src/lib.rs` lines 92–174 | the public API and its doc comments | signatures of `new`, `from_checksum`, `checksum`, `write_slice`, `Default`, `Hasher::finish`, `Hasher::write`, `adler32_slice` |
| `../upstream/src/lib.rs` lines 176–220 | `adler32<R: BufRead>` | out of scope for the spec (no reader model); read only so you know it exists |
| `../upstream/src/lib.rs` lines 222–287 | the crate's unit tests | vectors: `adler32_slice(&[]) == 1`, `&[0] == 0x00010001`, `&[1] == 0x00020002`, `&[40] == 0x00290029`, `b"Wikipedia" == 0x11E60398`; the `resume` test shows the intended `from_checksum` round trip |
| RFC 1950 §8.2, https://www.rfc-editor.org/rfc/rfc1950#section-8.2 | the algorithm | `s1 = 1 + D1 + … + Dn (mod 65521)`, `s2 = (1 + D1) + (1 + D1 + D2) + … (mod 65521)`, `Adler-32 = s2*65536 + s1`; the C reference code in the RFC is 10 lines |
| https://en.wikipedia.org/wiki/Adler-32 | worked example | the "Wikipedia" computation, digit by digit |

**Do not read `../upstream/src/algo.rs`.** It is the vectorised, chunked, deferred-modulo
implementation. A spec transcribed from it would just restate the code. The RFC's two-line
definition is the spec; the proof that `algo.rs` meets it is Claude's job and is the whole point.

## Do not read until the comparison is done

`../adler2/src/*.rs` (all contain contracts), `../VERIFICATION.md`, `../REPORT.md`,
`../evidence/DOC_REVIEW.md`, `../adler2/tests/doc_findings.rs`, `../negative_controls.py`,
`links.rs` and `compare.rs` in this directory.

## Functions to specify

| API | shape of the spec you need |
|---|---|
| `Adler32::new()`, `Default::default()` | the initial state |
| `Adler32::from_checksum(sum: u32)` | the state as a function of `sum` |
| `Adler32::checksum(&self) -> u32` | the `u32` as a function of the state |
| `Adler32::write_slice(&mut self, bytes: &[u8])`, `Hasher::write` | the new state as a function of the old state and `bytes` |
| `Hasher::finish(&self) -> u64` | relation to `checksum` |
| `adler32_slice(data: &[u8]) -> u32` | the checksum of `data` from the initial state |

Abstract state: decide first. The natural choices are the pair `(s1, s2)` as two integers, or the
packed `u32`. Both can be linked; the pair makes `write_slice` easiest to state. Whatever you pick,
`checksum` and `from_checksum` must be stated against it.

Out of scope: `adler32<R: BufRead>` (unverified in the annotated crate; a 12-line loop calling
`write_slice`), the `Hasher` integer methods (std defaults), `Clone`/`Debug`.

## Decisions the docs leave to you (write each down as a comment)

1. **Modulus and packing.** 65521 and `s2 * 65536 + s1`. Straight from the RFC.
2. **Initial state.** `s1 = 1, s2 = 0` (so `adler32(&[]) == 1`).
3. **Order.** Per byte: update `s1` first, then `s2` with the *new* `s1`. Getting this backwards
   gives a different checksum; the `[40]` vector catches it.
4. **Incremental writes.** `write(a); write(b)` must equal `write(a ++ b)`. Your definition
   should make this true by construction (a fold from an arbitrary state).
5. **`from_checksum` on a value that is not a checksum.** The docs say it takes "a precomputed
   Adler-32 checksum". A `u32` whose low or high 16 bits are ≥ 65521 is not one. Decide what
   your spec says the state is in that case, and what the next `write` does to it. Options: leave
   it unspecified (a `recommends`), reduce it, or define the state as the raw halves. **Not blind:**
   this case was discussed in chat on 2026-10-19 and is the one intent decision most worth your
   independent view. Write what the docs would lead you to, then note whether that differs from
   what you already heard.
6. **`finish`.** `u64::from(checksum())`: zero-extended, not sign-extended, not shifted.

## Vectors to include (from the RFC and the crate's tests)

```
[]            1
[0]           0x00010001
[1]           0x00020002
[40]          0x00290029
[0, 1, 2, 3, 4, 5]   0x00290010   (doc example; also as write [0,1,2] then [3,4,5])
b"Wikipedia"  0x11E60398
```

## adler2-specific places to get stuck

- Keep `s1`, `s2` as `int` in the spec and reduce with `% 65521`; the operands are non-negative
  so Euclidean `%` equals Rust's. Reducing once at the end instead of every byte is also a valid
  definition (the RFC does it per byte), but say which one you chose.
- `s2 * 65536 + s1` as the packed value: write it in `int`, then `as u32` only at the API
  boundary (`checksum`), or state `checksum` returns that integer and let the linker cast.
- `from_checksum`: `sum % 65536` and `sum / 65536` (Euclidean, exact on `u32`), or `sum as u16`
  and `(sum >> 16) as u16`: equivalent; the arithmetic form is easier to reason about later.
- `by (compute)` on `b"Wikipedia"` (9 bytes) takes well under a second; the 1024-byte vectors in
  the crate's tests are too long for the interpreter, skip them.
- Don't model chunk sizes, `u32` overflow or the 5552 constant. Those are implementation
  concerns and belong to the proof, not the spec.
