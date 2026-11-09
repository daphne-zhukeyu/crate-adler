#!/usr/bin/env bash
# Builds a renamed copy of ../upstream (cargo cannot hold two packages named adler2 2.0.1 in one
# lockfile), checks the copy's sources are byte-identical to ../upstream/src, then runs the
# benchmark. Output goes to stdout (the evidence collector logs it to ../evidence/bench.log).
set -eu
cd "$(dirname "$0")"
rm -rf up && mkdir -p up && cp -r ../upstream/src ../upstream/Cargo.toml up/
sed -i 's/^name = "adler2"$/name = "adler2_up"/' up/Cargo.toml
sed -i '/^\[\[bench\]\]/,/^harness/d' up/Cargo.toml
echo "# sources of up/ vs ../upstream/src:"; diff -r up/src ../upstream/src && echo "identical"
cargo run --release -q
