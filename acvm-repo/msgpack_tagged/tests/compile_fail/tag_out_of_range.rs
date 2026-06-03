//! Tag values must fit in `u8` (the design caps tag space at 256, well above
//! any realistic field count). `#[tag(300)]` should fail at parse time via
//! `LitInt::base10_parse::<u8>()` with a span on the literal.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(300)]
    a: u32,
}

fn main() {}
