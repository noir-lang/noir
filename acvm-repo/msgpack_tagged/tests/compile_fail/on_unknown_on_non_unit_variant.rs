//! Mirror of `on_reserved_on_non_unit_variant` for the `on_unknown` marker.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    Real,
    #[tag(9)]
    #[tagged(on_unknown)]
    NotUnit { x: u32 },
}

fn main() {}
