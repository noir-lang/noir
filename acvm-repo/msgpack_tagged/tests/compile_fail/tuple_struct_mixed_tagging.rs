//! In a multi-element tuple struct, every field must either carry an explicit
//! `#[tag(N)]` or none must — mixing the two styles is rejected so the wire
//! layout is unambiguous.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Mixed(#[tag(0)] u32, bool, #[tag(2)] u8);

fn main() {}
