//! `via(...)` and `reserved(...)` are mutually exclusive — `reserved` is a
//! wire-format property and belongs on the wire DTO, not the public type
//! that delegates to it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Wire {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(Wire), reserved(2, 5))] // reserved doesn't belong here
struct Public {}

fn main() {}
