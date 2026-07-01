//! Each modifier inside a variant-level `#[tagged(...)]` may appear at
//! most once — `on_reserved` is no exception.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2))]
enum Choice {
    #[tag(0)]
    Real,
    #[tag(9)]
    #[tagged(on_reserved, on_reserved)]
    Retired,
}

fn main() {}
