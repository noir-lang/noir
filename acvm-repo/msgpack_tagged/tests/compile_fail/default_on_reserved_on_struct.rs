//! `default_on_reserved` is a sum-only decode-policy flag — there's no
//! discriminator to substitute for on a product. Applied to a struct, it's
//! rejected loudly rather than silently ignored.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, Default)]
#[tagged(default_on_reserved)]
struct NotAnEnum {
    #[tag(0)]
    payload: u32,
}

fn main() {}
