use std::{hash::Hash, marker::PhantomData};

use acvm::FieldElement;
use serde::{Deserialize, Serialize};

use crate::ssa::ir::basic_block::BasicBlockId;

use super::{
    function::FunctionId,
    instruction::{InstructionId, Intrinsic},
    map::Id,
    types::Type,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Unresolved;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Resolved;

/// A resolved value ID is something we can directly compare.
pub(crate) type ResolvedValueId = ValueId<Resolved>;

/// A raw value ID that can be used as a key in maps.
pub(crate) type RawValueId = Id<Value>;

/// A value ID that can either be unresolved or resolved. Before it's resolved it's
/// generally not safe to compare IDs with each other, as they might have been replaced
/// during SSA passes, without having updated all the other occurrences.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub(crate) struct ValueId<R = Unresolved> {
    id: Id<Value<R>>,
    #[serde(skip)]
    _marker: PhantomData<R>,
}

impl<R> ValueId<R> {
    pub fn new(id: Id<Value<R>>) -> Self {
        Self { id, _marker: PhantomData }
    }

    /// Access the underlying raw ID for indexing into data structures.
    pub fn raw(&self) -> RawValueId {
        Id::new(self.id.to_usize())
    }
}

impl ValueId<Unresolved> {
    /// Be careful when using this comparison.
    /// Sure the IDs don't have to be resolved first?
    pub fn unresolved_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
    /// Promote an unresolved ID into a resolved one.
    pub fn resolved(self) -> ValueId<Resolved> {
        ValueId::new(Id::new(self.id.to_usize()))
    }
}

impl<R> Copy for ValueId<R> {}

impl<R> Clone for ValueId<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> std::fmt::Debug for ValueId<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl<R> std::fmt::Display for ValueId<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

/// The underlying ID is often used to index into maps, but in general
/// we have to be careful when we use this method and how we compare
/// the raw IDs.
impl<R> AsRef<Id<Value<R>>> for ValueId<R> {
    fn as_ref(&self) -> &Id<Value<R>> {
        &self.id
    }
}

/// Demote a resolved ID into an unresolved one.
impl Into<ValueId<Unresolved>> for ValueId<Resolved> {
    fn into(self) -> ValueId<Unresolved> {
        ValueId::new(self.raw())
    }
}

/// Wrap an `Id` into an equivalent `ValueId``
impl<R> Into<ValueId<R>> for Id<Value<R>> {
    fn into(self) -> ValueId<R> {
        ValueId::new(self)
    }
}

/// Value is the most basic type allowed in the IR.
/// Transition Note: A Id<Value> is similar to `NodeId` in our previous IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) enum Value<R = Unresolved> {
    /// This value was created due to an instruction
    ///
    /// instruction -- This is the instruction which defined it
    /// typ -- This is the `Type` of the instruction
    /// position -- Returns the position in the results
    /// vector that this `Value` is located.
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
    NumericConstant { constant: FieldElement, typ: Type },

    /// Represents a constant array value
    Array { array: im::Vector<ValueId<R>>, typ: Type },

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
}

impl<R> Value<R> {
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
