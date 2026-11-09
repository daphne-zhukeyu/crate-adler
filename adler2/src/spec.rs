//! Specification of Adler-32 (ghost code only; erased from normal builds).
//!
//! The contracts in `lib.rs` are stated with `adler_from` and `adler32_spec`, the textbook
//! per-byte definition from RFC 1950 §8.2:
//!
//! ```text
//! A = 1 + D1 + D2 + ... + Dn (mod 65521)
//! B = (1 + D1) + (1 + D1 + D2) + ... + (1 + D1 + D2 + ... + Dn) (mod 65521)
//! Adler-32(D) = B * 65536 + A
//! ```
//!
//! Everything else here (unreduced sums, lane sums and the lemmas) is proof machinery for
//! the vectorised, deferred-modulo implementation in `algo.rs`. Nothing here is trusted:
//! every `spec fn` is a definition and every `proof fn` is checked by Verus.
#![allow(unused_imports)]

#[cfg(verus_keep_ghost)] // ghost items: they do not exist in the erased (normal) build
use vstd::arithmetic::div_mod::{
    lemma_add_mod_noop, lemma_fundamental_div_mod, lemma_mod_bound, lemma_mod_multiples_vanish,
    lemma_mod_twice, lemma_sub_mod_noop,
};
use vstd::prelude::*;

verus! {

// ---------------------------------------------------------------- the specification

/// The Adler-32 modulus (RFC 1950 calls it BASE), the largest prime below 2^16.
pub open spec fn base() -> int {
    65521
}

/// One Adler-32 round on the state `(a, b)`.
pub open spec fn adler_step(ab: (int, int), d: u8) -> (int, int) {
    let a = (ab.0 + d) % base();
    (a, (ab.1 + a) % base())
}

/// Adler-32 rounds over `data` from the state `(a, b)` (first byte first).
pub open spec fn adler_from(a: int, b: int, data: Seq<u8>) -> (int, int)
    decreases data.len(),
{
    if data.len() == 0 {
        (a, b)
    } else {
        adler_step(adler_from(a, b, data.drop_last()), data.last())
    }
}

/// Packs the state into the 32-bit checksum: `b` in the high half, `a` in the low half.
pub open spec fn pack(ab: (int, int)) -> int {
    ab.1 * 65536 + ab.0
}

/// The Adler-32 checksum of `data` (initial state `a = 1, b = 0`).
pub open spec fn adler32_spec(data: Seq<u8>) -> int {
    pack(adler_from(1, 0, data))
}

// ---------------------------------------------------------------- unreduced sums

/// `D1 + ... + Dn`.
pub open spec fn sum(s: Seq<u8>) -> int
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        sum(s.drop_last()) + s.last()
    }
}

/// Sum of the prefix sums of `s`, i.e. `n*D1 + (n-1)*D2 + ... + 1*Dn`.
pub open spec fn wsum(s: Seq<u8>) -> int
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        wsum(s.drop_last()) + sum(s)
    }
}

/// Sum of lane `j` over the first `m` groups of 4: `s[j] + s[4+j] + ... + s[4(m-1)+j]`.
pub open spec fn lane_sum(s: Seq<u8>, j: int, m: int) -> int
    decreases m,
{
    if m <= 0 {
        0
    } else {
        lane_sum(s, j, m - 1) + s[4 * (m - 1) + j]
    }
}

/// Running total of `lane_sum` over groups, as the vector loop's `b_vec += a_vec` builds it.
pub open spec fn lane_wsum(s: Seq<u8>, j: int, m: int) -> int
    decreases m,
{
    if m <= 0 {
        0
    } else {
        lane_wsum(s, j, m - 1) + lane_sum(s, j, m)
    }
}

pub open spec fn lanes_sum(s: Seq<u8>, m: int) -> int {
    lane_sum(s, 0, m) + lane_sum(s, 1, m) + lane_sum(s, 2, m) + lane_sum(s, 3, m)
}

