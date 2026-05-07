//! Named-variant payload fields follow the same rule as top-level named
//! struct fields: every field needs an explicit `#[tag(N)]` (or auto-skip
//! via `#[tag(skip)]` / `PhantomData<_>`). A bare named field with no tag
//! is a compile error so reordering or inserting fields never silently
//! shifts existing tag values on the wire.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    Multi {
        #[tag(0)]
        a: u32,
        b: bool,
    },
}

fn main() {}
