#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Brillig bytecode is distinct from regular [ACIR][acir] in that it does not generate constraints.
//!
//! [acir]: https://crates.io/crates/acir
//! [acvm]: https://crates.io/crates/acvm
//! [brillig_vm]: https://crates.io/crates/brillig_vm

mod black_box;
mod foreign_call;
mod opcodes;

pub use black_box::BlackBoxOp;
pub use foreign_call::{ForeignCallParam, ForeignCallResult};
pub use opcodes::{
    BinaryFieldOp, BinaryIntOp, HeapArray, HeapValueType, HeapVector, MemoryAddress, ValueOrArray,
};
pub use opcodes::{BitSize, BrilligOpcode as Opcode, IntegerBitSize, Label};
