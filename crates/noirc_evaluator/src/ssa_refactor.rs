//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use crate::errors::RuntimeError;
use acvm::{
    acir::circuit::{Circuit, Opcode},
    Language,
};
use noirc_abi::Abi;

use noirc_frontend::monomorphization::ast::Program;

use self::acir_gen::Acir;

mod acir_gen;
mod ir;
mod opt;
mod ssa_builder;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub fn optimize_into_acir(program: Program) -> Acir {
    ssa_gen::generate_ssa(program).inline_functions().into_acir()
}
/// Compiles the Program into ACIR and applies optimizations to the arithmetic gates
/// This is analogous to `ssa:create_circuit` and this method is called when one wants
/// to use the new ssa module to process Noir code.
pub fn experimental_create_circuit(
    _program: Program,
    _np_language: Language,
    _is_opcode_supported: fn(&Opcode) -> bool,
    _enable_logging: bool,
    _show_output: bool,
) -> Result<(Circuit, Abi), RuntimeError> {
    todo!("this is a stub function for the new SSA refactor module")
}
