//! `#[tagged(on_reserved)]` marks a fallback routing target. On decode the
//! wrapper discards the wire payload before visiting the fallback, so the
//! variant must be a unit variant — no payload to populate.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2))]
enum Choice {
    #[tag(0)]
    Real,
    #[tag(9)]
    #[tagged(on_reserved)]
    NotUnit(u32),
}

fn main() {}
