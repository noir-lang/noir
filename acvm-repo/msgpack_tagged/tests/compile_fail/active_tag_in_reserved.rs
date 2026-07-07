//! `#[tagged(reserved(...))]` lists tags retired from the type. An active `#[tag(N)]`
//! on a field whose `N` is in that list is a compile error — the macro
//! refuses to let a retired tag be reused.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2, 5))]
struct Foo {
    #[tag(0)]
    a: u32,
    #[tag(2)] // collides with the reserved list
    b: u32,
}

fn main() {}
