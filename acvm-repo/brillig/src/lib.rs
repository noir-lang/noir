#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]
#![doc = include_str!("../README.md")]

mod black_box;
mod foreign_call;
mod opcodes;

pub use black_box::BlackBoxOp;
pub use foreign_call::{ForeignCallParam, ForeignCallResult};
pub use opcodes::{
    BinaryFieldOp, BinaryIntOp, HeapArray, HeapValueType, HeapVector, MemoryAddress, ValueOrArray,
};
pub use opcodes::{BitSize, BrilligOpcode as Opcode, IntegerBitSize, Label};
