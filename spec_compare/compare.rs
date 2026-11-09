//! Two-way refinement check between the Claude spec and the hand-written gold spec.
#![allow(unused_imports, dead_code)]
use vstd::prelude::*;

#[path = "../adler2/src/spec.rs"]
pub mod claude_spec;
pub mod human_written_spec;
pub mod links;
