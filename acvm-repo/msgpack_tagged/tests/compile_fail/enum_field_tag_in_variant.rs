//! Per-variant struct/tuple field tagging is not yet supported — only the
//! variant itself can be tagged. Field-level `#[tag(...)]` inside a variant's
//! payload is rejected with a clear error rather than being silently ignored.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    Multi {
        #[tag(0)]
        a: u32,
    },
}

fn main() {}
