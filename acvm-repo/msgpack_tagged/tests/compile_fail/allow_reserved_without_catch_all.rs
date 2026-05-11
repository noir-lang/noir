//! Same rule as `allow_unknown_without_catch_all`, but for `allow_reserved`:
//! the flag requires a `#[serde(other)]` catch-all variant on the enum so
//! the wrapper has a place to route retired tags to on decode.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2), allow_reserved)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(1)]
    Second,
}

fn main() {}
