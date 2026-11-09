use crate::Adler32;
use std::ops::{AddAssign, MulAssign, RemAssign};
use vstd::prelude::*;
#[cfg(verus_keep_ghost)]
use crate::spec::*;
#[cfg(verus_keep_ghost)]
use crate::trusted::*;
#[cfg(verus_keep_ghost)]
use vstd::std_specs::ops::{AddAssignSpecImpl, MulAssignSpecImpl, RemAssignSpecImpl};

verus! {

impl Adler32 {
    pub(crate) fn compute(&mut self, bytes: &[u8])
        ensures
            final(self)@ == adler_from(old(self)@.0 % base(), old(self)@.1 % base(), bytes@),
    {
        // The basic algorithm is, for every byte:
        //   a = (a + byte) % MOD
        //   b = (b + a) % MOD
        // where MOD = 65521.
        //
        // For efficiency, we can defer the `% MOD` operations as long as neither a nor b overflows:
        // - Between calls to `write`, we ensure that a and b are always in range 0..MOD.
        // - We use 32-bit arithmetic in this function.
        // - Therefore, a and b must not increase by more than 2^32-MOD without performing a `% MOD`
        //   operation.
        //
        // According to Wikipedia, b is calculated as follows for non-incremental checksumming:
        //   b = n×D1 + (n−1)×D2 + (n−2)×D3 + ... + Dn + n*1 (mod 65521)
        // Where n is the number of bytes and Di is the i-th Byte. We need to change this to account
        // for the previous values of a and b, as well as treat every input Byte as being 255:
        //   b_inc = n×255 + (n-1)×255 + ... + 255 + n*65520
        // Or in other words:
        //   b_inc = n*65520 + n(n+1)/2*255
        // The max chunk size is thus the largest value of n so that b_inc <= 2^32-65521.
        //   2^32-65521 = n*65520 + n(n+1)/2*255
        // Plugging this into an equation solver since I can't math gives n = 5552.18..., so 5552.
        //
        // On top of the optimization outlined above, the algorithm can also be parallelized with a
        // bit more work:
        //
        // Note that b is a linear combination of a vector of input bytes (D1, ..., Dn).
        //
        // If we fix some value k<N and rewrite indices 1, ..., N as
        //
        //   1_1, 1_2, ..., 1_k, 2_1, ..., 2_k, ..., (N/k)_k,
        //
        // then we can express a and b in terms of sums of smaller sequences kb and ka:
        //
        //   ka(j) := D1_j + D2_j + ... + D(N/k)_j where j <= k
        //   kb(j) := (N/k)*D1_j + (N/k-1)*D2_j + ... + D(N/k)_j where j <= k
        //
        //  a = ka(1) + ka(2) + ... + ka(k) + 1
        //  b = k*(kb(1) + kb(2) + ... + kb(k)) - 1*ka(2) - ...  - (k-1)*ka(k) + N
        //
        // We use this insight to unroll the main loop and process k=4 bytes at a time.
        // The resulting code is highly amenable to SIMD acceleration, although the immediate speedups
        // stem from increased pipeline parallelism rather than auto-vectorization.
        //
        // This technique is described in-depth (here:)[https://software.intel.com/content/www/us/\
        // en/develop/articles/fast-computation-of-fletcher-checksums.html]

        const MOD: u32 = 65521;
        const CHUNK_SIZE: usize = 5552 * 4;

        // Verus: ghost names for the inputs.
        let ghost data = bytes@;
        let ghost a0 = old(self)@.0;
        let ghost b0 = old(self)@.1;

        let mut a = u32::from(self.a);
        let mut b = u32::from(self.b);
        let mut a_vec = U32X4([0; 4]);
        let mut b_vec = a_vec;

        let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);

        // Verus: `al` is the 4-aligned prefix; `g` counts the 4-byte groups folded into the lanes.
        let ghost al = bytes@;
        let ghost full = al.len() as int - (al.len() as int) % (CHUNK_SIZE as int);
        let ghost mut g: int = 0;
        proof {
            assert(data =~= al + remainder@);
            lemma_lanes_zero(al);
            assert(lanes_reduced(a_vec.0@, b_vec.0@, al, 0)) by {
                assert(b_vec_bound(0) == base() - 1);
                assert forall|j: int| 0 <= j < 4 implies #[trigger] lane_ok(a_vec.0@, b_vec.0@, al, j, 0, 0) by {
                    assert(lane_sum(al, j, 0) == 0 && lane_wsum(al, j, 0) == 0);
                }
            }
        }

        // iterate over 4 bytes at a time
        let mut chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
        let remainder_chunk = chunk_iter.remainder();
        proof {
            assert(0 <= full <= al.len()) by {
                vstd::arithmetic::div_mod::lemma_mod_bound(al.len() as int, CHUNK_SIZE as int);
            }
            assert(full % (CHUNK_SIZE as int) == 0) by {
                vstd::arithmetic::div_mod::lemma_fundamental_div_mod(al.len() as int, CHUNK_SIZE as int);
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(al.len() as int / (CHUNK_SIZE as int), CHUNK_SIZE as int);
            }
            assert(al.take(full) =~= al.subrange(0, full));
        }
        // Verus: `while let` is what `for chunk in chunk_iter` desugars to; see trusted.rs (T3).
        while let Some(chunk) = chunk_iter.next()
            invariant
                al == bytes@,
                al.len() % 4 == 0,
                full == al.len() as int - (al.len() as int) % (CHUNK_SIZE as int),
                remainder_chunk@ == al.skip(full),
                chunks_exact_size(chunk_iter) == CHUNK_SIZE,
                chunks_exact_rest(chunk_iter) == al.subrange(4 * g, full),
                0 <= 4 * g <= full,
                (full - 4 * g) % (CHUNK_SIZE as int) == 0,
                a == a0,
                0 <= a0 < 65536,
                0 <= b0 < 65536,
                b < 65536,
                (b as int) % base() == (b0 + 4 * g * a0) % base(),
                lanes_reduced(a_vec.0@, b_vec.0@, al, g),
            ensures
                chunks_exact_rest(chunk_iter).len() < CHUNK_SIZE,
            decreases chunks_exact_rest(chunk_iter).len(),
        {
            let ghost g_start = g;
            proof {
                assert(chunk@ =~= al.subrange(4 * g, 4 * g + CHUNK_SIZE));
                lemma_lanes_to_partial(a_vec.0@, b_vec.0@, al, g);
            }
            let mut byte_vec_iter = chunk.chunks_exact(4);
            proof {
                assert(chunk@.take(CHUNK_SIZE as int) =~= al.subrange(4 * g, 4 * g_start + CHUNK_SIZE));
            }
            // Verus: `while let` is what `for byte_vec in chunk.chunks_exact(4)` desugars to.
            while let Some(byte_vec) = byte_vec_iter.next()
                invariant
                    al == bytes@,
                    4 * g_start + CHUNK_SIZE <= al.len(),
                    chunks_exact_size(byte_vec_iter) == 4,
                    chunks_exact_rest(byte_vec_iter) == al.subrange(4 * g, 4 * g_start + CHUNK_SIZE),
                    g_start <= g <= g_start + 5552,
                    lanes_partial(a_vec.0@, b_vec.0@, al, g, g - g_start),
                ensures
                    chunks_exact_rest(byte_vec_iter).len() < 4,
                decreases chunks_exact_rest(byte_vec_iter).len(),
            {
                let ghost av = a_vec.0@;
                let ghost bv = b_vec.0@;
                proof {
                    assert(byte_vec@ =~= al.subrange(4 * g, 4 * g + 4));
                    assert(byte_vec@[0] == al[4 * g] && byte_vec@[1] == al[4 * g + 1]);
                    assert(byte_vec@[2] == al[4 * g + 2] && byte_vec@[3] == al[4 * g + 3]);
                    lemma_lanes_partial_bounds(av, bv, al, g, g - g_start);
                }
                let val = U32X4::from(byte_vec);
                a_vec += val;
                b_vec += a_vec;
                proof {
                    lemma_lanes_partial_step(av, bv, a_vec.0@, b_vec.0@, al, g, g - g_start);
                    assert(al.subrange(4 * g, 4 * g_start + CHUNK_SIZE).skip(4)
                        =~= al.subrange(4 * (g + 1), 4 * g_start + CHUNK_SIZE));
                    g = g + 1;
                }
            }
            proof {
                assert(g == g_start + 5552);
                assert(CHUNK_SIZE as int * a0 <= 5552 * 4 * 65535) by (nonlinear_arith)
                    requires 0 <= a0 < 65536;
            }
            let ghost b_pre = b as int;
            let ghost av = a_vec.0@;
            let ghost bv = b_vec.0@;

            b += CHUNK_SIZE as u32 * a;
            a_vec %= MOD;
            b_vec %= MOD;
            b %= MOD;
            proof {
                lemma_lanes_reduce(av, bv, a_vec.0@, b_vec.0@, al, g, g - g_start);
                lemma_b_chunk(b_pre, b as int, b0, a0, g_start, g, CHUNK_SIZE as int);
                assert(al.subrange(4 * g_start, full).skip(CHUNK_SIZE as int)
                    =~= al.subrange(4 * g, full));
                vstd::arithmetic::div_mod::lemma_mod_multiples_vanish(-1, full - 4 * g_start, CHUNK_SIZE as int);
            }
        }
        // special-case the final chunk because it may be shorter than the rest
        let ghost g_start = g;
        proof {
            lemma_small_mod_zero(full - 4 * g, CHUNK_SIZE as int);
            assert(4 * g == full);
            lemma_lanes_to_partial(a_vec.0@, b_vec.0@, al, g);
        }
        // Verus: `while let` is what `for byte_vec in remainder_chunk.chunks_exact(4)` desugars to.
        let mut byte_vec_iter = remainder_chunk.chunks_exact(4);
        proof {
            assert(remainder_chunk@.take(remainder_chunk@.len() as int) =~= al.subrange(4 * g, al.len() as int));
        }
        while let Some(byte_vec) = byte_vec_iter.next()
            invariant
                al == bytes@,
                al.len() - 4 * g_start < CHUNK_SIZE,
                chunks_exact_size(byte_vec_iter) == 4,
                chunks_exact_rest(byte_vec_iter) == al.subrange(4 * g, al.len() as int),
                g_start <= g,
                4 * g <= al.len(),
                (al.len() - 4 * g) % 4 == 0,
                lanes_partial(a_vec.0@, b_vec.0@, al, g, g - g_start),
            ensures
                chunks_exact_rest(byte_vec_iter).len() < 4,
            decreases chunks_exact_rest(byte_vec_iter).len(),
        {
            let ghost av = a_vec.0@;
            let ghost bv = b_vec.0@;
            proof {
                assert(byte_vec@ =~= al.subrange(4 * g, 4 * g + 4));
                assert(byte_vec@[0] == al[4 * g] && byte_vec@[1] == al[4 * g + 1]);
                assert(byte_vec@[2] == al[4 * g + 2] && byte_vec@[3] == al[4 * g + 3]);
                lemma_lanes_partial_bounds(av, bv, al, g, g - g_start);
            }
            let val = U32X4::from(byte_vec);
            a_vec += val;
            b_vec += a_vec;
            proof {
                lemma_lanes_partial_step(av, bv, a_vec.0@, b_vec.0@, al, g, g - g_start);
                assert(al.subrange(4 * g, al.len() as int).skip(4)
                    =~= al.subrange(4 * (g + 1), al.len() as int));
                g = g + 1;
            }
        }
        proof {
            assert(4 * g == al.len());
            assert(remainder_chunk@.len() * a0 <= 5552 * 4 * 65535) by (nonlinear_arith)
                requires 0 <= a0 < 65536, remainder_chunk@.len() < 5552 * 4;
        }
        let ghost b_pre = b as int;
        let ghost av = a_vec.0@;
        let ghost bv = b_vec.0@;
        b += remainder_chunk.len() as u32 * a;
        a_vec %= MOD;
        b_vec %= MOD;
        b %= MOD;
        let ghost m = g;
        let ghost b_red = b as int;
        let ghost av0 = a_vec.0@;
        let ghost bv0 = b_vec.0@;
        proof {
            lemma_lanes_reduce(av, bv, av0, bv0, al, g, g - g_start);
            lemma_b_chunk(b_pre, b_red, b0, a0, g_start, g, remainder_chunk@.len() as int);
            assert(av0[0] < base() && bv0[0] < base() && av0[1] < base() && bv0[1] < base());
            assert(av0[2] < base() && bv0[2] < base() && av0[3] < base() && bv0[3] < base());
        }

        // combine the sub-sum results into the main sum
        b_vec *= 4;
        b_vec.0[1] += MOD - a_vec.0[1];
        b_vec.0[2] += (MOD - a_vec.0[2]) * 2;
        b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
        let ghost bv1 = b_vec.0@;
        // Verus: `it` names the iterator for the invariant.
        for &av in it: a_vec.0.iter()
            invariant
                a_vec.0@ == av0,
                it.seq().unref() == av0,
                a == a0 + prefix_sum(av0, it.index() as int),
                forall|j: int| 0 <= j < 4 ==> #[trigger] av0[j] < base(),
                0 <= a0 < 65536,
        {
            proof {
                lemma_prefix_sum_bound(av0, it.index() as int + 1, base());
            }
            a += av;
        }
        for &bv in it: b_vec.0.iter()
            invariant
                b_vec.0@ == bv1,
                it.seq().unref() == bv1,
                b == b_red + prefix_sum(bv1, it.index() as int),
                0 <= b_red < base(),
                forall|j: int| 0 <= j < 4 ==> #[trigger] bv1[j] < 8 * base(),
        {
            proof {
                lemma_prefix_sum_bound(bv1, it.index() as int + 1, 8 * base());
            }
            b += bv;
        }
        let ghost aa = a as int;
        let ghost bb = b as int;
        proof {
            lemma_prefix_sum_bound(av0, 4, base());
            lemma_prefix_sum_bound(bv1, 4, 8 * base());
        }

        // iterate over the remaining few bytes in serial
        // Verus: `it` names the iterator for the invariant.
        for &byte in it: remainder.iter()
            invariant
                it.seq().unref() == remainder@,
                remainder@.len() < 4,
                0 <= aa <= 65535 + 4 * base(),
                0 <= bb <= base() + 32 * base(),
                a == aa + sum(remainder@.take(it.index() as int)),
                b == bb + it.index() * aa + wsum(remainder@.take(it.index() as int)),
        {
            let ghost k = it.index() as int;
            proof {
                let t = remainder@.take(k + 1);
                assert(t.drop_last() =~= remainder@.take(k));
                lemma_sum_bounds(remainder@.take(k));
                lemma_sum_bounds(t);
                assert(wsum(t) <= 4 * 765) by {
                    reveal_with_fuel(wsum, 5);
                    lemma_wsum_bound(t);
                }
                assert((k + 1) * aa == k * aa + aa) by (nonlinear_arith);
                assert((k + 1) * aa <= 4 * aa) by (nonlinear_arith) requires k < 3, aa >= 0;
            }
            a += u32::from(byte);
            b += a;
        }
        proof {
            assert(remainder@.take(remainder@.len() as int) =~= remainder@);
            lemma_combine(data, al, remainder@, m, a0, b0, b_red, av0, bv0, bv1, a as int, b as int);
            lemma_adler_from_unreduced(a0, b0, data);
        }

        self.a = (a % MOD) as u16;
        self.b = (b % MOD) as u16;
    }
}

