//! Anything past the integer tag in `#[tag(N, ...)]` is rejected — the
//! grammar accepts only a single integer tag or the bare keyword `skip`.
//! Pins the rejection with a span on the offending modifier.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0, foo)]
    a: u32,
}

fn main() {}
