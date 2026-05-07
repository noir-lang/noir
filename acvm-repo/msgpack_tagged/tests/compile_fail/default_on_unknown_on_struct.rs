//! `default_on_unknown` is a sum-only decode-policy flag — for products,
//! the equivalent "be lenient on unknown tags" knob is `allow_unknown_tags`,
//! whose semantics are to skip the unknown field rather than substitute a
//! default value. Applied to a struct, it's rejected loudly.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, Default)]
#[tagged(default_on_unknown)]
struct NotAnEnum {
    #[tag(0)]
    payload: u32,
}

fn main() {}
