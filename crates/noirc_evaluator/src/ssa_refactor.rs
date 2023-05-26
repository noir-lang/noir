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
    acir::circuit::{Circuit, Opcode as AcirOpcode, PublicInputs},
    Language,
};
use noirc_abi::Abi;

use noirc_frontend::monomorphization::ast::Program;

use self::{abi_gen::gen_abi, acir_gen::GeneratedAcir, ssa_gen::Ssa};

mod abi_gen;
mod acir_gen;
mod ir;
mod opt;
mod ssa_builder;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub(crate) fn optimize_into_acir(program: Program) -> GeneratedAcir {
    let func_signature = program.main_function_signature.clone();
    ssa_gen::generate_ssa(program)
        .print("Initial SSA:")
        .inline_functions()
        .print("After Inlining:")
        .unroll_loops()
        .print("After Unrolling:")
        .simplify_cfg()
        .print("After Simplifying:")
        .flatten_cfg()
        .print("After Flattening:")
        .mem2reg()
        .print("After Mem2Reg:")
        .into_acir(func_signature)
}

/// Compiles the Program into ACIR and applies optimizations to the arithmetic gates
/// This is analogous to `ssa:create_circuit` and this method is called when one wants
/// to use the new ssa module to process Noir code.
pub fn experimental_create_circuit(
    program: Program,
    _np_language: Language,
    _is_opcode_supported: &impl Fn(&AcirOpcode) -> bool,
    _enable_logging: bool,
    _show_output: bool,
) -> Result<(Circuit, Abi), RuntimeError> {
    let func_sig = program.main_function_signature.clone();
    let GeneratedAcir { current_witness_index, opcodes, return_witnesses } =
        optimize_into_acir(program);

    let abi = gen_abi(func_sig, return_witnesses.clone());
    let public_abi = abi.clone().public_abi();

    let public_parameters =
        PublicInputs(public_abi.param_witnesses.values().flatten().copied().collect());
    let return_values = PublicInputs(return_witnesses.into_iter().collect());
    let circuit = Circuit { current_witness_index, opcodes, public_parameters, return_values };
    Ok((circuit, abi))
}

impl Ssa {
    fn print(self, msg: &str) -> Ssa {
        println!("{msg}\n{self}");
        self
    }
}