/// What the combine step of `compute` adds to `b`: `sum_j (4*lane_wsum_j - j*lane_sum_j)`.
pub open spec fn lanes_wsum(s: Seq<u8>, m: int) -> int {
    4 * (lane_wsum(s, 0, m) + lane_wsum(s, 1, m) + lane_wsum(s, 2, m) + lane_wsum(s, 3, m))
        - lane_sum(s, 1, m) - 2 * lane_sum(s, 2, m) - 3 * lane_sum(s, 3, m)
}

/// `0 + 1 + ... + t`.
pub open spec fn tri(t: int) -> int
    decreases t,
{
    if t <= 0 {
        0
    } else {
        tri(t - 1) + t
    }
}

/// Bound on `b_vec` lanes after `t` groups since the last reduction (given lanes start `< base()`):
/// the `n*65520 + n(n+1)/2*255` of the comment in `compute`, plus the starting value.
pub open spec fn b_vec_bound(t: int) -> int {
    (base() - 1) * (t + 1) + 255 * tri(t)
}

// ---------------------------------------------------------------- lemmas

pub proof fn lemma_sum_bounds(s: Seq<u8>)
    ensures
        0 <= sum(s) <= 255 * s.len(),
        0 <= wsum(s),
    decreases s.len(),
{
    if s.len() > 0 {
        lemma_sum_bounds(s.drop_last());
    }
}

pub proof fn lemma_sum_append(x: Seq<u8>, y: Seq<u8>)
    ensures
        sum(x + y) == sum(x) + sum(y),
        wsum(x + y) == wsum(x) + y.len() * sum(x) + wsum(y),
    decreases y.len(),
{
    if y.len() == 0 {
        assert(x + y =~= x);
    } else {
        lemma_sum_append(x, y.drop_last());
        assert((x + y).drop_last() =~= x + y.drop_last());
        assert(y.len() * sum(x) == (y.len() - 1) * sum(x) + sum(x)) by (nonlinear_arith);
    }
}

/// The deferred-modulo identity: Adler-32 rounds from reduced `(a, b)` equal the unreduced
/// sums reduced once at the end.
pub proof fn lemma_adler_from_unreduced(a: int, b: int, s: Seq<u8>)
    requires
        a >= 0,
        b >= 0,
    ensures
        adler_from(a % base(), b % base(), s) == (
            (a + sum(s)) % base(),
            (b + s.len() * a + wsum(s)) % base(),
        ),
    decreases s.len(),
{
    if s.len() > 0 {
        let s1 = s.drop_last();
        let d = s.last();
        lemma_adler_from_unreduced(a, b, s1);
        lemma_sum_bounds(s1);
        let x = (a + sum(s1)) % base();
        let y = (b + s1.len() * a + wsum(s1)) % base();
        assert(adler_from(a % base(), b % base(), s) == adler_step((x, y), d));
        // a' = (x + d) % MOD == (a + sum(s)) % MOD
        lemma_add_mod_noop(a + sum(s1), d as int, base());
        lemma_mod_twice(d as int, base());
        assert((x + d) % base() == (a + sum(s)) % base()) by {
            lemma_add_mod_noop(x, d as int, base());
            lemma_mod_twice(a + sum(s1), base());
        }
        let a2 = (a + sum(s)) % base();
        // b' = (y + a') % MOD == (b + n*a + wsum(s)) % MOD
        lemma_add_mod_noop(b + s1.len() * a + wsum(s1), a + sum(s), base());
        lemma_add_mod_noop(y, a2, base());
        lemma_mod_twice(b + s1.len() * a + wsum(s1), base());
        lemma_mod_twice(a + sum(s), base());
        assert(s.len() * a == s1.len() * a + a) by (nonlinear_arith)
            requires
                s.len() == s1.len() + 1,
        ;
    }
}

