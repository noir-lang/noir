//! `#[tagged(extra_bound = "...")]` parses the string as a comma-separated
//! list of where-predicates. A malformed string surfaces a clear parse
//! error pointing at the literal — the user knows they need to fix the
//! syntax, not where the bound is being used.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(extra_bound = "this is not a valid where predicate")]
struct Foo {
    #[tag(0)]
    a: u32,
}

fn main() {}
