//! Throughput of upstream adler2 vs the annotated crate on the same 64 MiB buffer, in the same
//! process, with equal results asserted. Run: `cargo run --release` (log: ../evidence/bench.log).
use std::time::Instant;

fn main() {
    let data: Vec<u8> = (0..64u32 << 20).map(|i| (i.wrapping_mul(2654435761) >> 13) as u8).collect();
    assert_eq!(up::adler32_slice(&data), ver::adler32_slice(&data));
    let gb = 10.0 * data.len() as f64 / 1e9;
    for round in 0..3 {
        let t = Instant::now();
        let mut x = 0u32;
        for _ in 0..10 { x ^= up::adler32_slice(std::hint::black_box(&data)); }
        let tu = t.elapsed();
        let t = Instant::now();
        let mut y = 0u32;
        for _ in 0..10 { y ^= ver::adler32_slice(std::hint::black_box(&data)); }
        let tv = t.elapsed();
        assert_eq!(x, y);
        println!("round {round}: upstream {:.2} GB/s, annotated {:.2} GB/s (results equal)",
                 gb / tu.as_secs_f64(), gb / tv.as_secs_f64());
    }
}