/// The lane decomposition behind the 4-way unrolled loop: for the first `4m` bytes,
/// `sum` is the sum of the 4 lane sums and `wsum` is `lanes_wsum`.
pub proof fn lemma_lanes(s: Seq<u8>, m: int)
    requires
        0 <= m,
        4 * m <= s.len(),
    ensures
        sum(s.take(4 * m)) == lanes_sum(s, m),
        wsum(s.take(4 * m)) == lanes_wsum(s, m),
    decreases m,
{
    if m > 0 {
        lemma_lanes(s, m - 1);
        lemma_lane_step(s, m - 1);
        let t = s.take(4 * (m - 1));
        let g = s.subrange(4 * (m - 1), 4 * m);
        assert(s.take(4 * m) =~= t + g);
        lemma_sum_append(t, g);
        // unfold sum / wsum of the 4-byte group
        assert(g.drop_last().drop_last().drop_last().drop_last() =~= Seq::<u8>::empty());
        assert(sum(g) == g[0] + g[1] + g[2] + g[3]) by {
            reveal_with_fuel(sum, 5);
        }
        assert(wsum(g) == 4 * g[0] + 3 * g[1] + 2 * g[2] + g[3]) by {
            reveal_with_fuel(wsum, 5);
            reveal_with_fuel(sum, 5);
        }
        let b = 4 * (m - 1);
        assert(g[0] == s[b] && g[1] == s[b + 1] && g[2] == s[b + 2] && g[3] == s[b + 3]);
        assert(lane_sum(s, 0, m) == lane_sum(s, 0, m - 1) + s[b]);
        assert(lane_sum(s, 1, m) == lane_sum(s, 1, m - 1) + s[b + 1]);
        assert(lane_sum(s, 2, m) == lane_sum(s, 2, m - 1) + s[b + 2]);
        assert(lane_sum(s, 3, m) == lane_sum(s, 3, m - 1) + s[b + 3]);
        assert(lane_wsum(s, 0, m) == lane_wsum(s, 0, m - 1) + lane_sum(s, 0, m));
        assert(lane_wsum(s, 1, m) == lane_wsum(s, 1, m - 1) + lane_sum(s, 1, m));
        assert(lane_wsum(s, 2, m) == lane_wsum(s, 2, m - 1) + lane_sum(s, 2, m));
        assert(lane_wsum(s, 3, m) == lane_wsum(s, 3, m - 1) + lane_sum(s, 3, m));
    }
}

/// Group `m` of `s` (bytes `4m .. 4m+4`) extends each lane by one element.
pub proof fn lemma_lane_step(s: Seq<u8>, m: int)
    requires
        0 <= m,
    ensures
        forall|j: int|
            lane_sum(s, j, m + 1) == lane_sum(s, j, m) + s[4 * m + j],
        forall|j: int|
            lane_wsum(s, j, m + 1) == lane_wsum(s, j, m) + lane_sum(s, j, m + 1),
{
}

/// Lane sums only look at the first `4m` bytes.
pub proof fn lemma_lane_prefix(s: Seq<u8>, t: Seq<u8>, j: int, m: int)
    requires
        0 <= j < 4,
        0 <= m,
        4 * m <= s.len(),
        4 * m <= t.len(),
        s.take(4 * m) == t.take(4 * m),
    ensures
        lane_sum(s, j, m) == lane_sum(t, j, m),
        lane_wsum(s, j, m) == lane_wsum(t, j, m),
    decreases m,
{
    if m > 0 {
        assert(s.take(4 * (m - 1)) =~= t.take(4 * (m - 1))) by {
            assert(s.take(4 * (m - 1)) =~= s.take(4 * m).take(4 * (m - 1)));
            assert(t.take(4 * (m - 1)) =~= t.take(4 * m).take(4 * (m - 1)));
        }
        lemma_lane_prefix(s, t, j, m - 1);
        assert(s[4 * (m - 1) + j] == s.take(4 * m)[4 * (m - 1) + j]);
        assert(t[4 * (m - 1) + j] == t.take(4 * m)[4 * (m - 1) + j]);
    }
}

