//! Modifiers in `#[tag(N, ...)]` are extensible (the only one today is
//! `default`), but unknown idents in that position should error with a
//! span on the offending token.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0, foo)]
    a: u32,
}

fn main() {}
