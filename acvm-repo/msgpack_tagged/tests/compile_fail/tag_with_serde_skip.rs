//! `#[tag(N)]` says "put this field on the wire under tag N"; `#[serde(skip)]`
//! says "don't put this field on the wire at all". They contradict each
//! other on the same field, so the macro rejects the combination rather
//! than silently picking one.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, serde::Serialize)]
struct Conflict {
    #[tag(0)]
    #[serde(skip)]
    payload: u32,
}

fn main() {}
