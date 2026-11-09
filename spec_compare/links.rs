//! Linking lemmas between the Claude spec (`c`) and the gold spec (`h`), one pair per public
//! function. Written by Claude on 2026-11-08 after the gold spec was frozen (human_written_spec.v1.rs).
//!
//! Shape: `claude_implies_human_f` assumes the crate's verified contract for `f` (as stated in
//! `../adler2/src/lib.rs`, in terms of `c`) and concludes the gold spec's statement;
//! `human_implies_claude_f` is the converse.
//!
//! State abstractions differ: the crate's view `Adler32@` is `(int, int)` (the two `u16` fields as
//! integers); the gold spec's `State` is `(u16, u16)`. `hv` maps the latter to the former.
//!
//! Expected outcome, stated before running: the unconditional `write_slice` obligations fail in
//! both directions. The gold spec says an empty write leaves a non-canonical state (a field
//! >= 65521, reachable only through `from_checksum`) untouched; the crate reduces both fields
//! modulo 65521 on every `write_slice`, including an empty one (verified contract
//! `adler_from(old.0 % 65521, old.1 % 65521, ...)`, and `tests/doc_findings.rs::d1`). The two
//! specs agree whenever the state is canonical or the slice is non-empty; those are the
//! `write_slice_canonical` and `write_slice_nonempty` rows. `counterexample_write_slice_empty`
//! exhibits the disagreement by computation.
#![allow(unused_imports)]
use vstd::prelude::*;
use vstd::arithmetic::div_mod::{lemma_add_mod_noop, lemma_mod_twice, lemma_mod_bound, lemma_small_mod};
use crate::claude_spec as c;
use crate::human_written_spec as h;

