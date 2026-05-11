//! `#[serde(other)]` must be on a unit variant — serde-derive only routes
//! unknown identifiers to unit variants, and our wrapper inherits the
//! constraint when emitting `catch_all_tag`. A struct- or tuple-shaped
//! catch-all is rejected with a clear error from our macro before
//! serde-derive can fire its own.

use msgpack_tagged::MsgpackTagged;

#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged)]
#[tagged(allow_unknown)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(9)]
    #[serde(other)]
    NotUnit(u32),
}

fn main() {}