#[cfg(verus_keep_ghost)] // spec-only: how `+=`, `%=` and `*=` behave
impl AddAssignSpecImpl<U32X4> for U32X4 {
    closed spec fn obeys_add_assign_spec() -> bool {
        true
    }

    closed spec fn add_assign_req(&self, rhs: U32X4) -> bool {
        &&& self.0[0] + rhs.0[0] <= u32::MAX
        &&& self.0[1] + rhs.0[1] <= u32::MAX
        &&& self.0[2] + rhs.0[2] <= u32::MAX
        &&& self.0[3] + rhs.0[3] <= u32::MAX
    }

    closed spec fn add_assign_spec(&self, rhs: U32X4) -> &U32X4 {
        &U32X4(
            [
                (self.0[0] + rhs.0[0]) as u32,
                (self.0[1] + rhs.0[1]) as u32,
                (self.0[2] + rhs.0[2]) as u32,
                (self.0[3] + rhs.0[3]) as u32,
            ],
        )
    }
}

#[cfg(verus_keep_ghost)] // spec-only: how `+=`, `%=` and `*=` behave
impl RemAssignSpecImpl<u32> for U32X4 {
    closed spec fn obeys_rem_assign_spec() -> bool {
        true
    }

    closed spec fn rem_assign_req(&self, rhs: u32) -> bool {
        rhs != 0
    }

