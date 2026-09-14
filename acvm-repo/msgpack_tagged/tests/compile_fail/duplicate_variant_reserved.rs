//! Same per-modifier-once rule as the type-level parser: each named
//! modifier inside `#[tagged(...)]` may appear at most once across all
//! `#[tagged(...)]` attributes on the same variant.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    #[tagged(reserved(2))]
    #[tagged(reserved(3))]
    First,
}

fn main() {}