verus! {

/// Gold state as the crate's view.
pub open spec fn hv(s: h::State) -> (int, int) {
    (s.0 as int, s.1 as int)
}

pub open spec fn canonical(s: h::State) -> bool {
    s.0 < 65521 && s.1 < 65521
}

/// One round agrees once the crate's state is the gold state reduced mod 65521.
proof fn lemma_step_agree(s: h::State, d: u8)
    ensures
        c::adler_step((s.0 as int % 65521, s.1 as int % 65521), d) == hv(h::step(s, d)),
{
    let a0 = s.0 as int;
    let b0 = s.1 as int;
    // claude: a' = ((a0 % M) + d) % M;  gold: s1 = (a0 + d) % M
    lemma_add_mod_noop(a0, d as int, 65521);
    lemma_mod_twice(d as int, 65521);
    lemma_mod_twice(a0, 65521);
    assert((a0 % 65521 + d) % 65521 == (a0 + d) % 65521) by {
        lemma_add_mod_noop(a0 % 65521, d as int, 65521);
    }
    let a1 = (a0 + d) % 65521;
    // claude: b' = ((b0 % M) + a') % M;  gold: s2 = (b0 + s1) % M
    lemma_add_mod_noop(b0, a1, 65521);
    lemma_add_mod_noop(b0 % 65521, a1, 65521);
    lemma_mod_twice(b0, 65521);
    lemma_mod_twice(a1, 65521);
    lemma_mod_bound(a0 + d, 65521);
    lemma_mod_bound(b0 + a1, 65521);
}

/// The folds agree from any state when the slice is non-empty (the first round reduces both
/// fields) and from a canonical state always.
proof fn lemma_fold_agree(s: h::State, bytes: Seq<u8>)
    requires
        bytes.len() > 0 || canonical(s),
    ensures
        c::adler_from(s.0 as int % 65521, s.1 as int % 65521, bytes) == hv(h::adler_from(s, bytes)),
    decreases bytes.len(),
{
    if bytes.len() == 0 {
        // canonical: x % 65521 == x (linear, literal modulus)
        assert(s.0 as int % 65521 == s.0 as int);
        assert(s.1 as int % 65521 == s.1 as int);
        assert(c::adler_from(s.0 as int % 65521, s.1 as int % 65521, bytes) == (s.0 as int, s.1 as int));
        assert(hv(h::adler_from(s, bytes)) == (s.0 as int, s.1 as int));
    } else {
        let p = bytes.drop_last();
        let d = bytes.last();
        assert(c::adler_from(s.0 as int % 65521, s.1 as int % 65521, bytes)
            == c::adler_step(c::adler_from(s.0 as int % 65521, s.1 as int % 65521, p), d));
        assert(h::adler_from(s, bytes) == h::step(h::adler_from(s, p), d));
        if p.len() == 0 {
            assert(c::adler_from(s.0 as int % 65521, s.1 as int % 65521, p) == (s.0 as int % 65521, s.1 as int % 65521));
            assert(h::adler_from(s, p) == s);
            lemma_step_agree(s, d);
        } else {
            lemma_fold_agree(s, p);
            let m = h::adler_from(s, p);
            // after at least one round the gold state is canonical, so reducing it is a no-op
            assert(canonical(m)) by {
                lemma_fold_canonical(s, p);
            }
            assert(m.0 as int % 65521 == m.0 as int);
            assert(m.1 as int % 65521 == m.1 as int);
            lemma_step_agree(m, d);
            assert(c::adler_step(hv(m), d) == hv(h::step(m, d)));
        }
    }
}

/// After at least one round the gold state has both fields below 65521.
proof fn lemma_fold_canonical(s: h::State, bytes: Seq<u8>)
    requires
        bytes.len() > 0,
    ensures
        canonical(h::adler_from(s, bytes)),
    decreases bytes.len(),
{
    let m = h::adler_from(s, bytes.drop_last());
    lemma_mod_bound(m.0 as int + bytes.last() as int, 65521);
    lemma_mod_bound(m.1 as int + (m.0 as int + bytes.last() as int) % 65521, 65521);
}

// ---- Adler32::new / Default: crate ensures `r@ == (1, 0)`; gold: `new_result()` / `default_result()`

pub proof fn claude_implies_human_new(v: (int, int))
    requires
        v == (1int, 0int),
    ensures
        v == hv(h::new_result()),
{
}

pub proof fn human_implies_claude_new(s: h::State)
    requires
        s == h::new_result(),
    ensures
        hv(s) == (1int, 0int),
{
}

pub proof fn claude_implies_human_default(v: (int, int))
    requires
        v == (1int, 0int),
    ensures
        v == hv(h::default_result()),
{
}

pub proof fn human_implies_claude_default(s: h::State)
    requires
        s == h::default_result(),
    ensures
        hv(s) == (1int, 0int),
{
}

// ---- Adler32::from_checksum(sum): crate ensures `r@ == (sum % 65536, sum / 65536)`;
//      gold: `from_checksum_result(sum)`

pub proof fn claude_implies_human_from_checksum(sum: u32, v: (int, int))
    requires
        v == (sum as int % 65536, sum as int / 65536),
    ensures
        v == hv(h::from_checksum_result(sum)),
{
    lemma_mod_bound(sum as int, 65536);
    assert(sum as int / 65536 < 65536) by (nonlinear_arith)
        requires
            sum <= 0xffff_ffffu32,
    ;
}

pub proof fn human_implies_claude_from_checksum(sum: u32, s: h::State)
    requires
        s == h::from_checksum_result(sum),
    ensures
        hv(s) == (sum as int % 65536, sum as int / 65536),
{
    lemma_mod_bound(sum as int, 65536);
    assert(sum as int / 65536 < 65536) by (nonlinear_arith)
        requires
            sum <= 0xffff_ffffu32,
    ;
}

// ---- Adler32::checksum: crate ensures `r == pack(self@)`; gold: `checksum_result(state)`

pub proof fn claude_implies_human_checksum(s: h::State, r: u32)
    requires
        r == c::pack(hv(s)),
    ensures
        r == h::checksum_result(s),
{
}

pub proof fn human_implies_claude_checksum(s: h::State, r: u32)
    requires
        r == h::checksum_result(s),
    ensures
        r == c::pack(hv(s)),
{
}

// ---- Hasher::finish: crate ensures `r == pack(self@)` (as u64); gold: `finish_result(state)`

pub proof fn claude_implies_human_finish(s: h::State, r: u64)
    requires
        r == c::pack(hv(s)),
    ensures
        r == h::finish_result(s),
{
}

pub proof fn human_implies_claude_finish(s: h::State, r: u64)
    requires
        r == h::finish_result(s),
    ensures
        r == c::pack(hv(s)),
{
}

// ---- Adler32::write_slice(bytes) (and Hasher::write, same contract):
//      crate ensures `final(self)@ == adler_from(old.0 % 65521, old.1 % 65521, bytes@)`;
//      gold: `write_slice_next(old, bytes)`. Unconditional obligations: EXPECTED TO FAIL (see header).

pub proof fn claude_implies_human_write_slice(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
    ensures
        new == hv(h::write_slice_next(old, bytes)),
{
    if bytes.len() > 0 || canonical(old) {
        lemma_fold_agree(old, bytes);
    }
}

pub proof fn human_implies_claude_write_slice(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        new == hv(h::write_slice_next(old, bytes)),
    ensures
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
{
    if bytes.len() > 0 || canonical(old) {
        lemma_fold_agree(old, bytes);
    }
}

// Conditional variants: where the two specs agree.

pub proof fn claude_implies_human_write_slice_canonical(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        canonical(old),
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
    ensures
        new == hv(h::write_slice_next(old, bytes)),
{
    lemma_fold_agree(old, bytes);
}

pub proof fn human_implies_claude_write_slice_canonical(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        canonical(old),
        new == hv(h::write_slice_next(old, bytes)),
    ensures
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
{
    lemma_fold_agree(old, bytes);
}

pub proof fn claude_implies_human_write_slice_nonempty(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        bytes.len() > 0,
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
    ensures
        new == hv(h::write_slice_next(old, bytes)),
{
    lemma_fold_agree(old, bytes);
}

pub proof fn human_implies_claude_write_slice_nonempty(old: h::State, bytes: Seq<u8>, new: (int, int))
    requires
        bytes.len() > 0,
        new == hv(h::write_slice_next(old, bytes)),
    ensures
        new == c::adler_from(old.0 as int % 65521, old.1 as int % 65521, bytes),
{
    lemma_fold_agree(old, bytes);
}

/// The disagreement, by computation: state restored from 0xFFFF_FFFF, empty write.
/// Crate: (14, 14). Gold: (65535, 65535). The crate's behaviour is what the code does
/// (`../adler2/tests/doc_findings.rs::d1_from_checksum_non_canonical_state`).
proof fn counterexample_write_slice_empty() {
    assert(c::adler_from(0xffffint % 65521int, 0xffffint % 65521int, Seq::<u8>::empty()) == (14int, 14int)) by (compute);
    assert(hv(h::write_slice_next((0xffffu16, 0xffffu16), Seq::<u8>::empty())) == (65535int, 65535int)) by (compute);
}

// ---- adler32_slice(data): crate ensures `r == adler32_spec(data)`; gold: `adler32_slice_result(data)`

pub proof fn claude_implies_human_adler32_slice(data: Seq<u8>, r: u32)
    requires
        r == c::adler32_spec(data),
    ensures
        r == h::adler32_slice_result(data),
{
    lemma_fold_agree((1u16, 0u16), data);
}

pub proof fn human_implies_claude_adler32_slice(data: Seq<u8>, r: u32)
    requires
        r == h::adler32_slice_result(data),
    ensures
        r == c::adler32_spec(data),
{
    lemma_fold_agree((1u16, 0u16), data);
}

} // verus!
