//! `#[tag(skip)]` was removed — the macro now redirects users to
//! `#[serde(skip)]`, which is auto-recognized as the canonical skip
//! signal. Pins the redirect error message.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(0)]
    visible: u32,
    #[tag(skip)]
    hidden: u32,
}

fn main() {}
