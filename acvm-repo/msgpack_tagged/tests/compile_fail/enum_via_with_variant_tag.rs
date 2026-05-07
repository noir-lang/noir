//! `#[tagged(via(...))]` delegates the wire shape entirely to the wire DTO;
//! variant-level `#[tag(...)]` annotations on the public type are
//! wire-irrelevant and rejected so users aren't misled into thinking they
//! affect the wire format.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct WireDto {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireDto))]
enum Public {
    #[tag(0)]
    First,
}

fn main() {}
