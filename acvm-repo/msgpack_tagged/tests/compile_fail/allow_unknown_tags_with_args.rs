//! `#[allow_unknown_tags]` is a presence-only flag; it doesn't take any
//! arguments. `#[allow_unknown_tags(...)]` is a parse error.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[allow_unknown_tags(true)] // attribute takes no arguments
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
