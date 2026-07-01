//! `#[tagged(allow_unknown_tags)]` flips struct-level decode tolerance for
//! unknown *field* tags (skip them silently). For enums, that policy has no
//! sound landing place — an unknown variant tag means the value's
//! discriminator itself is unknown, and there's no fragment to skip — so
//! the macro rejects it on enums rather than silently doing nothing.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(allow_unknown_tags)]
enum Choice {
    #[tag(0)]
    First,
}

fn main() {}
