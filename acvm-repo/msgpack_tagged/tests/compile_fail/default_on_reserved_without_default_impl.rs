//! Opting into `default_on_reserved` means the runtime decoder will call
//! `Self::default()` when it sees a reserved tag. The macro emits
//! `where Self: Default` so a missing `derive(Default)` surfaces as a clear
//! "Self: Default is not satisfied" error here rather than at runtime.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
#[tagged(reserved(2), default_on_reserved)]
enum BackwardsCompat {
    #[tag(0)]
    First,
    #[tag(1)]
    Second,
}

fn main() {}
