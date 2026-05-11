//! `allow_unknown` routes unknown variant tags to the type's `#[serde(other)]`
//! catch-all variant on decode. Without such a variant the macro has nothing
//! to route to, so the flag is rejected at compile time.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(allow_unknown)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(1)]
    Second,
}

fn main() {}
