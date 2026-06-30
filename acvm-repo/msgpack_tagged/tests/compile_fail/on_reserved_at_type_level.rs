//! `on_reserved` is a variant-level marker, not a type-level modifier —
//! you put it on the variant that should catch retired tags, not on the
//! enum itself. The type-level grammar rejects it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(on_reserved)]
enum Choice {
    #[tag(0)]
    A,
}

fn main() {}
