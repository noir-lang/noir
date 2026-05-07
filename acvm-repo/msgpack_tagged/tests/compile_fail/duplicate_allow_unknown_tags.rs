//! Each type may carry at most one `#[allow_unknown_tags]` attribute.
//! Duplicates are a compile error pointing at the second occurrence.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[allow_unknown_tags]
#[allow_unknown_tags] // duplicate
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
