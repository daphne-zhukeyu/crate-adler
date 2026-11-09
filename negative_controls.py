#!/usr/bin/env python3
"""Injects one bug at a time into a scratch copy of adler2/ and checks that Verus rejects it.

Usage: VERUS=/path/to/verus python3 -I negative_controls.py
"""
import os, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
VERUS = os.environ.get("VERUS", "verus")

# (id, file, old, new, occurrence (1-based) or 0 for the only one, description)
MUTANTS = [
    ("N0", "src/algo.rs", "let ghost data = bytes@;", "let ghost data = bytes@; proof { assert(false); }", 0,
     "sanity: `assert(false)` in compute (would pass only if the context were inconsistent)"),
    ("N1", "src/algo.rs", "const CHUNK_SIZE: usize = 5552 * 4;", "const CHUNK_SIZE: usize = 5553 * 4;", 0,
     "chunk one group too long (b_vec can overflow)"),
    ("N2", "src/algo.rs", "const MOD: u32 = 65521;", "const MOD: u32 = 65519;", 0,
     "wrong modulus"),
    ("N3", "src/algo.rs", "        b_vec.0[1] += MOD - a_vec.0[1];\n", "", 0,
     "combine step drops lane-1 correction"),
    ("N4", "src/algo.rs", "b_vec.0[3] += (MOD - a_vec.0[3]) * 3;", "b_vec.0[3] += (MOD - a_vec.0[3]) * 2;", 0,
     "combine step uses weight 2 for lane 3"),
    ("N5", "src/algo.rs", "b_vec *= 4;", "b_vec *= 3;", 0,
     "combine step scales by 3"),
    ("N6", "src/algo.rs", "            b += CHUNK_SIZE as u32 * a;\n", "", 0,
     "full chunks forget the b += n*a term"),
    ("N7", "src/algo.rs", "        a_vec %= MOD;\n", "", 2,
     "final reduction of a_vec dropped (MOD - a_vec may underflow)"),
    ("N8", "src/algo.rs", "            a += u32::from(byte);\n            b += a;",
     "            b += a;\n            a += u32::from(byte);", 0,
     "serial tail updates b before a"),
    ("N9", "src/algo.rs", "self.0[2] += other.0[2];", "self.0[2] += other.0[1];", 0,
     "U32X4 += mixes lanes"),
    ("N10", "src/lib.rs", "Adler32 { a: 1, b: 0 }", "Adler32 { a: 0, b: 1 }", 0,
     "wrong initial state"),
    ("N11", "src/lib.rs", "(u32::from(self.b) << 16) | u32::from(self.a)",
     "(u32::from(self.a) << 16) | u32::from(self.b)", 0,
     "checksum packs a and b the wrong way round"),
    ("N12", "src/lib.rs", "b: (sum >> 16) as u16,", "b: (sum >> 15) as u16,", 0,
     "from_checksum takes the wrong bits for b"),
    ("N13", "src/spec.rs", "pub open spec fn base() -> int {\n    65521", "pub open spec fn base() -> int {\n    65519", 0,
     "spec typo: wrong modulus (caught by the test vectors)"),
]


def mutate(text, old, new, occ):
    n = text.count(old)
    if n == 0 or (occ == 0 and n != 1) or occ > n:
        return None
    if occ == 0:
        return text.replace(old, new)
    idx = -1
    for _ in range(occ):
        idx = text.index(old, idx + 1)
    return text[:idx] + new + text[idx + len(old):]


def main():
    tmp = tempfile.mkdtemp()
    ok = bad = 0
    try:
        for mid, f, old, new, occ, desc in MUTANTS:
            work = os.path.join(tmp, mid)
            shutil.copytree(os.path.join(HERE, "adler2"), work, ignore=shutil.ignore_patterns("target"))
            path = os.path.join(work, f)
            text = open(path).read()
            m = mutate(text, old, new, occ)
            if m is None:
                print(f"{mid} MUTATION DID NOT APPLY: {desc}")
                bad += 1
                continue
            open(path, "w").write(m)
            r = subprocess.run([VERUS, "--crate-type=lib", "--edition", "2021", "--cfg", 'feature="std"',
                                os.path.join(work, "src/lib.rs")], capture_output=True, text=True)
            out = r.stdout + r.stderr
            if r.returncode == 0:
                print(f"{mid} ACCEPTED (bad): {desc}")
                bad += 1
            else:
                first = next((l for l in out.splitlines() if l.startswith("error")), "?")
                print(f"{mid} rejected: {desc}  [{first[:70]}]")
                ok += 1
    finally:
        shutil.rmtree(tmp)
    print(f"rejected {ok}, problems {bad}")
    sys.exit(1 if bad else 0)


main()
