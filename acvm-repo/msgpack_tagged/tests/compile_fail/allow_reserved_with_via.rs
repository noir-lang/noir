//! `via(...)` delegates the entire wire shape — including any decode-policy
//! flags — to the wire DTO. Setting `allow_reserved` on the public type is
//! wire-irrelevant and rejected so users put the flag on the wire DTO where
//! it actually applies.

use msgpack_tagged::MsgpackTagged;

#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged)]
enum WireDto {
    #[tag(0)]
    First,
    #[tag(9)]
    #[serde(other)]
    Unknown,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireDto), allow_reserved)]
struct Public;

fn main() {}
