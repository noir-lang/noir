//! `#[tag(skip)]` is meaningful for struct fields (omit the field from the
//! wire) but has no obvious semantics for an enum variant — variants are the
//! discriminator, not optional payload — and is rejected.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(skip)]
    Second,
}

fn main() {}
