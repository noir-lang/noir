//! `#[tagged(via(...))]` delegates the whole wire shape — including any
//! where-clause customization — to the wire DTO. `extra_bound` on a
//! `via`-delegating type is rejected; the bound belongs on the wire DTO
//! where the actual register_into and field tags live.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct WireDto {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireDto), extra_bound = "u32: msgpack_tagged::MsgpackTagged")]
struct Public;

fn main() {}
