//! `allow_unknown_tags` inside `#[tagged(...)]` is a presence-only modifier.
//! Attaching a value (`#[tagged(allow_unknown_tags = true)]`) doesn't match
//! the recognized `Meta::Path` form and is rejected with an "expected
//! `reserved(...)` or `allow_unknown_tags`" error.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(allow_unknown_tags = true)] // attribute takes no arguments
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
