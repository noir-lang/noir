//! This file holds the pass to convert from Noir's SSA IR to ACIR.

pub(crate) use acir_ir::generated_acir::GeneratedAcir;

use super::ssa_gen::Ssa;

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

impl Ssa {
    pub(crate) fn into_acir(self, _main_param_array_lengths: &[usize]) -> GeneratedAcir {
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    fn new() -> Self {
        Self {}
    }

    fn convert_ssa(&mut self, _ssa: Ssa) -> GeneratedAcir {
        // Milestone 0 is to compile an empty program with no return value. This stub will be
        // filled out for milestone 1.
        GeneratedAcir {
            current_witness_index: 0,
            opcodes: Vec::new(),
            return_witnesses: Vec::new(),
        }
    }
}
