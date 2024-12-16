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
    Instruction { instruction: InstructionId, position: usize },

    /// This Value originates from a block parameter. Since function parameters
    /// are also represented as block parameters, this includes function parameters as well.
    ///
    /// position -- the index of this Value in the block parameters list
    Param { block: BasicBlockId, position: usize },

    /// This Value originates from a numeric constant
    NumericConstant { constant: FieldElement, typ: NumericType },

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

impl Value {
    pub(crate) fn constant(constant: FieldElement, typ: NumericType) -> Self {
        Self::NumericConstant { constant, typ }
    }

    pub(crate) fn field_constant(constant: FieldElement) -> Self {
        Self::NumericConstant { constant, typ: NumericType::NativeField }
    }

    pub(crate) fn length_constant(constant: FieldElement) -> Self {
        Self::NumericConstant { constant, typ: NumericType::length_type() }
    }

    pub(crate) fn bool_constant(constant: bool) -> Self {
        Self::NumericConstant { constant: constant.into(), typ: NumericType::bool() }
    }

    pub(crate) fn block_param(block: BasicBlockId, position: usize) -> Self {
        Self::Param { block, position }
    }

    pub(crate) fn instruction_result(instruction: InstructionId, position: usize) -> Self {
        Self::Instruction { instruction, position }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Instruction { instruction, position } => write!(f, "{instruction}:{position}"),
            Value::Param { block, position } => write!(f, "{block}:{position}"),
            Value::NumericConstant { constant, typ } => write!(f, "{typ} {constant}"),
            Value::Function(id) => write!(f, "{id}"),
            Value::Intrinsic(intrinsic) => write!(f, "{intrinsic}"),
            Value::ForeignFunction(id) => write!(f, "{id}"),
        }
    }
}
