//! `#[tag(...)]` requires an integer tag literal — passing a bare ident
//! (here `default`, but any ident triggers the same rejection) errors at
//! parse time. Pins the rejection.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(default)]
    a: u32,
}

fn main() {}
