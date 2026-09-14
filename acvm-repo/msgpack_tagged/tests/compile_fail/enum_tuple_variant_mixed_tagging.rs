//! Tuple-variant payload fields follow the same all-or-nothing rule as
//! top-level multi-element tuple structs: either every payload field
//! carries `#[tag(N)]` (explicit, allows reordering / `default`) or none
//! do (implicit positional 0, 1, …). Mixing the two styles inside a
//! variant payload is rejected.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    Mixed(#[tag(0)] u32, bool),
}

fn main() {}
