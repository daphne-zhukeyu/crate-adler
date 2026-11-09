# Evidence index for adler2 (2026-11-04T22:52:44Z)

| log | command | exit | key line |
|---|---|---|---|
| `verus_std.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --time src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_nostd.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --time src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_nocheating.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --no-cheating --cfg feature="std" src/lib.rs` | 1 | error: external_body/assume_specification not allowed with --no-cheating   --> src/trusted.rs:24:1 error: external_body/assume_specification not allowed with -- |
| `verus_seed1.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=1 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_seed2.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=2 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_seed3.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=3 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_seed4.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=4 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_seed5.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=5 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `verus_seed6.log` | `/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus --crate-type=lib --edition 2021 --cfg feature="std" --smt-option smt.random_seed=6 src/lib.rs` | 0 | verification results:: 68 verified, 0 errors  |
| `cargo_build.log` | `cargo build` | 0 |     Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.24s  |
| `cargo_test.log` | `cargo test` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filt |
| `cargo_test_nostd.log` | `cargo test --no-default-features` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filt |
| `cargo_test_release.log` | `cargo test --release` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filt |
| `negative_controls.log` | `env VERUS=/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux/verus python3 -I negative_controls.py` | 0 | rejected 14, problems 0  |
| `cargo_verus_nostd.log` | `env PATH=/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux:/home/zhukeyu/.local/bin:/home/zhukeyu/.vscode-server/cli/servers/Stable-6d62b1d5dace93a33b3a6d3eca1317bd5e15bcb1/server/bin/remote-cli:/home/zhukeyu/.local/bin:/home/zhukeyu/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin cargo verus verify --no-default-features` | 0 | verification results:: 1876 verified, 0 errors verification results:: 68 verified, 0 errors  |
| `cargo_verus_std.log` | `env PATH=/home/zhukeyu/verus-0.2026.10.04/verus-x86-linux:/home/zhukeyu/.local/bin:/home/zhukeyu/.vscode-server/cli/servers/Stable-6d62b1d5dace93a33b3a6d3eca1317bd5e15bcb1/server/bin/remote-cli:/home/zhukeyu/.local/bin:/home/zhukeyu/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin cargo verus verify` | 101 | Error: could not find .vir file for 'adler2' error: could not compile `adler2` (lib)  |
| `bench.log` | `./run.sh` | 0 | identical round 0: upstream 16.15 GB/s, annotated 16.49 GB/s (results equal) round 1: upstream 16.47 GB/s, annotated 16.44 GB/s (results equal)  |

Other evidence files in this directory: FN_INVENTORY.md, TIMELINE.md, code_changes/ (CODE_CHANGES.md, SIGNOFF.md), verus_std.json, DOC_REVIEW.md, SPEC_COMPARE.md (when the gold spec exists).
