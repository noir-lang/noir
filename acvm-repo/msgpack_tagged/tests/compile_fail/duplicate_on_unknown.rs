//! Same rule as `duplicate_on_reserved`, but for `on_unknown`: only one
//! variant per marker kind so the wrapper has an unambiguous routing target.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    Real,
    #[tag(8)]
    #[tagged(on_unknown)]
    UnknownA,
    #[tag(9)]
    #[tagged(on_unknown)]
    UnknownB,
}

fn main() {}
