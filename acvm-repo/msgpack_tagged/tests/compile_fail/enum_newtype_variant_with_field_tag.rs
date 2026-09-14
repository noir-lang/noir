//! Newtype enum variants (single-element tuple variants) pass through to the
//! inner type — `#[tag(N)]` on the inner field would imply a field-level tag
//! the wire shape can't express, so it's rejected.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum E {
    #[tag(0)]
    Wrap(#[tag(0)] u32),
}

fn main() {}
