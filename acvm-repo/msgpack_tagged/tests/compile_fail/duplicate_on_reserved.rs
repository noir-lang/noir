//! `#[tagged(on_reserved)]` designates a single fallback variant for
//! retired wire tags. Two variants both marked `on_reserved` would leave
//! the wrapper with no single answer to "where do I route this?" — so
//! the macro rejects it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2))]
enum Choice {
    #[tag(0)]
    Real,
    #[tag(8)]
    #[tagged(on_reserved)]
    RetiredA,
    #[tag(9)]
    #[tagged(on_reserved)]
    RetiredB,
}

fn main() {}
