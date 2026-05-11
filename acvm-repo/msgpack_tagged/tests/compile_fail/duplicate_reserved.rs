//! Each tag in `#[tagged(reserved(...))]` must appear at most once. Duplicates are a
//! compile error pointing at the second occurrence.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2, 5, 2))] // tag 2 listed twice
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
