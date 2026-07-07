//! Two variants cannot share the same wire tag — that would make the format
//! ambiguous on decode.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(0)]
    Second,
}

fn main() {}
