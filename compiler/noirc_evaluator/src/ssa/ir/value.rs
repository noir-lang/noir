use acvm::FieldElement;
use serde::{Deserialize, Serialize};

use crate::ssa::ir::basic_block::BasicBlockId;

use super::{
    function::FunctionId,
    instruction::{InstructionId, Intrinsic},
    map::Id,
    types::NumericType,
};

/// Value is the most basic type allowed in the IR.
/// Transition Note: A Id<Value> is similar to `NodeId` in our previous IR.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum Value {
    /// This value was created due to an instruction
    ///
    /// instruction -- This is the instruction which defined it
    /// typ -- This is the `Type` of the instruction
    /// position -- Returns the position in the results
    /// vector that this `Value` is located.
    /// Example, if you add two numbers together, then the resulting
    /// value would have position `0`, the typ would be the type
    /// of the operands, and the instruction would map to an add instruction.
    Instruction { instruction: InstructionId, position: u16 },

    /// This Value originates from a block parameter. Since function parameters
    /// are also represented as block parameters, this includes function parameters as well.
    ///
    /// position -- the index of this Value in the block parameters list
    Param { block: BasicBlockId, position: u16 },

    /// This Value originates from a numeric constant
    NumericConstant { constant: FieldElementId, typ: NumericType },

    /// This Value refers to a function in the IR.
    /// Functions always have the type Type::Function.
    /// If the argument or return types are needed, users should retrieve
    /// their types via the Call instruction's arguments or the Call instruction's
    /// result types respectively.
    Function(FunctionId),

    /// An Intrinsic is a special kind of builtin function that may be handled internally
    /// or optimized into a special form.
    Intrinsic(Intrinsic),

    /// This Value refers to an external function in the IR.
    /// ForeignFunction's always have the type Type::Function and have similar semantics to Function,
    /// other than generating different backend operations and being only accessible through Brillig.
    ForeignFunction(ForeignFunctionId),
}

pub(crate) struct ForeignFunction(pub(crate) String);
pub(crate) type ForeignFunctionId = Id<ForeignFunction>;

pub(crate) type FieldElementId = Id<FieldElement>;

impl Value {
    pub(crate) fn block_param(block: BasicBlockId, position: u16) -> Self {
        Self::Param { block, position }
    }

    pub(crate) fn instruction_result(instruction: InstructionId, position: u16) -> Self {
        Self::Instruction { instruction, position }
    }

    #[cfg(test)]
    pub(crate) fn test_instruction_result(instruction: u32, position: u16) -> Self {
        Self::Instruction { instruction: Id::test_new(instruction), position }
    }

    /// Return the instruction id associated with this value.
    /// Panics if this is not a Value::Instruction
    pub(crate) fn instruction_id(&self) -> InstructionId {
        match self {
            Value::Instruction { instruction, .. } => *instruction,
            other => panic!("Expected Value::Instruction, found {other}"),
        }
    }

    /// True if this is a constant value like an integer or function.
    /// False if this is an instruction result or parameter.
    pub(crate) fn is_constant(&self) -> bool {
        use Value::*;
        matches!(self, NumericConstant { .. } | Function(_) | Intrinsic(_) | ForeignFunction(_))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Instruction { instruction, position } => {
                // Because these are so common, we don't show the `:0` suffix since
                // most instructions only have 1 result
                if *position == 0 {
                    write!(f, "{instruction}")
                } else {
                    write!(f, "{instruction}.{position}")
                }
            }
            Value::Param { block, position } => write!(f, "{block}.{position}"),
            Value::NumericConstant { constant, typ } => write!(f, "{typ} {constant}"),
            Value::Function(id) => write!(f, "{id}"),
            Value::Intrinsic(intrinsic) => write!(f, "{intrinsic}"),
            Value::ForeignFunction(id) => write!(f, "{id}"),
        }
    }
}
