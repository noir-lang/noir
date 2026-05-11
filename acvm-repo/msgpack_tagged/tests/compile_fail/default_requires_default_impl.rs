//! `#[tag(N, default)]` requires the field's type to implement `Default` —
//! the macro emits a `T: Default` where bound that triggers a clear error
//! at the impl site if the bound isn't satisfied.

use msgpack_tagged::MsgpackTagged;

/// Has `MsgpackTagged` and `serde::Serialize` impls (via derive) but
/// intentionally NOT `Default`.
#[derive(MsgpackTagged, serde::Serialize)]
struct NotDefault {
    #[tag(0)]
    x: u32,
}

/// `inner` is marked `default`, but `NotDefault: Default` doesn't hold —
/// should fail with "the trait bound `NotDefault: Default` is not satisfied".
/// `serde::Serialize` is derived purely to register the `serde` attribute
/// namespace so `#[serde(default)]` is accepted by rustc — without that
/// pairing, the macro's own validation would fire first (covered by the
/// separate `default_without_serde_default` fixture).
#[derive(MsgpackTagged, serde::Serialize)]
struct Container {
    #[tag(0, default)]
    #[serde(default)]
    inner: NotDefault,
}

fn main() {}
