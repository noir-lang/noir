use acvm::FieldElement;

use crate::ssa::ir::basic_block::BasicBlockId;

use super::{
    function::FunctionId,
    instruction::{InstructionId, Intrinsic},
    map::Id,
    types::Type,
};

#[derive(Debug)]
pub(crate) struct NumericConstant {
    constant: FieldElement,
    typ: Type,
}

#[derive(Debug)]
pub(crate) struct ArrayOrSlice {
    elements: im::Vector<ValueId>,

    /// This is expected to be either Type::Slice { .. } or Type::Array { .. }
    typ: Type,
}

/// A foreign function is represented by just its name in the SSA IR
pub(crate) type ForeignFunctionName = String;

pub(crate) type NumericConstantId = Id<NumericConstant>;
pub(crate) type ForeignFunctionNameId = Id<ForeignFunctionName>;
pub(crate) type ArrayId = Id<ArrayOrSlice>;

/// ValueId is the basic type in the IR used to represent a Value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ValueId {
    InstructionResult { id: InstructionId, position: u32 },
    Param { block: BasicBlockId, position: u32 },

    NumericConstant(NumericConstantId),

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
    ForeignFunction(ForeignFunctionNameId),
}

impl Value {
    /// Retrieves the type of this Value
    pub(crate) fn get_type(&self) -> &Type {
        match self {
            Value::Instruction { typ, .. } => typ,
            Value::Param { typ, .. } => typ,
            Value::NumericConstant { typ, .. } => typ,
            Value::Array { typ, .. } => typ,
            Value::Function { .. } => &Type::Function,
            Value::Intrinsic { .. } => &Type::Function,
            Value::ForeignFunction { .. } => &Type::Function,
        }
    }
}
