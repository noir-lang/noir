//! Each `#[tagged(...)]` modifier may appear at most once across all
//! `#[tagged(...)]` attributes — `allow_reserved` is no exception.

use msgpack_tagged::MsgpackTagged;

#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged)]
#[tagged(allow_reserved)]
#[tagged(allow_reserved)]
enum Choice {
    #[tag(0)]
    First,
    #[tag(9)]
    #[serde(other)]
    Unknown,
}

fn main() {}
