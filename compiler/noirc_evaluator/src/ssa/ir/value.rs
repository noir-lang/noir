use std::borrow::Cow;

use acvm::FieldElement;
use serde::{Deserialize, Serialize};

use crate::ssa::ir::basic_block::BasicBlockId;

use super::{
    function::FunctionId,
    instruction::{InstructionId, Intrinsic},
    map::Id,
    types::{NumericType, Type},
};

pub(crate) type ValueId = Id<Value>;

/// Value is the most basic type allowed in the IR.
/// Transition Note: A Id<Value> is similar to `NodeId` in our previous IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) enum Value {
    /// This value was created due to an instruction
    ///
    /// * `instruction`: This is the instruction which defined it
    /// * `typ`: This is the `Type` of the instruction
    /// * `position`: Returns the position in the results vector that this `Value` is located.
    ///
    /// Example, if you add two numbers together, then the resulting
    /// value would have position `0`, the typ would be the type
    /// of the operands, and the instruction would map to an add instruction.
    Instruction { instruction: InstructionId, position: usize, typ: Type },

    /// This Value originates from a block parameter. Since function parameters
    /// are also represented as block parameters, this includes function parameters as well.
    ///
    /// position -- the index of this Value in the block parameters list
    Param { block: BasicBlockId, position: usize, typ: Type },

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
    ForeignFunction(String),

    /// This Value indicates we have a reserved slot that needs to be accessed in a separate global context
    Global(Type),
}

impl Value {
    /// Retrieves the type of this Value
    pub(crate) fn get_type(&self) -> Cow<Type> {
        match self {
            Value::Instruction { typ, .. } | Value::Param { typ, .. } => Cow::Borrowed(typ),
            Value::NumericConstant { typ, .. } => Cow::Owned(Type::Numeric(*typ)),
            Value::Function { .. } | Value::Intrinsic { .. } | Value::ForeignFunction { .. } => {
                Cow::Owned(Type::Function)
            }
            Value::Global(typ) => Cow::Borrowed(typ),
        }
    }
}
