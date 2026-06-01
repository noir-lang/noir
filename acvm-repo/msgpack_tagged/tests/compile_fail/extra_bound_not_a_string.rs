//! `extra_bound` requires a string literal value. Passing anything else
//! (an integer, an ident, etc.) is rejected with a clear "requires a
//! string literal" message.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(extra_bound = 42)]
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