pub proof fn lemma_tri(t: int)
    requires
        0 <= t,
    ensures
        2 * tri(t) == t * (t + 1),
    decreases t,
{
    if t > 0 {
        lemma_tri(t - 1);
        let p = tri(t - 1);
        assert(tri(t) == p + t);
        assert(2 * (p + t) == t * (t + 1)) by (nonlinear_arith)
            requires
                2 * p == (t - 1) * ((t - 1) + 1),
        ;
    } else {
        assert(t * (t + 1) == 0) by (nonlinear_arith)
            requires
                t == 0,
        ;
    }
}

/// The overflow budget of one `CHUNK_SIZE` chunk: 5552 groups fit in `u32`.
pub proof fn lemma_b_vec_bound(t: int)
    requires
        0 <= t < 5552,
    ensures
        b_vec_bound(t) + (base() - 1) + 255 * (t + 1) == b_vec_bound(t + 1),
        b_vec_bound(t + 1) <= u32::MAX,
        0 <= b_vec_bound(t) <= b_vec_bound(t + 1),
{
    lemma_tri(t);
    lemma_tri(t + 1);
    assert(tri(t + 1) == tri(t) + (t + 1));
    assert(b_vec_bound(t) == 65520 * (t + 1) + 255 * tri(t));
    assert(b_vec_bound(t + 1) == 65520 * (t + 2) + 255 * tri(t + 1));
    assert((t + 1) * (t + 2) <= 5552 * 5553) by (nonlinear_arith)
        requires
            0 <= t < 5552,
    ;
    assert(0 <= t * (t + 1)) by (nonlinear_arith)
        requires
            0 <= t,
    ;
}

/// `x` is a multiple of `d` below `d`, so it is zero.
pub proof fn lemma_small_mod_zero(x: int, d: int)
    requires
        0 <= x < d,
        x % d == 0,
    ensures
        x == 0,
{
    vstd::arithmetic::div_mod::lemma_small_mod(x as nat, d as nat);
}

// ---------------------------------------------------------------- loop invariants of `compute`

/// Lane `j` of `(a_vec, b_vec)` after `g` groups of `s`, `t` groups since the last reduction:
/// congruent to the lane sums, and within the overflow budget.
pub open spec fn lane_ok(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, j: int, g: int, t: int) -> bool {
    &&& av[j] <= (base() - 1) + 255 * t
    &&& bv[j] <= b_vec_bound(t)
    &&& (av[j] as int) % base() == lane_sum(s, j, g) % base()
    &&& (bv[j] as int) % base() == lane_wsum(s, j, g) % base()
}

pub open spec fn lanes_partial(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, g: int, t: int) -> bool {
    &&& av.len() == 4
    &&& bv.len() == 4
    &&& 0 <= t <= 5552
    &&& 0 <= g
    &&& 4 * g <= s.len()
    &&& forall|j: int| 0 <= j < 4 ==> #[trigger] lane_ok(av, bv, s, j, g, t)
}

/// Lanes right after a `%= MOD`: reduced and congruent to the lane sums.
pub open spec fn lanes_reduced(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, g: int) -> bool {
    &&& lanes_partial(av, bv, s, g, 0)
    &&& forall|j: int| 0 <= j < 4 ==> #[trigger] av[j] < base() && bv[j] < base()
}

pub proof fn lemma_lanes_zero(s: Seq<u8>)
    ensures
        forall|j: int| 0 <= j < 4 ==> #[trigger] lane_sum(s, j, 0) == 0 && lane_wsum(s, j, 0) == 0,
{
}

