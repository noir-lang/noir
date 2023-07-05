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

use self::{abi_gen::gen_abi, acir_gen::GeneratedAcir, ir::function::RuntimeType, ssa_gen::Ssa};

mod abi_gen;
mod acir_gen;
pub mod ir;
mod opt;
mod ssa_builder;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub(crate) fn optimize_into_acir(
    program: Program,
    allow_log_ops: bool,
    print_ssa_passes: bool,
) -> GeneratedAcir {
    let abi_distinctness = program.return_distinctness;
    let mut ssa = ssa_gen::generate_ssa(program).print(print_ssa_passes, "Initial SSA:");
    let brillig = ssa.to_brillig();
    if let RuntimeType::Acir = ssa.main().runtime() {
        ssa = ssa
            .inline_functions()
            .print(print_ssa_passes, "After Inlining:")
            .unroll_loops()
            .print(print_ssa_passes, "After Unrolling:")
            .simplify_cfg()
            .print(print_ssa_passes, "After Simplifying:")
            .flatten_cfg()
            .print(print_ssa_passes, "After Flattening:")
            .mem2reg()
            .print(print_ssa_passes, "After Mem2Reg:")
            .fold_constants()
            .print(print_ssa_passes, "After Constant Folding:")
            .dead_instruction_elimination()
            .print(print_ssa_passes, "After Dead Instruction Elimination:");
    }
    ssa.into_acir(brillig, abi_distinctness, allow_log_ops)
}

/// Compiles the Program into ACIR and applies optimizations to the arithmetic gates
/// This is analogous to `ssa:create_circuit` and this method is called when one wants
/// to use the new ssa module to process Noir code.
pub fn experimental_create_circuit(
    program: Program,
    np_language: Language,
    is_opcode_supported: &impl Fn(&AcirOpcode) -> bool,
    enable_logging: bool,
    show_output: bool,
) -> Result<(Circuit, Abi), RuntimeError> {
    let func_sig = program.main_function_signature.clone();
    let GeneratedAcir { current_witness_index, opcodes, return_witnesses, .. } =
        optimize_into_acir(program, show_output, enable_logging);

    let abi = gen_abi(func_sig, return_witnesses.clone());
    let public_abi = abi.clone().public_abi();

    let public_parameters =
        PublicInputs(public_abi.param_witnesses.values().flatten().copied().collect());
    let return_values = PublicInputs(return_witnesses.into_iter().collect());

    // This region of code will optimize the ACIR bytecode for a particular backend
    // it will be removed in the near future and we will subsequently only return the
    // unoptimized backend-agnostic bytecode here
    let optimized_circuit = {
        use crate::errors::RuntimeErrorKind;
        use acvm::compiler::CircuitSimplifier;

        let abi_len = abi.field_count();

        let simplifier = CircuitSimplifier::new(abi_len);
        acvm::compiler::compile(
            Circuit { current_witness_index, opcodes, public_parameters, return_values },
            np_language,
            is_opcode_supported,
            &simplifier,
        )
        .map_err(|_| RuntimeErrorKind::Spanless(String::from("produced an acvm compile error")))?
    };

    Ok((optimized_circuit, abi))
}

impl Ssa {
    fn print(self, print_ssa_passes: bool, msg: &str) -> Ssa {
        if print_ssa_passes {
            println!("{msg}\n{self}");
        }
        self
    }
}
