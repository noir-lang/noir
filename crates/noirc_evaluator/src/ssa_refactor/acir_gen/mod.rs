//! This file holds the pass to convert from Noir's SSA IR to ACIR.
use acvm::acir::{circuit::Opcode, native_types::Witness};

use super::ssa_gen::Ssa;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

/// The output of the Acir-gen pass
pub(crate) struct GeneratedAcir {
    /// The next witness index that may be declared.
    ///
    /// Equivalent to acvm::acir::circuit::Circuit's field of the same name.
    pub(crate) current_witness_index: u32,

    /// The opcodes of which the compiled ACIR will comprise.
    pub(crate) opcodes: Vec<Opcode>,

    /// All witness indices that comprise the final return value of the program
    ///
    /// Note: This may contain repeated indices, which is necessary for later mapping into the
    /// abi's return type.
    pub(crate) return_witnesses: Vec<Witness>,
}

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
