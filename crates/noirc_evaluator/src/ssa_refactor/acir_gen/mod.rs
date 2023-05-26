//! This file holds the pass to convert from Noir's SSA IR to ACIR.

pub(crate) use acir_ir::generated_acir::GeneratedAcir;
use noirc_abi::FunctionSignature;

use super::{abi_gen::collate_array_lengths, ssa_gen::Ssa};

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

impl Ssa {
    pub(crate) fn into_acir(self, main_function_signature: FunctionSignature) -> GeneratedAcir {
        let _param_array_lengths = collate_array_lengths(&main_function_signature.0);
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    fn new() -> Self {
        Self {}
    }

    fn convert_ssa(&mut self, _ssa: Ssa) -> GeneratedAcir {
        GeneratedAcir {
            current_witness_index: 0,
            opcodes: Vec::new(),
            return_witnesses: Vec::new(),
        }
    }
}
