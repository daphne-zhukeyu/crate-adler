# Code changes after macro expansion

upstream: `/home/zhukeyu/crate-adler2/upstream`  
annotated: `/home/zhukeyu/crate-adler2/adler2`  
features: `default`

Full expansions: `expanded_upstream.rs`, `expanded_annotated.rs`; diff: `expanded.diff`.

**10 diff hunk(s)** in executable code, **10 trusted/unsafe marker line(s)**. Each is a finding in `SIGNOFF.md`.

## C1

```diff
@@ -13,9 +13,12 @@
 #![doc(test(attr(deny(unused_imports, unused_must_use))))]
 #![warn(missing_debug_implementations)]
-#![forbid(unsafe_code)]
+#![deny(unsafe_code)]
 extern crate std;
 #[prelude_import]
 use std::prelude::rust_2021::*;
+// was `forbid`; Verus `assume_specification` expands to unsafe code (trusted.rs only)
+use vstd::prelude::*;
 mod algo {
+    // verus!
     // from above
     // from above
```

## C2

```diff
@@ -25,4 +28,5 @@
     use crate::Adler32;
     use std::ops::{AddAssign, MulAssign, RemAssign};
+    use vstd::prelude::*;
     impl Adler32 {
         pub(crate) fn compute(&mut self, bytes: &[u8]) {
```

## C3

```diff
@@ -34,26 +38,42 @@
             let mut b_vec = a_vec;
             let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);
-            let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
+            {}
+            let mut chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
             let remainder_chunk = chunk_iter.remainder();
-            for chunk in chunk_iter {
-                for byte_vec in chunk.chunks_exact(4) {
+            {}
+            while let Some(chunk) = chunk_iter.next() {
+                {}
+                let mut byte_vec_iter = chunk.chunks_exact(4);
+                {}
+                while let Some(byte_vec) = byte_vec_iter.next() {
+                    {}
                     let val = U32X4::from(byte_vec);
                     a_vec += val;
                     b_vec += a_vec;
+                    {}
                 }
+                {}
                 b += CHUNK_SIZE as u32 * a;
                 a_vec %= MOD;
                 b_vec %= MOD;
                 b %= MOD;
-            }
-            for byte_vec in remainder_chunk.chunks_exact(4) {
+                {}
+            }
+            {}
+            let mut byte_vec_iter = remainder_chunk.chunks_exact(4);
+            {}
+            while let Some(byte_vec) = byte_vec_iter.next() {
+                {}
                 let val = U32X4::from(byte_vec);
                 a_vec += val;
                 b_vec += a_vec;
-            }
+                {}
+            }
+            {}
             b += remainder_chunk.len() as u32 * a;
             a_vec %= MOD;
             b_vec %= MOD;
             b %= MOD;
+            {}
             b_vec *= 4;
             b_vec.0[1] += MOD - a_vec.0[1];
```

## C4

```diff
@@ -61,13 +81,18 @@
             b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
             for &av in a_vec.0.iter() {
+                {}
                 a += av;
             }
             for &bv in b_vec.0.iter() {
+                {}
                 b += bv;
             }
+            {}
             for &byte in remainder.iter() {
+                {}
                 a += u32::from(byte);
                 b += a;
             }
+            {}
             self.a = (a % MOD) as u16;
             self.b = (b % MOD) as u16;
```

## C5

```diff
@@ -106,4 +131,5 @@
             self.0[2] += other.0[2];
             self.0[3] += other.0[3];
+            {}
         }
     }
```

## C6

```diff
@@ -115,4 +141,5 @@
             self.0[2] %= quotient;
             self.0[3] %= quotient;
+            {}
         }
     }
```

## C7

```diff
@@ -124,7 +151,29 @@
             self.0[2] *= rhs;
             self.0[3] *= rhs;
-        }
-    }
-}
+            {}
+        }
+    }
+}
+#[doc = " Specification of Adler-32 (ghost; erased from normal builds)."]
+pub mod spec {
+    //! Specification of Adler-32 (ghost code only; erased from normal builds).
+    //!
+    //! The contracts in `lib.rs` are stated with `adler_from` and `adler32_spec`, the textbook
+    //! per-byte definition from RFC 1950 §8.2:
+    //!
+    //! ```text
+    //! A = 1 + D1 + D2 + ... + Dn (mod 65521)
+    //! B = (1 + D1) + (1 + D1 + D2) + ... + (1 + D1 + D2 + ... + Dn) (mod 65521)
+    //! Adler-32(D) = B * 65536 + A
+    //! ```
+    //!
+    //! Everything else here (unreduced sums, lane sums and the lemmas) is proof machinery for
+    //! the vectorised, deferred-modulo implementation in `algo.rs`. Nothing here is trusted:
+    //! every `spec fn` is a definition and every `proof fn` is checked by Verus.
+    #![allow(unused_imports)]
+    use vstd::prelude::*;
+}
+#[allow(unused_imports)]
+use crate::spec::*;
 use std::hash::Hasher;
 use std::io::{self, BufRead};
```

## C8

```diff
@@ -212,4 +261,8 @@
     }
 }
+#[doc = " The abstract state is the pair `(a, b)` of running sums."]
+impl View for Adler32 {
+    type V = (int, int);
+}
 impl Adler32 {
     #[doc = " Creates a new Adler-32 instance with default state."]
```

## C9

```diff
@@ -245,4 +298,5 @@
     #[inline]
     pub const fn from_checksum(sum: u32) -> Self {
+        {}
         Adler32 {
             a: sum as u16,
```

## C10

```diff
@@ -253,4 +307,5 @@
     #[inline]
     pub fn checksum(&self) -> u32 {
+        {}
         (u32::from(self.b) << 16) | u32::from(self.a)
     }
```

## Trusted / unsafe markers (annotated sources)

- **T1** `src/lib.rs:16` (`assume_specification`): `#![deny(unsafe_code)] // was `forbid`; Verus `assume_specification` expands to unsafe code (trusted.rs only)`
- **T2** `src/lib.rs:33` (`allow(unsafe_code)`): `#[allow(unsafe_code)]`
- **T3** `src/trusted.rs:18` (`external_type_specification`): `#[verifier::external_type_specification]`
- **T4** `src/trusted.rs:19` (`external_body`): `#[verifier::external_body]`
- **T5** `src/trusted.rs:24` (`uninterp`): `pub uninterp spec fn chunks_exact_rest<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;`
- **T6** `src/trusted.rs:27` (`uninterp`): `pub uninterp spec fn chunks_exact_size<'a, T>(it: ChunksExact<'a, T>) -> usize;`
- **T7** `src/trusted.rs:30` (`uninterp`): `pub uninterp spec fn chunks_exact_rem<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;`
- **T8** `src/trusted.rs:33` (`assume_specification`): `pub assume_specification<'a, T>[ <[T]>::chunks_exact ](s: &'a [T], chunk_size: usize) -> (it:`
- **T9** `src/trusted.rs:44` (`assume_specification`): `pub assume_specification<'a, T>[ ChunksExact::<'a, T>::remainder ](it: &ChunksExact<'a, T>) -> (r:`
- **T10** `src/trusted.rs:51` (`assume_specification`): `pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::next ](`

## Release assembly comparison (per function; symbol hashes and local labels normalised)

| fn | result |
|---|---|
| `write_slice` | IDENTICAL release assembly (268 lines) |
| `adler32_slice` | IDENTICAL release assembly (15 lines) |
