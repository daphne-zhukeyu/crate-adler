#![feature(prelude_import)]
//! Adler-32 checksum implementation.
//!
//! This implementation features:
//!
//! - Permissively licensed (0BSD) clean-room implementation.
//! - Zero dependencies.
//! - Zero `unsafe`.
//! - Decent performance (3-4 GB/s).
//! - `#![no_std]` support (with `default-features = false`).
#![doc(html_root_url = "https://docs.rs/adler2/2.0.0")]
// Deny a few warnings in doctests, since rustdoc `allow`s many warnings by default
#![doc(test(attr(deny(unused_imports, unused_must_use))))]
#![warn(missing_debug_implementations)]
#![forbid(unsafe_code)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
mod algo {
    // from above
    // from above
    // Make sure that we can resume computing from the partial checksum via `from_checksum`.
    // from above
    // `BufReader` uses an 8 KB buffer, so this will test buffer refilling.
    use crate::Adler32;
    use std::ops::{AddAssign, MulAssign, RemAssign};
    impl Adler32 {
        pub(crate) fn compute(&mut self, bytes: &[u8]) {
            const MOD: u32 = 65521;
            const CHUNK_SIZE: usize = 5552 * 4;
            let mut a = u32::from(self.a);
            let mut b = u32::from(self.b);
            let mut a_vec = U32X4([0; 4]);
            let mut b_vec = a_vec;
            let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);
            let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
            let remainder_chunk = chunk_iter.remainder();
            for chunk in chunk_iter {
                for byte_vec in chunk.chunks_exact(4) {
                    let val = U32X4::from(byte_vec);
                    a_vec += val;
                    b_vec += a_vec;
                }
                b += CHUNK_SIZE as u32 * a;
                a_vec %= MOD;
                b_vec %= MOD;
                b %= MOD;
            }
            for byte_vec in remainder_chunk.chunks_exact(4) {
                let val = U32X4::from(byte_vec);
                a_vec += val;
                b_vec += a_vec;
            }
            b += remainder_chunk.len() as u32 * a;
            a_vec %= MOD;
            b_vec %= MOD;
            b %= MOD;
            b_vec *= 4;
            b_vec.0[1] += MOD - a_vec.0[1];
            b_vec.0[2] += (MOD - a_vec.0[2]) * 2;
            b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
            for &av in a_vec.0.iter() {
                a += av;
            }
            for &bv in b_vec.0.iter() {
                b += bv;
            }
            for &byte in remainder.iter() {
                a += u32::from(byte);
                b += a;
            }
            self.a = (a % MOD) as u16;
            self.b = (b % MOD) as u16;
        }
    }
    struct U32X4([u32; 4]);
    #[automatically_derived]
    impl ::core::marker::Copy for U32X4 {}
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for U32X4 {}
    #[automatically_derived]
    impl ::core::clone::Clone for U32X4 {
        #[inline]
        fn clone(&self) -> U32X4 {
            let _: ::core::clone::AssertParamIsClone<[u32; 4]>;
            *self
        }
    }
    impl U32X4 {
        #[inline]
        fn from(bytes: &[u8]) -> Self {
            U32X4([
                u32::from(bytes[0]),
                u32::from(bytes[1]),
                u32::from(bytes[2]),
                u32::from(bytes[3]),
            ])
        }
    }
    impl AddAssign<Self> for U32X4 {
        #[inline]
        fn add_assign(&mut self, other: Self) {
            self.0[0] += other.0[0];
            self.0[1] += other.0[1];
            self.0[2] += other.0[2];
            self.0[3] += other.0[3];
        }
    }
    impl RemAssign<u32> for U32X4 {
        #[inline]
        fn rem_assign(&mut self, quotient: u32) {
            self.0[0] %= quotient;
            self.0[1] %= quotient;
            self.0[2] %= quotient;
            self.0[3] %= quotient;
        }
    }
    impl MulAssign<u32> for U32X4 {
        #[inline]
        fn mul_assign(&mut self, rhs: u32) {
            self.0[0] *= rhs;
            self.0[1] *= rhs;
            self.0[2] *= rhs;
            self.0[3] *= rhs;
        }
    }
}
use std::hash::Hasher;
use std::io::{self, BufRead};
#[doc = " Adler-32 checksum calculator."]
#[doc = ""]
#[doc = " An instance of this type is equivalent to an Adler-32 checksum: It can be created in the default"]
#[doc = " state via [`new`] (or the provided `Default` impl), or from a precalculated checksum via"]
#[doc = " [`from_checksum`], and the currently stored checksum can be fetched via [`checksum`]."]
#[doc = ""]
#[doc = " This type also implements `Hasher`, which makes it easy to calculate Adler-32 checksums of any"]
#[doc = " type that implements or derives `Hash`. This also allows using Adler-32 in a `HashMap`, although"]
#[doc = " that is not recommended (while every checksum is a hash function, they are not necessarily a"]
#[doc = " good one)."]
#[doc = ""]
#[doc = " # Examples"]
#[doc = ""]
#[doc = " Basic, piecewise checksum calculation:"]
#[doc = ""]
#[doc = " ```"]
#[doc = " use adler2::Adler32;"]
#[doc = ""]
#[doc = " let mut adler = Adler32::new();"]
#[doc = ""]
#[doc = " adler.write_slice(&[0, 1, 2]);"]
#[doc = " adler.write_slice(&[3, 4, 5]);"]
#[doc = ""]
#[doc = " assert_eq!(adler.checksum(), 0x00290010);"]
#[doc = " ```"]
#[doc = ""]
#[doc = " Using `Hash` to process structures:"]
#[doc = ""]
#[doc = " ```"]
#[doc = " use std::hash::Hash;"]
#[doc = " use adler2::Adler32;"]
#[doc = ""]
#[doc = " #[derive(Hash)]"]
#[doc = " struct Data {"]
#[doc = "     byte: u8,"]
#[doc = "     word: u16,"]
#[doc = "     big: u64,"]
#[doc = " }"]
#[doc = ""]
#[doc = " let mut adler = Adler32::new();"]
#[doc = ""]
#[doc = " let data = Data { byte: 0x1F, word: 0xABCD, big: !0 };"]
#[doc = " data.hash(&mut adler);"]
#[doc = ""]
#[doc = " // hash value depends on architecture endianness"]
#[doc = " if cfg!(target_endian = \"little\") {"]
#[doc = "     assert_eq!(adler.checksum(), 0x33410990);"]
#[doc = " }"]
#[doc = " if cfg!(target_endian = \"big\") {"]
#[doc = "     assert_eq!(adler.checksum(), 0x331F0990);"]
#[doc = " }"]
#[doc = ""]
#[doc = " ```"]
#[doc = ""]
#[doc = " [`new`]: #method.new"]
#[doc = " [`from_checksum`]: #method.from_checksum"]
#[doc = " [`checksum`]: #method.checksum"]
pub struct Adler32 {
    a: u16,
    b: u16,
}
#[automatically_derived]
impl ::core::fmt::Debug for Adler32 {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f, "Adler32", "a", &self.a, "b", &&self.b,
        )
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Adler32 {}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for Adler32 {}
#[automatically_derived]
impl ::core::clone::Clone for Adler32 {
    #[inline]
    fn clone(&self) -> Adler32 {
        let _: ::core::clone::AssertParamIsClone<u16>;
        *self
    }
}
impl Adler32 {
    #[doc = " Creates a new Adler-32 instance with default state."]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[doc = " Creates an `Adler32` instance from a precomputed Adler-32 checksum."]
    #[doc = ""]
    #[doc = " This allows resuming checksum calculation without having to keep the `Adler32` instance"]
    #[doc = " around."]
    #[doc = ""]
    #[doc = " # Example"]
    #[doc = ""]
    #[doc = " ```"]
    #[doc = " # use adler2::Adler32;"]
    #[doc = " let parts = ["]
    #[doc = "     \"rust\","]
    #[doc = "     \"acean\","]
    #[doc = " ];"]
    #[doc = " let whole = adler2::adler32_slice(b\"rustacean\");"]
    #[doc = ""]
    #[doc = " let mut sum = Adler32::new();"]
    #[doc = " sum.write_slice(parts[0].as_bytes());"]
    #[doc = " let partial = sum.checksum();"]
    #[doc = ""]
    #[doc = " // ...later"]
    #[doc = ""]
    #[doc = " let mut sum = Adler32::from_checksum(partial);"]
    #[doc = " sum.write_slice(parts[1].as_bytes());"]
    #[doc = " assert_eq!(sum.checksum(), whole);"]
    #[doc = " ```"]
    #[inline]
    pub const fn from_checksum(sum: u32) -> Self {
        Adler32 {
            a: sum as u16,
            b: (sum >> 16) as u16,
        }
    }
    #[doc = " Returns the calculated checksum at this point in time."]
    #[inline]
    pub fn checksum(&self) -> u32 {
        (u32::from(self.b) << 16) | u32::from(self.a)
    }
    #[doc = " Adds `bytes` to the checksum calculation."]
    #[doc = ""]
    #[doc = " If efficiency matters, this should be called with Byte slices that contain at least a few"]
    #[doc = " thousand Bytes."]
    pub fn write_slice(&mut self, bytes: &[u8]) {
        self.compute(bytes);
    }
}
impl Default for Adler32 {
    #[inline]
    fn default() -> Self {
        Adler32 { a: 1, b: 0 }
    }
}
impl Hasher for Adler32 {
    #[inline]
    fn finish(&self) -> u64 {
        u64::from(self.checksum())
    }
    fn write(&mut self, bytes: &[u8]) {
        self.write_slice(bytes);
    }
}
#[doc = " Calculates the Adler-32 checksum of a byte slice."]
#[doc = ""]
#[doc = " This is a convenience function around the [`Adler32`] type."]
#[doc = ""]
#[doc = " [`Adler32`]: struct.Adler32.html"]
pub fn adler32_slice(data: &[u8]) -> u32 {
    let mut h = Adler32::new();
    h.write_slice(data);
    h.checksum()
}
#[doc = " Calculates the Adler-32 checksum of a `BufRead`'s contents."]
#[doc = ""]
#[doc = " The passed `BufRead` implementor will be read until it reaches EOF (or until it reports an"]
#[doc = " error)."]
#[doc = ""]
#[doc = " If you only have a `Read` implementor, you can wrap it in `std::io::BufReader` before calling"]
#[doc = " this function."]
#[doc = ""]
#[doc = " # Errors"]
#[doc = ""]
#[doc = " Any error returned by the reader are bubbled up by this function."]
#[doc = ""]
#[doc = " # Examples"]
#[doc = ""]
#[doc = " ```no_run"]
#[doc = " # fn run() -> Result<(), Box<dyn std::error::Error>> {"]
#[doc = " use adler2::adler32;"]
#[doc = ""]
#[doc = " use std::fs::File;"]
#[doc = " use std::io::BufReader;"]
#[doc = ""]
#[doc = " let file = File::open(\"input.txt\")?;"]
#[doc = " let mut file = BufReader::new(file);"]
#[doc = ""]
#[doc = " adler32(&mut file)?;"]
#[doc = " # Ok(()) }"]
#[doc = " # fn main() { run().unwrap() }"]
#[doc = " ```"]
pub fn adler32<R: BufRead>(mut reader: R) -> io::Result<u32> {
    let mut h = Adler32::new();
    loop {
        let len = {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                return Ok(h.checksum());
            }
            h.write_slice(buf);
            buf.len()
        };
        reader.consume(len);
    }
}
