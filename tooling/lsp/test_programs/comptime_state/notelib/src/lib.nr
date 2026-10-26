use macros::expect_registered;

// A library that registers into the macro library's state during its own elaboration.
// `expect_registered` is imported but unused on purpose: the warning must be reported once
// even though this library is elaborated once per dependent.
pub mod notes;

pub use notes::LibThing;
