// Arbitrary Circuit Intermediate Representation

// XXX: Final version will have acir stdlib which uses arithmetic gates

pub mod circuit;
pub mod native_types;
pub mod optimiser;

pub mod opcode;

pub use noir_field::FieldElement;
pub use opcode::OpCode;
