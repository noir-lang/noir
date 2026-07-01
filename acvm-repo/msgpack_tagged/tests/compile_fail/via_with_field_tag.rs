//! Field-level `#[tag(...)]` is not allowed on a type with `#[tagged(via(...))]`
//! — the public type's fields are wire-irrelevant, and the macro rejects
//! the annotation rather than silently ignoring it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Wire {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(Wire))]
struct Public {
    #[tag(0)] // not allowed: Public's fields don't reach the wire
    leftover: u32,
}

fn main() {}
