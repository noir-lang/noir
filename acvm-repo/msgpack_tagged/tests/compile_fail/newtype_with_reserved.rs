//! Newtype structs have no wire shape of their own — `#[tagged(reserved(...))]`
//! and `#[tagged(allow_unknown_tags)]` belong on types that carry their own
//! tag table.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2, 5))]
struct Witness(u32);

fn main() {}
