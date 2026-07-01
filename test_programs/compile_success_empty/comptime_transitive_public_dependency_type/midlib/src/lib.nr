// `LeafPublic` is a maximally-public type (`pub struct` in `pub mod`) that lives in `leaflib`, a
// transitive dependency of the downstream binary. The binary can reach it only by reflecting over
// `PublicBox`'s field, but because the type is fully public it is part of the public API surface
// and may legitimately be spliced into the binary's public API.
pub struct PublicBox {
    inner: leaflib::deep::LeafPublic,
}