/// One 4-byte group: the additions stay in `u32`, and the lanes advance to `g + 1`.
pub proof fn lemma_lanes_partial_step(
    av: Seq<u32>,
    bv: Seq<u32>,
    av2: Seq<u32>,
    bv2: Seq<u32>,
    s: Seq<u8>,
    g: int,
    t: int,
)
    requires
        lanes_partial(av, bv, s, g, t),
        t < 5552,
        4 * g + 4 <= s.len(),
        av2.len() == 4,
        bv2.len() == 4,
        forall|j: int| 0 <= j < 4 ==> #[trigger] av2[j] == av[j] + s[4 * g + j],
        forall|j: int| 0 <= j < 4 ==> #[trigger] bv2[j] == bv[j] + av2[j],
    ensures
        lanes_partial(av2, bv2, s, g + 1, t + 1),
{
    lemma_b_vec_bound(t);
    lemma_lane_step(s, g);
    assert forall|j: int| 0 <= j < 4 implies #[trigger] lane_ok(av2, bv2, s, j, g + 1, t + 1) by {
        assert(lane_ok(av, bv, s, j, g, t));
        let x = s[4 * g + j] as int;
        lemma_add_mod_noop(av[j] as int, x, base());
        lemma_add_mod_noop(lane_sum(s, j, g), x, base());
        lemma_add_mod_noop(bv[j] as int, av2[j] as int, base());
        lemma_add_mod_noop(lane_wsum(s, j, g), lane_sum(s, j, g + 1), base());
    }
}

/// The bounds that make one group's `a_vec += val; b_vec += a_vec` overflow-free.
pub proof fn lemma_lanes_partial_bounds(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, g: int, t: int)
    requires
        lanes_partial(av, bv, s, g, t),
        t < 5552,
        4 * g + 4 <= s.len(),
    ensures
        group_fits(av, bv, s, g, t, 0),
        group_fits(av, bv, s, g, t, 1),
        group_fits(av, bv, s, g, t, 2),
        group_fits(av, bv, s, g, t, 3),
{
    lemma_b_vec_bound(t);
    assert(lane_ok(av, bv, s, 0, g, t));
    assert(lane_ok(av, bv, s, 1, g, t));
    assert(lane_ok(av, bv, s, 2, g, t));
    assert(lane_ok(av, bv, s, 3, g, t));
}

/// Lane `j` of group `g` can be added without overflowing `u32`.
pub open spec fn group_fits(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, g: int, t: int, j: int) -> bool {
    &&& av[j] + s[4 * g + j] <= u32::MAX
    &&& bv[j] + av[j] + s[4 * g + j] <= u32::MAX
}

pub proof fn lemma_lanes_to_partial(av: Seq<u32>, bv: Seq<u32>, s: Seq<u8>, g: int)
    requires
        lanes_reduced(av, bv, s, g),
    ensures
        lanes_partial(av, bv, s, g, 0),
{
}

/// `a_vec %= MOD; b_vec %= MOD` re-establishes `lanes_reduced`.
pub proof fn lemma_lanes_reduce(
    av: Seq<u32>,
    bv: Seq<u32>,
    av2: Seq<u32>,
    bv2: Seq<u32>,
    s: Seq<u8>,
    g: int,
    t: int,
)
    requires
        lanes_partial(av, bv, s, g, t),
        av2.len() == 4,
        bv2.len() == 4,
        forall|j: int| 0 <= j < 4 ==> #[trigger] av2[j] == (av[j] as int) % base(),
        forall|j: int| 0 <= j < 4 ==> #[trigger] bv2[j] == (bv[j] as int) % base(),
    ensures
        lanes_reduced(av2, bv2, s, g),
{
    assert(b_vec_bound(0) == base() - 1);
    assert forall|j: int| 0 <= j < 4 implies #[trigger] lane_ok(av2, bv2, s, j, g, 0) && av2[j] < base()
        && bv2[j] < base() by {
        assert(lane_ok(av, bv, s, j, g, t));
        lemma_mod_twice(av[j] as int, base());
        lemma_mod_twice(bv[j] as int, base());
    }
}

