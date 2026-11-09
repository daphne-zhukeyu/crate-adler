//! Executable checks of claims in the upstream documentation (see ../evidence/DOC_REVIEW.md).
//! Each test names the doc claim it exercises. A test that passes while documenting a
//! surprising behaviour is evidence for a doc finding, not a crate bug.
use adler2::{adler32_slice, Adler32};
#[cfg(feature = "std")]
use adler2::adler32;
#[cfg(feature = "std")]
use std::io::{self, BufRead, Read};

/// Doc (lib.rs:31-33): "An instance of this type is equivalent to an Adler-32 checksum: It can be
/// created ... from a precalculated checksum via `from_checksum`".
/// Finding D1: for a u32 that is NOT a real Adler-32 checksum (a or b >= 65521), `from_checksum`
/// accepts it, `checksum()` returns it unchanged, and the first `write_slice` (even of nothing)
/// reduces it mod 65521 and changes the value. The docs don't say any of this.
#[test]
fn d1_from_checksum_non_canonical_state() {
    let mut h = Adler32::from_checksum(0xFFFF_FFFF);
    assert_eq!(h.checksum(), 0xFFFF_FFFF); // returned unchanged, though no stream produces it
    h.write_slice(&[]);
    assert_eq!(h.checksum(), 0x000E_000E); // 0xFFFF % 65521 == 0x000E in both halves
    // a canonical checksum is unaffected by the same empty write
    let mut g = Adler32::from_checksum(adler32_slice(b"Wikipedia"));
    g.write_slice(&[]);
    assert_eq!(g.checksum(), 0x11E60398);
}

/// Doc (lib.rs:99-102): `from_checksum` "allows resuming checksum calculation". Checked: for
/// every checksum the crate itself produces, resuming through `from_checksum` equals continuing.
#[test]
fn d2_from_checksum_roundtrip_on_canonical_states() {
    let data: Vec<u8> = (0..70_000u32).map(|i| (i.wrapping_mul(2654435761) >> 13) as u8).collect();
    for cut in [0usize, 1, 3, 4, 5, 4096, 5552 * 4, 5552 * 4 + 1, 22_209, 69_999, 70_000] {
        let mut a = Adler32::new();
        a.write_slice(&data[..cut]);
        let partial = a.checksum();
        a.write_slice(&data[cut..]);
        let mut b = Adler32::from_checksum(partial);
        b.write_slice(&data[cut..]);
        assert_eq!(a.checksum(), b.checksum(), "cut {cut}");
        assert_eq!(a.checksum(), adler32_slice(&data));
    }
}

#[cfg(feature = "std")]
struct FailingReader {
    chunks: Vec<&'static [u8]>,
}
#[cfg(feature = "std")]
impl Read for FailingReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        unreachable!()
    }
}
#[cfg(feature = "std")]
impl BufRead for FailingReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.chunks.is_empty() {
            Err(io::Error::new(io::ErrorKind::Other, "boom"))
        } else {
            Ok(self.chunks[0])
        }
    }
    fn consume(&mut self, n: usize) {
        let c = self.chunks[0];
        if n >= c.len() {
            self.chunks.remove(0);
        } else {
            self.chunks[0] = &c[n..];
        }
    }
}

/// Doc (lib.rs:177-186): `adler32` reads "until it reaches EOF (or until it reports an error)";
/// "Any error returned by the reader are bubbled up". Checked: an error after some data is
/// returned as Err, and a reader that hands out data in several buffers gives the slice result.
#[cfg(feature = "std")]
#[test]
fn d3_adler32_bufread_error_and_chunking() {
    let r = FailingReader { chunks: vec![b"Wiki", b"pedia"] };
    assert!(adler32(r).is_err());
    let ok = io::BufReader::with_capacity(3, &b"Wikipedia"[..]);
    assert_eq!(adler32(ok).unwrap(), 0x11E60398);
}

/// Doc (lib.rs:8): "Decent performance (3-4 GB/s)". Not a correctness claim; measured separately
/// (evidence/bench.log). Doc (lib.rs:7): "Zero `unsafe`": upstream has `#![forbid(unsafe_code)]`;
/// the annotated crate keeps `deny` for its executable code (evidence/code_changes/CODE_CHANGES.md).
#[test]
fn d4_wikipedia_vector_from_docs() {
    assert_eq!(adler32_slice(b"Wikipedia"), 0x11E60398);
}
