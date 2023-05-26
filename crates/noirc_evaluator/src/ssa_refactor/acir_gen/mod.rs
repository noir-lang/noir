//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use std::collections::HashMap;

use self::acir_ir::{acir_variable::AcirVar, errors::AcirGenError};
use super::{
    abi_gen::collate_array_lengths,
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, InstructionId},
        map::Id,
        value::Value,
    },
    ssa_gen::Ssa,
};
use noirc_abi::FunctionSignature;

pub(crate) use acir_ir::generated_acir::GeneratedAcir;

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {
    /// Maps SSA values to `AcirVar`.
    ///
    /// This is needed so that we only create a single
    /// AcirVar per SSA value. Before creating an `AcirVar`
    /// for an SSA value, we check this map. If an `AcirVar`
    /// already exists for this Value, we return the `AcirVar`.
    ssa_value_to_acir_var: HashMap<Id<Value>, AcirVar>,
}

impl Ssa {
    pub(crate) fn into_acir(self, main_function_signature: FunctionSignature) -> GeneratedAcir {
        let _param_array_lengths = collate_array_lengths(&main_function_signature.0);
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    /// Creates a new `Context` object.
    fn new() -> Self {
        Self { ssa_value_to_acir_var: HashMap::new() }
    }

    /// Converts SSA into ACIR
    fn convert_ssa(&mut self, _ssa: Ssa) -> GeneratedAcir {
        GeneratedAcir {
            current_witness_index: 0,
            opcodes: Vec::new(),
            return_witnesses: Vec::new(),
        }
    }

    /// Converts an SSA value into a `AcirVar`
    fn convert_ssa_value(
        &mut self,
        _value_id: &Id<Value>,
        _dfg: &DataFlowGraph,
    ) -> Result<AcirVar, AcirGenError> {
        todo!()
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(&mut self, _binary: &Binary, _dfg: &DataFlowGraph) -> AcirVar {
        todo!()
    }

    /// Creates an `AcirVar` from an SSA value, if a corresponding `AcirVar` does not already
    /// exist for it.
    fn get_or_create_acir_var(&mut self, _value_id: &Id<Value>, _dfg: &DataFlowGraph) -> AcirVar {
        todo!()
    }

    /// Returns all of the instructions in the program.
    ///
    /// This method assumes that all functions and basic blocks have
    /// been inlined and so we only need to look in the entry block,
    /// in the first function for all instructions.
    fn instructions(ssa: &Ssa) -> Vec<InstructionId> {
        assert_eq!(
            ssa.functions.len(),
            1,
            "expected only a single function to be present with all other functions being inlined."
        );
        let (_, entry_function) = ssa.functions.iter().next().expect(
            "This line should be infallible, since we checked that there is exactly one function",
        );

        let entry_block_id = entry_function.entry_block();
        let dfg = &entry_function.dfg;
        let entry_block = &dfg[entry_block_id];

        entry_block.instructions().to_vec()
    }
}
