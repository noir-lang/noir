#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// Arbitrary Circuit Intermediate Representation

pub mod circuit;
pub mod native_types;

pub use acir_field;
pub use acir_field::FieldElement;
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;
