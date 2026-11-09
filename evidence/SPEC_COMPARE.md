# Spec comparison (two-way refinement)

Log: `spec_compare.log` (exit 1). Each cell is a Verus proof obligation in `spec_compare/links.rs`.

Verus: 62 verified, 2 errors over the whole comparison crate (both specs, their vectors, and the links).

| function | Claude ⇒ human | human ⇒ Claude |
|---|---|---|
| `adler32_slice` | verified | verified |
| `checksum` | verified | verified |
| `default` | verified | verified |
| `finish` | verified | verified |
| `from_checksum` | verified | verified |
| `new` | verified | verified |
| `write_slice` | FAILED | FAILED |
| `write_slice_canonical` | verified | verified |
| `write_slice_nonempty` | verified | verified |

Claude ⇒ human: **8 of 9** verified (1 failed, 0 TODO)

human ⇒ Claude: **8 of 9** verified (1 failed, 0 TODO)