    closed spec fn rem_assign_spec(&self, rhs: u32) -> &U32X4 {
        &U32X4(
            [
                (self.0[0] % rhs) as u32,
                (self.0[1] % rhs) as u32,
                (self.0[2] % rhs) as u32,
                (self.0[3] % rhs) as u32,
            ],
        )
    }
}

#[cfg(verus_keep_ghost)] // spec-only: how `+=`, `%=` and `*=` behave
impl MulAssignSpecImpl<u32> for U32X4 {
    closed spec fn obeys_mul_assign_spec() -> bool {
        true
    }

    closed spec fn mul_assign_req(&self, rhs: u32) -> bool {
        &&& self.0[0] * rhs <= u32::MAX
        &&& self.0[1] * rhs <= u32::MAX
        &&& self.0[2] * rhs <= u32::MAX
        &&& self.0[3] * rhs <= u32::MAX
    }

    closed spec fn mul_assign_spec(&self, rhs: u32) -> &U32X4 {
        &U32X4(
            [
                (self.0[0] * rhs) as u32,
                (self.0[1] * rhs) as u32,
                (self.0[2] * rhs) as u32,
                (self.0[3] * rhs) as u32,
            ],
        )
    }
}

#[derive(Copy, Clone)]
struct U32X4([u32; 4]);

