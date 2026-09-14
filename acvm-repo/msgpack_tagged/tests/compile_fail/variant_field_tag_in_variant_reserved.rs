//! A variant-level `#[tagged(reserved(...))]` retires payload field tags
//! within that variant. Reusing one of those tags on a payload field is a
//! compile error — same compile-time guard as the type-level reserved list
//! gives top-level structs.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    #[tagged(reserved(2))]
    Multi {
        #[tag(0)]
        a: u32,
        #[tag(2)]
        b: bool,
    },
}

fn main() {}
