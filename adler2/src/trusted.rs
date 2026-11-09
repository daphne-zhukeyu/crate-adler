//! TRUSTED declarations: seams to the Rust standard library that Verus cannot check.
//! Compiled only under Verus (`cfg(verus_keep_ghost)`); contains no executable code.
//!
//! | id | item | why trusted |
//! |----|------|-------------|
//! | T1 | `<[T]>::chunks_exact` | vstd 0.2026.10.04 has no spec for `chunks_exact`/`ChunksExact` (Verus main has one only for `chunks`) |
//! | T2 | `ChunksExact::remainder` | same |
//! | T3 | `<ChunksExact as Iterator>::next` | same; vstd's `IteratorSpec` cannot be implemented for a core type outside vstd (orphan rule), so `for` loops over `ChunksExact` cannot be specified from this crate |
//!
//! The model mirrors vstd's own (equally trusted) specification of `<[T]>::chunks` on Verus main
//! and the std documentation: `chunks_exact(n)` yields the `len / n` full chunks of `n` items in
//! order, and `remainder()` is the last `len % n` items, independent of iteration progress.
use core::slice::ChunksExact;
use vstd::prelude::*;

verus! {

#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::accept_recursive_types(T)]
pub struct ExChunksExact<'a, T: 'a>(ChunksExact<'a, T>);

/// The items of the full chunks not yet returned by the iterator, concatenated.
pub uninterp spec fn chunks_exact_rest<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;

/// The chunk size the iterator was created with.
pub uninterp spec fn chunks_exact_size<'a, T>(it: ChunksExact<'a, T>) -> usize;

/// The items that do not fill a whole chunk (what `remainder()` returns).
pub uninterp spec fn chunks_exact_rem<'a, T>(it: ChunksExact<'a, T>) -> Seq<T>;

// T1
pub assume_specification<'a, T>[ <[T]>::chunks_exact ](s: &'a [T], chunk_size: usize) -> (it:
    ChunksExact<'a, T>)
    requires
        chunk_size > 0,
    ensures
        chunks_exact_size(it) == chunk_size,
        chunks_exact_rest(it) == s@.take(s@.len() as int - (s@.len() as int) % (chunk_size as int)),
        chunks_exact_rem(it) == s@.skip(s@.len() as int - (s@.len() as int) % (chunk_size as int)),
;

// T2
pub assume_specification<'a, T>[ ChunksExact::<'a, T>::remainder ](it: &ChunksExact<'a, T>) -> (r:
    &'a [T])
    ensures
        r@ == chunks_exact_rem(*it),
;

// T3
pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::next ](
    it: &mut ChunksExact<'a, T>,
) -> (r: Option<&'a [T]>)
    ensures
        chunks_exact_size(*final(it)) == chunks_exact_size(*old(it)),
        chunks_exact_rem(*final(it)) == chunks_exact_rem(*old(it)),
        chunks_exact_rest(*old(it)).len() < chunks_exact_size(*old(it)) ==> {
            &&& r is None
            &&& chunks_exact_rest(*final(it)) == chunks_exact_rest(*old(it))
        },
        chunks_exact_rest(*old(it)).len() >= chunks_exact_size(*old(it)) ==> {
            &&& r is Some
            &&& r->Some_0@ == chunks_exact_rest(*old(it)).take(chunks_exact_size(*old(it)) as int)
            &&& chunks_exact_rest(*final(it)) == chunks_exact_rest(*old(it)).skip(
                chunks_exact_size(*old(it)) as int,
            )
        },
;

} // verus!
