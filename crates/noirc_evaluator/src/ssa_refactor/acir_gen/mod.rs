//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use std::collections::HashMap;

use self::acir_ir::acir_variable::{AcirContext, AcirVar};
use super::{
    abi_gen::collate_array_lengths,
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
        map::Id,
        types::Type,
        value::{Value, ValueId},
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
    ///
    acir_context: AcirContext,
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
        Self { ssa_value_to_acir_var: HashMap::new(), acir_context: AcirContext::new() }
    }

    /// Converts SSA into ACIR
    fn convert_ssa(mut self, ssa: Ssa) -> GeneratedAcir {
        assert_eq!(
            ssa.functions.len(),
            1,
            "expected only a single function to be present with all other functions being inlined."
        );
        let main_func = ssa.main();
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];

        for param_id in entry_block.parameters() {
            self.convert_ssa_block_param(*param_id, dfg);
        }

        for instruction_id in entry_block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg);
        }

        self.convert_ssa_return(entry_block.terminator().unwrap(), dfg);

        self.acir_context.finish()
    }

    /// Adds and binds an AcirVar for each numeric block parameter
    ///
    /// TODO: In the case of references, add AcirVars corresponding to the length of the array
    /// and bind an ArrayId.
    fn convert_ssa_block_param(&mut self, param_id: ValueId, dfg: &DataFlowGraph) {
        let value = dfg[param_id];
        let param_type = match value {
            Value::Param { typ, .. } => typ,
            _ => unreachable!("ICE: Only Param type values should appear in block parameters"),
        };
        match param_type {
            Type::Numeric(..) => {
                let acir_var = self.acir_context.add_variable();
                self.ssa_value_to_acir_var.insert(param_id, acir_var);
            }
            Type::Reference => {
                todo!("Alloc array for reference");
            }
            _ => unreachable!("ICE: invalid type {param_type} found in parameters"),
        }
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let instruction = &dfg[instruction_id];
        match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self.convert_ssa_binary(binary, dfg);
                let result_value_id = dfg
                    .instruction_results(instruction_id)
                    .first()
                    .expect("Binary ops have a single result");
                self.ssa_value_to_acir_var.insert(*result_value_id, result_acir_var);
            }
            _ => todo!(),
        }
    }

    fn convert_ssa_return(&mut self, terminator: &TerminatorInstruction, dfg: &DataFlowGraph) {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => unreachable!("ICE: Program must have a singular return"),
        };
        for value_id in return_values {
            let acir_var = self
                .ssa_value_to_acir_var
                .get(value_id)
                .expect("ICE: Return of value not yet encountered");
            self.acir_context.return_var(*acir_var);
        }
    }

    /// Converts an SSA value into a `AcirVar`
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirVar {
        let value = &dfg[value_id];
        if let Some(acir_var) = self.ssa_value_to_acir_var.get(&value_id) {
            return *acir_var;
        }
        let acir_var = match value {
            Value::NumericConstant { constant, .. } => {
                let field_element = &dfg[*constant].value();
                self.acir_context.add_constant(*field_element)
            }
            Value::Intrinsic(..) => todo!(),
            Value::Function(..) => unreachable!("ICE: All functions should have been inlined"),
            // `ssa_value_to_acir_var.get(value_id)` should have already returned the
            // corresponding `AcirVar`.
            Value::Instruction { .. } => {
                unreachable!("ICE: Instruction result value used before instruction encountered")
            }
            Value::Param { .. } => {
                unreachable!("ICE: All block params should have already been bound")
            }
        };
        self.ssa_value_to_acir_var.insert(value_id, acir_var);
        acir_var
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(&mut self, binary: &Binary, dfg: &DataFlowGraph) -> AcirVar {
        let lhs = self.convert_ssa_value(binary.lhs, dfg);
        let rhs = self.convert_ssa_value(binary.rhs, dfg);
        match binary.operator {
            BinaryOp::Add => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub => self.acir_context.sub_var(lhs, rhs),
            _ => todo!(),
        }
    }
}
