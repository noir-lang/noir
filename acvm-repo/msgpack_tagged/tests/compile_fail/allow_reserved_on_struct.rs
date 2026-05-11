//! `allow_reserved` is a sum-only decode-policy flag — there's no
//! catch-all variant on a product to route to. Applied to a struct, it's
//! rejected loudly rather than silently ignored.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(allow_reserved)]
struct NotAnEnum {
    #[tag(0)]
    payload: u32,
}

fn main() {}
