//! Every named field of a `MsgpackTagged` struct needs an explicit `#[tag(N)]`.
//! A missing tag is a derive-time compile error.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0)]
    a: u32,
    b: bool, // missing #[tag(N)] — error
}

fn main() {}
