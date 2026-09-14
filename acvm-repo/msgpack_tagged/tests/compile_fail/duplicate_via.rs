//! At most one `via(...)` modifier per type. Duplicates are a compile error
//! pointing at the second occurrence.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct WireA {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
struct WireB {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireA), via(WireB))] // duplicate `via(...)`
struct Public {}

fn main() {}
