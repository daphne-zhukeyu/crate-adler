//! Runtime check of the trusted `ChunksExact` model in `src/trusted.rs` (T1-T3) against the
//! real standard library. Verus cannot check those three specifications, so this test is the
//! evidence that they describe what std does, for every length and chunk size in a range.

fn check(len: usize, n: usize) {
    let s: Vec<u8> = (0..len).map(|i| i as u8).collect();
    let cut = len - len % n;

    // T1: chunks_exact(n) holds rest = s[..cut] and rem = s[cut..].
    let mut it = s.chunks_exact(n);
    // T2: remainder() is s[cut..], before and during iteration.
    assert_eq!(it.remainder(), &s[cut..]);

    // T3: each next() returns the first n items of the rest, until fewer than n remain,
    // then None (and stays None).
    let mut rest: &[u8] = &s[..cut];
    loop {
        let r = it.next();
        assert_eq!(it.remainder(), &s[cut..]);
        if rest.len() < n {
            assert!(r.is_none());
            assert!(it.next().is_none());
            break;
        }
        assert_eq!(r, Some(&rest[..n]));
        rest = &rest[n..];
    }
}

#[test]
fn chunks_exact_model_matches_std() {
    for n in [1, 2, 3, 4, 5, 7, 8, 16, 5552 * 4] {
        for len in 0..200 {
            check(len, n);
        }
        for len in [n * 3 - 1, n * 3, n * 3 + 1, 5552 * 4 * 2 + 3] {
            check(len, n);
        }
    }
}
