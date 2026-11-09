# Function inventory

Upstream source: `/home/zhukeyu/crate-adler2/upstream/src`  
Annotated crate: `/home/zhukeyu/crate-adler2/adler2`  
Verus JSON: `/home/zhukeyu/crate-adler2/evidence/verus_std.json` (version 0.2026.10.04.426d8b0; results: 68 verified, 0 errors, entire crate: True)

**13 of 14 upstream functions verified** (non-test, non-bench `fn` items).

| # | upstream location | function | Verus name | status |
|---|---|---|---|---|
| 1 | `algo.rs:5` | `Adler32::compute` | `lib::Adler32::compute` | verified |
| 2 | `algo.rs:116` | `U32X4::from` | `lib::algo::U32X4::from` | verified |
| 3 | `algo.rs:128` | `U32X4::add_assign  (impl AddAssign<Self>)` | `lib::algo::U32X4::add_assign` | verified |
| 4 | `algo.rs:139` | `U32X4::rem_assign  (impl RemAssign<u32>)` | `lib::algo::U32X4::rem_assign` | verified |
| 5 | `algo.rs:149` | `U32X4::mul_assign  (impl MulAssign<u32>)` | `lib::algo::U32X4::mul_assign` | verified |
| 6 | `lib.rs:95` | `Adler32::new` | `lib::Adler32::new` | verified |
| 7 | `lib.rs:125` | `Adler32::from_checksum` | `lib::Adler32::from_checksum` | verified |
| 8 | `lib.rs:134` | `Adler32::checksum` | `lib::Adler32::checksum` | verified |
| 9 | `lib.rs:142` | `Adler32::write_slice` | `lib::Adler32::write_slice` | verified |
| 10 | `lib.rs:149` | `Adler32::default  (impl Default)` | `lib::Adler32::default` | verified |
| 11 | `lib.rs:156` | `Adler32::finish  (impl Hasher)` | `lib::Adler32::finish` | verified |
| 12 | `lib.rs:160` | `Adler32::write  (impl Hasher)` | `lib::Adler32::write` | verified |
| 13 | `lib.rs:170` | `adler32_slice` | `lib::adler32_slice` | verified |
| 14 | `lib.rs:206` | `adler32` | `-` | unverified |

## Trusted / external markers in the annotated crate (text scan)

| file:line | text |
|---|---|
| `src/lib.rs:16` | `#![deny(unsafe_code)] // was `forbid`; Verus `assume_specification` expands to unsafe code (trusted.rs only)` |
| `src/trusted.rs:18` | `#[verifier::external_type_specification]` |
| `src/trusted.rs:19` | `#[verifier::external_body]` |
| `src/trusted.rs:24` | `pub uninterp spec fn chunks_exact_rest<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;` |
| `src/trusted.rs:27` | `pub uninterp spec fn chunks_exact_size<'a, T>(it: ChunksExact<'a, T>) -> usize;` |
| `src/trusted.rs:30` | `pub uninterp spec fn chunks_exact_rem<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;` |
| `src/trusted.rs:33` | `pub assume_specification<'a, T>[ <[T]>::chunks_exact ](s: &'a [T], chunk_size: usize) -> (it:` |
| `src/trusted.rs:44` | `pub assume_specification<'a, T>[ ChunksExact::<'a, T>::remainder ](it: &ChunksExact<'a, T>) -> (r:` |
| `src/trusted.rs:51` | `pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::next ](` |
