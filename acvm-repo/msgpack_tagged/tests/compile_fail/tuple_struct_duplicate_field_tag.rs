//! Same uniqueness rule as named-struct field tags applies to all-explicit
//! tuple-struct field tags — duplicates rejected at compile time.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Triple(#[tag(0)] u32, #[tag(0)] bool, #[tag(2)] u8);

fn main() {}