/// `b += k * a; b %= MOD` after `k` more bytes keeps `b ≡ b0 + (bytes so far) * a0`.
pub proof fn lemma_b_chunk(b: int, b2: int, b0: int, a0: int, g_start: int, g: int, k: int)
    requires
        b % base() == (b0 + 4 * g_start * a0) % base(),
        b2 == (b + k * a0) % base(),
        4 * g == 4 * g_start + k,
    ensures
        b2 % base() == (b0 + 4 * g * a0) % base(),
{
    lemma_add_mod_noop(b, k * a0, base());
    lemma_add_mod_noop(b0 + 4 * g_start * a0, k * a0, base());
    lemma_mod_twice(b + k * a0, base());
    assert(4 * g * a0 == 4 * g_start * a0 + k * a0) by (nonlinear_arith)
        requires
            4 * g == 4 * g_start + k,
    ;
}

/// `v[0] + ... + v[k-1]`.
pub open spec fn prefix_sum(v: Seq<u32>, k: int) -> int
    decreases k,
{
    if k <= 0 {
        0
    } else {
        prefix_sum(v, k - 1) + v[k - 1]
    }
}

pub proof fn lemma_prefix_sum_bound(v: Seq<u32>, k: int, c: int)
    requires
        0 <= k <= 4,
        v.len() == 4,
        forall|j: int| 0 <= j < 4 ==> #[trigger] v[j] < c,
    ensures
        0 <= prefix_sum(v, k) <= 4 * c,
{
    reveal_with_fuel(prefix_sum, 5);
    assert(v[0] < c && v[1] < c && v[2] < c && v[3] < c);
}

/// The final congruences of `compute`, combining the lane sums, the vector combine step and the
/// serial tail.
pub proof fn lemma_combine(
    data: Seq<u8>,
    al: Seq<u8>,
    rem: Seq<u8>,
    m: int,
    a0: int,
    b0: int,
    b_red: int,
    av0: Seq<u32>,
    bv0: Seq<u32>,
    bv1: Seq<u32>,
    aa: int,
    bb: int,
)
    requires
        data == al + rem,
        al.len() == 4 * m,
        0 <= m,
        0 <= a0,
        0 <= b0,
        lanes_reduced(av0, bv0, al, m),
        b_red % base() == (b0 + 4 * m * a0) % base(),
        bv1.len() == 4,
        bv1[0] == 4 * bv0[0],
        bv1[1] == 4 * bv0[1] + (base() - av0[1]),
        bv1[2] == 4 * bv0[2] + (base() - av0[2]) * 2,
        bv1[3] == 4 * bv0[3] + (base() - av0[3]) * 3,
        aa == a0 + prefix_sum(av0, 4) + sum(rem),
        bb == b_red + prefix_sum(bv1, 4) + rem.len() * (a0 + prefix_sum(av0, 4)) + wsum(rem),
    ensures
        aa % base() == (a0 + sum(data)) % base(),
        bb % base() == (b0 + data.len() * a0 + wsum(data)) % base(),
{
    let r = rem.len() as int;
    reveal_with_fuel(prefix_sum, 5);
    lemma_lanes(al, m);
    assert(al.take(4 * m) =~= al);
    lemma_sum_append(al, rem);
    let ls0 = lane_sum(al, 0, m);
    let ls1 = lane_sum(al, 1, m);
    let ls2 = lane_sum(al, 2, m);
    let ls3 = lane_sum(al, 3, m);
    let lw0 = lane_wsum(al, 0, m);
    let lw1 = lane_wsum(al, 1, m);
    let lw2 = lane_wsum(al, 2, m);
    let lw3 = lane_wsum(al, 3, m);
    assert(lane_ok(av0, bv0, al, 0, m, 0) && av0[0] < base() && bv0[0] < base());
    assert(lane_ok(av0, bv0, al, 1, m, 0) && av0[1] < base() && bv0[1] < base());
    assert(lane_ok(av0, bv0, al, 2, m, 0) && av0[2] < base() && bv0[2] < base());
    assert(lane_ok(av0, bv0, al, 3, m, 0) && av0[3] < base() && bv0[3] < base());
    // each reduced lane is its lane sum minus a multiple of 65521
    let qa0 = lemma_mod_rep(av0[0] as int, ls0);
    let qa1 = lemma_mod_rep(av0[1] as int, ls1);
    let qa2 = lemma_mod_rep(av0[2] as int, ls2);
    let qa3 = lemma_mod_rep(av0[3] as int, ls3);
    let qb0 = lemma_mod_rep(bv0[0] as int, lw0);
    let qb1 = lemma_mod_rep(bv0[1] as int, lw1);
    let qb2 = lemma_mod_rep(bv0[2] as int, lw2);
    let qb3 = lemma_mod_rep(bv0[3] as int, lw3);
    // b_red is (b0 + 4m*a0) minus a multiple of 65521
    let qr = lemma_mod_rep(b_red, b0 + 4 * m * a0);
    let qa = qa0 + qa1 + qa2 + qa3;
    let A = a0 + prefix_sum(av0, 4);
    assert(A == a0 + lanes_sum(al, m) - 65521 * qa);
    let k1 = 6 + qa1 + 2 * qa2 + 3 * qa3 - 4 * (qb0 + qb1 + qb2 + qb3) - qr;
    assert(b_red + prefix_sum(bv1, 4) == b0 + 4 * m * a0 + lanes_wsum(al, m) + 65521 * k1);
    assert(r * A == r * (a0 + lanes_sum(al, m)) - 65521 * (r * qa)) by (nonlinear_arith)
        requires
            A == a0 + lanes_sum(al, m) - 65521 * qa,
    ;
    assert(data.len() * a0 == 4 * m * a0 + r * a0) by (nonlinear_arith)
        requires
            data.len() == 4 * m + r,
    ;
    assert(r * (a0 + lanes_sum(al, m)) == r * a0 + r * sum(al)) by (nonlinear_arith)
        requires
            lanes_sum(al, m) == sum(al),
    ;
    // aa
    assert(aa == 65521 * (-qa) + (a0 + sum(data)));
    lemma_mod_multiples_vanish(-qa, a0 + sum(data), 65521);
    // bb
    assert(bb == 65521 * (k1 - r * qa) + (b0 + data.len() * a0 + wsum(data)));
    lemma_mod_multiples_vanish(k1 - r * qa, b0 + data.len() * a0 + wsum(data), 65521);
}

