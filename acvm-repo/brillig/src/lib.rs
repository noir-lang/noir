#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Brillig bytecode is distinct from regular [ACIR][acir] in that it does not generate constraints.
//! This is a generalization over the fixed directives that exists within in the ACVM.
//!
//! [acir]: https://crates.io/crates/acir
//! [acvm]: https://crates.io/crates/acvm
//! [brillig_vm]: https://crates.io/crates/brillig_vm

mod black_box;
mod foreign_call;
mod opcodes;
mod value;

pub use black_box::BlackBoxOp;
pub use foreign_call::{ForeignCallOutput, ForeignCallResult};
pub use opcodes::{
    BinaryFieldOp, BinaryIntOp, HeapArray, HeapVector, RegisterIndex, RegisterOrMemory,
};
pub use opcodes::{Label, Opcode};
pub use value::Typ;
pub use value::Value;
