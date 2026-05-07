//! Newtype structs (single-element tuple structs) pass through to the inner
//! type — `#[tag(N)]` on the inner field is meaningless and rejected.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Witness(#[tag(0)] u32);

fn main() {}
