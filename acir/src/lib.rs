// Arbitrary Circuit Intermediate Representation

// XXX: Final version will have acir stdlib which uses arithmetic gates

pub mod circuit;
pub mod native_types;
pub mod optimiser;

pub mod partial_witness_generator;

pub mod opcode;

pub use opcode::OPCODE;