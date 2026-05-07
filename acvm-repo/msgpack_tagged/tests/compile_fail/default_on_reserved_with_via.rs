//! `via(...)` delegates the entire wire shape — including any decode-policy
//! flags — to the wire DTO. Setting `default_on_reserved` on the public type
//! is wire-irrelevant and rejected so users put the flag on the wire DTO
//! where it actually applies.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, Default)]
enum WireDto {
    #[default]
    #[tag(0)]
    First,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireDto), default_on_reserved)]
struct Public;

fn main() {}
