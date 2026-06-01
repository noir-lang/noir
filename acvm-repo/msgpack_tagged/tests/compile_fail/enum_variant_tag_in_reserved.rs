//! A variant's `#[tag(N)]` cannot reuse a tag listed in
//! `#[tagged(reserved(...))]` — the reserved list explicitly retires those
//! tags from being assigned to anything.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(1))]
enum Choice {
    #[tag(0)]
    First,
    #[tag(1)]
    Second,
}

fn main() {}
