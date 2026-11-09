// Gold spec for adler2 2.0.1.
// Sources: the public API, upstream test vectors, and RFC 1950.
// Scope: Adler32 and adler32_slice. The BufRead API is outside this model.

use vstd::prelude::*;

verus! {

/// Running sums in (s1, s2) order. Each component fits in a 16-bit field.
/// A state restored from an arbitrary u32 may have noncanonical residues.
pub type State = (u16, u16);

/// RFC 1950 reduces both sums modulo 65521 after each byte.
/// Update s1 first, then use the new s1 to update s2.
pub open spec fn step(state: State, byte: u8) -> State {
    let s1 = (state.0 as int + byte as int) % 65521int;
    let s2 = (state.1 as int + s1) % 65521int;
    (s1 as u16, s2 as u16)
}

/// Fold left from the supplied state. The empty sequence is the identity,
/// and the state after a prefix is the starting state for the suffix.
pub open spec fn adler_from(state: State, bytes: Seq<u8>) -> State
    decreases bytes.len(),
{
    if bytes.len() == 0 {
        state
    } else {
        step(adler_from(state, bytes.drop_last()), bytes.last())
    }
}

/// `Adler32::new` starts at the RFC initial state.
pub open spec fn new_result() -> State {
    (1u16, 0u16)
}

/// `Default::default` gives the same initial state as `Adler32::new`.
pub open spec fn default_result() -> State {
    new_result()
}

/// `Adler32::from_checksum` restores the two packed fields verbatim.
/// For noncanonical inputs, an empty write preserves the raw fields.
/// The first byte of a nonempty write reduces both sums modulo 65521.
pub open spec fn from_checksum_result(sum: u32) -> State {
    (((sum as int) % 65536int) as u16, ((sum as int) / 65536int) as u16)
}

/// `Adler32::checksum` packs s1 into the low half and s2 into the high half.
pub open spec fn checksum_result(state: State) -> u32 {
    (state.1 as int * 65536int + state.0 as int) as u32
}

/// `Adler32::write_slice` continues the checksum from the current state.
pub open spec fn write_slice_next(state: State, bytes: Seq<u8>) -> State {
    adler_from(state, bytes)
}

/// `Hasher::write` has the same state transition as `write_slice`.
pub open spec fn write_next(state: State, bytes: Seq<u8>) -> State {
    write_slice_next(state, bytes)
}

/// `Hasher::finish` zero-extends the packed 32-bit checksum.
pub open spec fn finish_result(state: State) -> u64 {
    checksum_result(state) as u64
}

/// `adler32_slice` checksums the whole input from the RFC initial state.
pub open spec fn adler32_slice_result(data: Seq<u8>) -> u32 {
    checksum_result(write_slice_next(new_result(), data))
}

proof fn vectors() {
    assert(adler32_slice_result(Seq::<u8>::empty()) == 1u32) by (compute);
    assert(adler32_slice_result(seq![0u8]) == 0x00010001u32) by (compute);
    assert(adler32_slice_result(seq![1u8]) == 0x00020002u32) by (compute);
    assert(adler32_slice_result(seq![40u8]) == 0x00290029u32) by (compute);
    assert(adler32_slice_result(seq![0u8, 1, 2, 3, 4, 5])
        == 0x00290010u32) by (compute);
    assert(adler32_slice_result(seq![0x57u8, 0x69, 0x6b, 0x69, 0x70, 0x65, 0x64, 0x69, 0x61])
        == 0x11e60398u32) by (compute);

    let prefix = write_slice_next(default_result(), seq![0u8, 1, 2]);
    let suffix = seq![3u8, 4, 5];
    let split = write_next(prefix, suffix);
    assert(finish_result(split) == 0x00290010u64) by (compute);

    let saved = checksum_result(prefix);
    let restored = from_checksum_result(saved);
    assert(checksum_result(restored) == saved) by (compute);
    assert(write_slice_next(restored, suffix) == split) by (compute);

    let raw = from_checksum_result(0xffffffffu32);
    assert(checksum_result(write_slice_next(raw, Seq::<u8>::empty()))
        == 0xffffffffu32) by (compute);
    assert(checksum_result(write_slice_next(raw, seq![0u8]))
        == 0x001c000eu32) by (compute);
}

} // verus!