/// `x` congruent to `y` means `y - x` is a multiple of `base()`; returns the multiplier.
pub proof fn lemma_mod_rep(x: int, y: int) -> (q: int)
    requires
        x % base() == y % base(),
    ensures
        y - x == 65521 * q,
{
    lemma_fundamental_div_mod(y - x, 65521);
    lemma_sub_mod_noop(y, x, 65521);
    (y - x) / 65521
}

pub proof fn lemma_wsum_bound(s: Seq<u8>)
    ensures
        0 <= wsum(s) <= 255 * s.len() * s.len(),
    decreases s.len(),
{
    if s.len() > 0 {
        lemma_wsum_bound(s.drop_last());
        lemma_sum_bounds(s);
        let n = s.len() as int;
        assert(255 * (n - 1) * (n - 1) + 255 * n <= 255 * n * n) by (nonlinear_arith)
            requires
                n >= 1,
        ;
    }
}

// ---------------------------------------------------------------- spec sanity

/// The definition reproduces known checksums (evaluated by Verus's interpreter).
proof fn adler32_test_vectors() {
    assert(adler32_spec(seq![]) == 1) by (compute);
    assert(adler32_spec(seq![0u8]) == 0x00010001) by (compute);
    assert(adler32_spec(seq![1u8]) == 0x00020002) by (compute);
    // "Wikipedia"
    assert(adler32_spec(seq![0x57u8, 0x69, 0x6b, 0x69, 0x70, 0x65, 0x64, 0x69, 0x61])
        == 0x11E60398) by (compute);
}

} // verus!
