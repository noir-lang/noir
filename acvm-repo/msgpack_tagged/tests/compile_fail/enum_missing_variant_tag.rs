//! Every enum variant must carry an explicit `#[tag(N)]` — there's no implicit
//! numbering. Missing it is a compile error so reordering or inserting variants
//! never silently shifts existing tag values on the wire.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    First,
    Second,
}

fn main() {}
