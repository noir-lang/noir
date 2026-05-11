//! `default` is a *modifier* on a numbered tag, not a tag form on its own.
//! `#[tag(default)]` should produce a parse error pointing at the `default`
//! ident — only `skip` is valid as a bare keyword inside `#[tag(...)]`.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
struct Foo {
    #[tag(default)]
    a: u32,
}

fn main() {}
