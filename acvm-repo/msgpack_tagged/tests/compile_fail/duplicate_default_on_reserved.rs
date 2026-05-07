//! Each `#[tagged(...)]` modifier may appear at most once across all
//! `#[tagged(...)]` attributes — `default_on_reserved` is no exception.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, Default)]
#[tagged(default_on_reserved)]
#[tagged(default_on_reserved)]
enum Choice {
    #[default]
    #[tag(0)]
    First,
}

fn main() {}
