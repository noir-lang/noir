//! `#[tag(default)]` was never a valid tag form. After the removal of the
//! `#[tag(N, default)]` modifier, only an integer tag literal or the
//! `skip` keyword is accepted inside `#[tag(...)]` — this fixture pins
//! the rejection of the bare `default` ident as a tag.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(default)]
    a: u32,
}

fn main() {}
