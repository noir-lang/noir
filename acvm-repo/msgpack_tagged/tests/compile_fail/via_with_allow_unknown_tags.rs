//! `via(...)` and `allow_unknown_tags` are mutually exclusive — the lenient
//! decode flag is a wire-format property and belongs on the wire DTO, not
//! the public type that delegates to it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Wire {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(Wire), allow_unknown_tags)] // allow_unknown_tags doesn't belong here
struct Public {}

fn main() {}
