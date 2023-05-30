//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use std::collections::HashMap;

use self::acir_ir::acir_variable::{AcirContext, AcirVar};
use super::{
    abi_gen::collate_array_lengths,
    ir::{
        dfg::DataFlowGraph,
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
        },
        map::Id,
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use iter_extended::vecmap;
use noirc_abi::FunctionSignature;

pub(crate) use acir_ir::generated_acir::GeneratedAcir;

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
#[derive(Default)]
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
        let context = Context::default();
        context.convert_ssa(self)
    }
}

impl Context {
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
                todo!("Create an abstract array holding an AcirVar for each element of the reference and bind its ArrayId to the reference.");
            }
            _ => {
                unreachable!("ICE: Params to the program should only contains numerics and arrays")
            }
        }
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let instruction = &dfg[instruction_id];

        let (results_id, results_vars) = match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self.convert_ssa_binary(binary, dfg);
                let result_ids = dfg.instruction_results(instruction_id);
                assert_eq!(result_ids.len(), 1, "Binary ops have a single result");
                (vec![result_ids[0]], vec![result_acir_var])
            }
            Instruction::Constrain(value_id) => {
                let constrain_condition = self.convert_ssa_value(*value_id, dfg);
                self.acir_context.assert_eq_one(constrain_condition);
                (Vec::new(), Vec::new())
            }
            Instruction::Cast(value_id, typ) => {
                let result_acir_var = self.convert_ssa_cast(value_id, typ, dfg);
                let result_ids = dfg.instruction_results(instruction_id);
                assert_eq!(result_ids.len(), 1, "Cast ops have a single result");
                (vec![result_ids[0]], vec![result_acir_var])
            }
            Instruction::Call { func, arguments } => {
                let intrinsic = Self::id_to_intrinsic(*func, dfg);
                let black_box = match intrinsic {
                    Intrinsic::BlackBox(black_box) => black_box,
                    _ => todo!("expected a black box function"),
                };

                let inputs = vecmap(arguments, |value_id| self.convert_ssa_value(*value_id, dfg));
                let outputs = self
                    .acir_context
                    .black_box_function(black_box, inputs)
                    .expect("add Result types to all methods so errors bubble up");

                let result_ids = dfg.instruction_results(instruction_id);
                (result_ids.to_vec(), outputs)
            }
            _ => todo!("{instruction:?}"),
        };

        // Map the results of the instructions to Acir variables
        for (result_id, result_var) in results_id.into_iter().zip(results_vars) {
            self.ssa_value_to_acir_var.insert(result_id, result_var);
        }
    }

    /// Converts a `ValueId` into an `Intrinsic`.
    ///
    /// Panics if the `ValueId` does not represent an intrinsic.
    fn id_to_intrinsic(value_id: ValueId, dfg: &DataFlowGraph) -> Intrinsic {
        let value = &dfg[value_id];
        match value {
            Value::Intrinsic(intrinsic) => *intrinsic,
            _ => unimplemented!("expected an intrinsic call, but found {value:?}"),
        }
    }

    /// Converts an SSA terminator's return values into their ACIR representations
    fn convert_ssa_return(&mut self, terminator: &TerminatorInstruction, dfg: &DataFlowGraph) {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => unreachable!("ICE: Program must have a singular return"),
        };

        // Check if the program returns the `Unit/None` type.
        // This type signifies that the program returns nothing.
        let is_return_unit_type =
            return_values.len() == 1 && dfg.type_of_value(return_values[0]) == Type::Unit;
        if is_return_unit_type {
            return;
        }

        for value_id in return_values {
            let acir_var = self.convert_ssa_value(*value_id, dfg);
            self.acir_context.return_var(acir_var);
        }
    }

    /// Gets the cached `AcirVar` that was converted from the corresponding `ValueId`. If it does
    /// not already exist in the cache, a conversion is attempted and cached for simple values
    /// that require no further context such as numeric types - values requiring more context
    /// should have already been cached elsewhere.
    ///
    /// Conversion is assumed to have already been performed for instruction results and block
    /// parameters. This is because block parameters are converted before anything else, and
    /// because instructions results are converted when the corresponding instruction is
    /// encountered. (An instruction result cannot be referenced before the instruction occurs.)
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
            Value::Intrinsic(..) => unreachable!("intrinsic functions are handled separately"),
            Value::Function(..) => unreachable!("ICE: All functions should have been inlined"),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value:?}")
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
            BinaryOp::Mul => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(lhs, rhs),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => self
                .acir_context
                .less_than_var(lhs, rhs)
                .expect("add Result types to all methods so errors bubble up"),
            _ => todo!(),
        }
    }
    /// Returns an `AcirVar` that is constrained to be
    fn convert_ssa_cast(&mut self, value_id: &ValueId, typ: &Type, dfg: &DataFlowGraph) -> AcirVar {
        let variable = self.convert_ssa_value(*value_id, dfg);

        match typ {
            Type::Numeric(numeric_type) => self
                .acir_context
                .numeric_cast_var(variable, numeric_type)
                .expect("invalid range constraint was applied {numeric_type}"),
            _ => unimplemented!("The cast operation is only valid for integers."),
        }
    }
}
