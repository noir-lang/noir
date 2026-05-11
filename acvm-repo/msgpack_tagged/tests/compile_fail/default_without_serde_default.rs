//! `#[tag(N, default)]` on a named-struct field needs `#[serde(default)]`
//! (or `#[serde(default = "...")]`) for serde-derive's decoder to fill
//! `T::default()` when the tag is missing on the wire. Our macro
//! flags the missing pairing at compile time so the wire-tolerance
//! signal doesn't go silent.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0)]
    a: u32,
    #[tag(1, default)]
    b: Vec<u8>,
}

fn main() {}
