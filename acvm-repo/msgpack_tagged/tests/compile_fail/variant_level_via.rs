//! `via(WireType)` is a type-level delegation — the *whole* type's wire
//! shape is the wire DTO's. There's no variant-level analog (variants
//! aren't independently delegating types) so the macro rejects it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct WireDto {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    #[tagged(via(WireDto))]
    First,
}

fn main() {}
