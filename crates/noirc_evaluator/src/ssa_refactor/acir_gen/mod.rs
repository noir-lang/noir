//! This file holds the pass to convert from Noir's SSA IR to ACIR.

mod acir_variable;
mod errors;
pub(crate) mod generated_acir;

use self::generated_acir::GeneratedAcir;

use super::{
    abi_gen::parse_abi::ProcessedAbiParam,
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, BinaryOp},
        map::Id,
        value::Value,
    },
    ssa_gen::Ssa,
};
use crate::ssa_refactor::ir::instruction::Instruction;

use acvm::acir::native_types::Expression;
use acvm::acir::native_types::Witness;
use noirc_abi::FunctionSignature;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {
    acir: GeneratedAcir,
    /// Function signature of the function we are converting to
    /// ACIR. ACIR generation assumes there is only one function
    /// and all other functions have been inlined.
    function_signature: FunctionSignature,

    processed_params: Vec<ProcessedAbiParam>,
}

impl Ssa {
    // TODO We ideally don't want to bring `FunctionSignature` into acir_gen
    // TODO so we should find a way to abstract away what is needed here.
    pub(crate) fn into_acir(self, function_signature: FunctionSignature) -> GeneratedAcir {
        let mut context = Context::new(function_signature);
        context.convert_ssa(self);
        context.acir
    }
}

impl Context {
    fn new(function_signature: FunctionSignature) -> Self {
        Self { function_signature, acir: GeneratedAcir::default(), processed_params: Vec::new() }
    }

    fn convert_ssa(&mut self, ssa: Ssa) {
        // When converting SSA to ACIR, we expect the legalization pass in the SSA module
        // to ensure the following:
        // - All functions will be inlined
        // - All basic blocks will be inlined
        //
        // When generating ACIR, we therefore only need to look at the entry block's
        // instructions.
        // TODO: should we do an expect for 1 function exactly?
        let entry_func = ssa.functions.first().expect("expected at least one function");
        let entry_block_id = entry_func.entry_block();
        let dfg = &entry_func.dfg;
        let entry_block = &dfg[entry_block_id];
        self.parse_abi();

        // Instruction Ids for all instructions in the entry block
        let instruction_ids = entry_block.instructions();
        for ins_id in instruction_ids {
            let ins = &entry_func.dfg[*ins_id];
            let expression = match ins {
                Instruction::Binary(binary) => self.convert_ssa_binary(binary, dfg),
                _ => todo!(),
            };
            self.acir.assert_is_zero(expression)
        }
    }

    fn parse_abi(&mut self) {
        let (processed_params, witness_offset) =
            super::abi_gen::parse_abi::parse_abi(&self.function_signature);

        self.processed_params = processed_params;
        self.acir.current_witness_index = witness_offset;
    }

    fn convert_ssa_binary(&mut self, binary: &Binary, dfg: &DataFlowGraph) -> Expression {
        let _lhs = self.convert_ssa_value(&binary.lhs, dfg);
        let _rhs = self.convert_ssa_value(&binary.rhs, dfg);
        match binary.operator {
            BinaryOp::Add => &_lhs + &_rhs,
            BinaryOp::Sub => &_lhs - &_rhs,
            BinaryOp::Mul => todo!(),
            BinaryOp::Div => todo!(),
            BinaryOp::Mod => todo!(),
            BinaryOp::Eq => todo!(),
            BinaryOp::Lt => todo!(),
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::Xor => todo!(),
            BinaryOp::Shl => todo!(),
            BinaryOp::Shr => todo!(),
        }
    }

    fn convert_ssa_value(&mut self, value_id: &Id<Value>, dfg: &DataFlowGraph) -> Expression {
        match dfg[*value_id] {
            Value::Instruction { instruction, position, typ } => todo!("instruction not done"),
            Value::Param { block, position, typ } => {
                // Since we only have one basic block. This is a parameter to the main function.
                //

                // Block parameters already have their witnesses created, so we fetch it from a map
                let processed_param = &self.processed_params[position];
                let num_elements_needed = processed_param.num_field_elements_needed;
                if num_elements_needed == 1 {
                    return Witness(processed_param.witness_start).into();
                } else {
                    todo!("cannot do this")
                };
            }

            Value::NumericConstant { constant, typ } => {
                let value = dfg[constant].value();
                Expression::from(value)
            }
            Value::Function(_) => unreachable!(),
            Value::Intrinsic(_) => todo!("intrinsic not done"),
        }
    }
}
