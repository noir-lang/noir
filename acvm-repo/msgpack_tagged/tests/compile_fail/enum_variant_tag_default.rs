//! The `default` modifier — meaningful only for struct fields where the
//! decoder fills `T::default()` if the tag is missing — has no obvious
//! semantics for an enum variant and is rejected.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0, default)]
    First,
}

fn main() {}
