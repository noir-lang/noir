//! Each tagged field of a named struct (or named-variant payload) must have
//! a unique tag — duplicates would make the wire layout ambiguous on
//! decode. Caught at compile time, same as duplicate variant tags.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Named {
    #[tag(1)]
    a: u32,
    #[tag(1)]
    b: bool,
}

fn main() {}
