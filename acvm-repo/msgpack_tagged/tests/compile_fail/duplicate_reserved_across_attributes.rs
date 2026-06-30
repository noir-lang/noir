//! Multiple `#[tagged(reserved(...))]` attributes accumulate, but
//! duplicate numbers across them are rejected — the dedup is on the
//! combined list, not just within a single attribute.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(1, 2))]
#[tagged(reserved(2, 3))]
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
