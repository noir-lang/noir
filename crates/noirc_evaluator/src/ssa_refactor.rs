//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use noirc_frontend::monomorphization::ast::Program;

use self::acir_gen::Acir;

mod acir_gen;
mod ir;
mod ssa_builder;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub fn optimize_into_acir(program: Program) -> Acir {
    ssa_gen::generate_ssa(program).into_acir()
}
