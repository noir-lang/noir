//! `#[tag(N, default)]` requires the field's type to implement `Default` —
//! the macro emits a `T: Default` where bound that triggers a clear error
//! at the impl site if the bound isn't satisfied.

use msgpack_tagged::MsgpackTagged;

/// Has `MsgpackTagged` impl (via derive) but intentionally NOT `Default`.
#[derive(MsgpackTagged)]
struct NotDefault {
    #[tag(0)]
    x: u32,
}

/// `inner` is marked `default`, but `NotDefault: Default` doesn't hold —
/// should fail with "the trait bound `NotDefault: Default` is not satisfied".
#[derive(MsgpackTagged)]
struct Container {
    #[tag(0, default)]
    inner: NotDefault,
}

fn main() {}
