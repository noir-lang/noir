//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use crate::errors::{RuntimeError, RuntimeErrorKind};
use acvm::{
    acir::circuit::{Circuit, Opcode as AcirOpcode},
    Language,
};
use noirc_abi::Abi;

use noirc_frontend::monomorphization::ast::Program;

use self::ssa_gen::Ssa;

mod acir_gen;
mod ir;
mod opt;
mod ssa_builder;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub fn optimize_into_acir(program: Program) {
    ssa_gen::generate_ssa(program)
        .print("Initial SSA:")
        .inline_functions()
        .print("After Inlining:")
        .unroll_loops()
        .print("After Unrolling:");
}

/// Compiles the Program into ACIR and applies optimizations to the arithmetic gates
/// This is analogous to `ssa:create_circuit` and this method is called when one wants
/// to use the new ssa module to process Noir code.
pub fn experimental_create_circuit(
    _program: Program,
    _np_language: Language,
    _is_opcode_supported: &impl Fn(&AcirOpcode) -> bool,
    _enable_logging: bool,
    _show_output: bool,
) -> Result<(Circuit, Abi), RuntimeError> {
    optimize_into_acir(_program);
    let error_kind = RuntimeErrorKind::Spanless("Acir-gen is unimplemented".into());
    Err(RuntimeError::new(error_kind, None))
}

impl Ssa {
    fn print(self, msg: &str) -> Ssa {
        println!("{msg}\n{self}");
        self
    }
}
