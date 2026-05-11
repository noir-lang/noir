//! Anything past the integer tag in `#[tag(N, ...)]` is rejected — the
//! grammar accepts a single integer tag literal and nothing else. Pins
//! the rejection.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0, foo)]
    a: u32,
}

fn main() {}
