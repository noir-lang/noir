//! `#[serde(skip)]` on a tuple-style field would shift the positional
//! indices of subsequent fields when implicit tagging is in effect — same
//! brittleness rationale as `#[tag(skip)]` in tuple fields. The macro
//! rejects it rather than silently honoring it.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, serde::Serialize)]
struct Pair(u32, #[serde(skip)] bool);

fn main() {}
