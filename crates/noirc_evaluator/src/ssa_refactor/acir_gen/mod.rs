//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use super::{
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
use acvm::acir::{circuit::opcodes::Opcode as AcirOpcode, native_types::Witness};

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

/// The output of the Acir-gen pass
pub struct GeneratedAcir {
    // Why is this not u64?
    //
    // At the moment, wasm32 is being used in the default backend
    // so it is safer to use a u32, at least until clang is changed
    // to compile wasm64.
    //
    // XXX: Barretenberg, reserves the first index to have value 0.
    // When we increment, we do not use this index at all.
    // This means that every constraint system at the moment, will either need
    // to decrease each index by 1, or create a dummy witness.
    //
    // We ideally want to not have this and have Barretenberg apply the
    // following transformation to the witness index : f(i) = i + 1
    current_witness_index: u32,
    opcodes: Vec<AcirOpcode>,
}

impl GeneratedAcir {
    /// Returns the current witness index.
    // TODO: This can be put as a method on `Circuit` in ACVM
    pub fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    // TODO: This can be put as a method on `Circuit` in ACVM
    fn next_witness_index(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    /// Converts an expression into a Witness.
    ///
    /// This is done by creating a new Witness and creating an opcode which
    /// sets the Witness to be equal to the expression.
    ///
    /// The reason we do this is because _constraints_ in ACIR have a degree limit
    /// This means you cannot multiply an infinite amount of Expressions together.
    /// Once the expression goes over degree-2, then it needs to be reduced to a Witness
    /// which has degree-1 in order to be able to continue the multiplication chain.
    pub fn expression_to_witness(&mut self, expression: Expression) -> Witness {
        let fresh_witness = self.next_witness_index();

        // Create a constraint that sets them to be equal to each other
        // Then return the witness as this can now be used in places
        // where we would have used the Witness.
        let constraint = &expression - fresh_witness;
        self.opcodes.push(AcirOpcode::Arithmetic(constraint));

        fresh_witness
    }
}

impl Ssa {
    pub(crate) fn into_acir(self) -> GeneratedAcir {
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    fn new() -> Self {
        Self {}
    }

    fn convert_ssa(&mut self, ssa: Ssa) -> GeneratedAcir {
        // When converting SSA to ACIR, we expect the legalization pass in the SSA module
        // to ensure the following:
        // - All functions will be inlined
        // - All basic blocks will be inlined
        //
        // When generating ACIR, we therefore only need to look at the entry block's
        // instructions.
        let entry_func = ssa.functions.first().expect("expected at least one function");
        let entry_block_id = entry_func.entry_block();
        let dfg = &entry_func.dfg;
        let entry_block = &dfg[entry_block_id];

        // Instruction Ids for all instructions in the entry block
        let instruction_ids = entry_block.instructions();
        for ins_id in instruction_ids {
            let ins = &entry_func.dfg[*ins_id];
            match ins {
                Instruction::Binary(binary) => self.convert_ssa_binary(binary, dfg),
                _ => todo!(),
            }
        }
        todo!()
    }

    fn convert_ssa_binary(&self, binary: &Binary, dfg: &DataFlowGraph) {
        let _lhs = self.convert_ssa_value(&binary.lhs, dfg);
        let _rhs = self.convert_ssa_value(&binary.rhs, dfg);
        match binary.operator {
            BinaryOp::Add => {}
            BinaryOp::Sub => todo!(),
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

    fn convert_ssa_value(&self, value_id: &Id<Value>, dfg: &DataFlowGraph) -> Expression {
        match dfg[*value_id] {
            Value::Instruction { .. } => todo!(),
            Value::Param { .. } => todo!(),
            Value::NumericConstant { .. } => todo!(),
            Value::Function(_) => unreachable!(),
            Value::Intrinsic(_) => todo!(),
        }
    }
}