impl U32X4 {
    #[inline]
    fn from(bytes: &[u8]) -> (r: Self)
        requires
            bytes@.len() >= 4,
        ensures
            r.0[0] == bytes@[0],
            r.0[1] == bytes@[1],
            r.0[2] == bytes@[2],
            r.0[3] == bytes@[3],
    {
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
        let ghost s0 = *self;
        // Implement this in a primitive manner to help out the compiler a bit.
        self.0[0] += other.0[0];
        self.0[1] += other.0[1];
        self.0[2] += other.0[2];
        self.0[3] += other.0[3];
        proof {
            assert(self.0 =~= [(s0.0[0] + other.0[0]) as u32, (s0.0[1] + other.0[1]) as u32, (s0.0[2] + other.0[2]) as u32, (s0.0[3] + other.0[3]) as u32]);
        }
    }
}

impl RemAssign<u32> for U32X4 {
    #[inline]
    fn rem_assign(&mut self, quotient: u32) {
        let ghost s0 = *self;
        self.0[0] %= quotient;
        self.0[1] %= quotient;
        self.0[2] %= quotient;
        self.0[3] %= quotient;
        proof {
            assert(self.0 =~= [(s0.0[0] % quotient) as u32, (s0.0[1] % quotient) as u32, (s0.0[2] % quotient) as u32, (s0.0[3] % quotient) as u32]);
        }
    }
}

impl MulAssign<u32> for U32X4 {
    #[inline]
    fn mul_assign(&mut self, rhs: u32) {
        let ghost s0 = *self;
        self.0[0] *= rhs;
        self.0[1] *= rhs;
        self.0[2] *= rhs;
        self.0[3] *= rhs;
        proof {
            assert(self.0 =~= [(s0.0[0] * rhs) as u32, (s0.0[1] * rhs) as u32, (s0.0[2] * rhs) as u32, (s0.0[3] * rhs) as u32]);
        }
    }
}

} // verus!
