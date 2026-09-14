//! When the enum delegates its wire shape via `#[tagged(via(...))]`, every
//! variant of it is wire-irrelevant — including any per-variant
//! `#[tagged(...)]` configuration. Caught here rather than silently dropped.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct WireDto {
    #[tag(0)]
    payload: u32,
}

#[derive(MsgpackTagged)]
#[tagged(via(WireDto))]
enum Public {
    #[tagged(reserved(2))]
    First,
}

fn main() {}
