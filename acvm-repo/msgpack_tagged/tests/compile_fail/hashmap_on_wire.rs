//! `HashMap` is deliberately not supported on the wire (its iteration order is
//! non-deterministic). Reaching for it should produce a compile error with the
//! diagnostic note pointing at `BTreeMap`.

use std::collections::HashMap;

use msgpack_tagged::MsgpackTagged;

fn requires_tagged<T: MsgpackTagged>() {}

fn main() {
    requires_tagged::<HashMap<String, u32>>();
}
