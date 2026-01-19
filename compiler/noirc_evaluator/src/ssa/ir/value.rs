use rustc_hash::FxHashMap as HashMap;
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

pub type ValueId = Id<Value>;

/// Value is the most basic type allowed in the IR.
/// Transition Note: A `Id<Value>` is similar to `NodeId` in our previous IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Value {
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

/// Like `HashMap<ValueId, ValueId>` but handles:
/// 1. recursion (if v0 -> v1 and v1 -> v2, then v0 -> v2)
/// 2. self-mapping values (a value mapped to itself won't be inserted into the HashMap)
#[derive(Default, Debug)]
pub(crate) struct ValueMapping {
    map: HashMap<ValueId, ValueId>,
}

impl ValueMapping {
    pub(crate) fn insert(&mut self, from: ValueId, to: ValueId) {
        if from == to {
            return;
        }

        // If `to` is mapped to something, directly map `from` to that value
        let to = self.get(to);
        self.map.insert(from, to);
    }

    pub(crate) fn batch_insert(&mut self, from: &[ValueId], to: &[ValueId]) {
        assert_eq!(from.len(), to.len(), "Lengths of arrays of values being mapped must match");
        for (from_value, to_value) in from.iter().zip(to) {
            self.insert(*from_value, *to_value);
        }
    }

    pub(crate) fn get(&self, value: ValueId) -> ValueId {
        if let Some(replacement) = self.map.get(&value) { self.get(*replacement) } else { value }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if all [`ValueId`]s are mapped to a [`ValueId`] of the same type.
    ///
    /// Mapping a [`ValueId`] to one of a different type implies a compilation error.
    #[must_use]
    #[cfg(debug_assertions)]
    pub(crate) fn value_types_are_consistent(&self, dfg: &super::dfg::DataFlowGraph) -> bool {
        self.map.iter().all(|(from, to)| dfg.type_of_value(*from) == dfg.type_of_value(*to))
    }
}
