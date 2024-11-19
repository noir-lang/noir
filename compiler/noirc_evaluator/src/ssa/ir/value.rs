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
pub(crate) enum Unresolved {}

/// Marker for resolved status.
///
/// It has a lifetime so it's not easy to store it in data structures forever,
/// where it could become stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
pub(crate) struct Resolved<'a> {
    _marker: PhantomData<&'a ()>,
}

pub(crate) type ResolvedValueId<'a> = ValueId<Resolved<'a>>;
pub(crate) type FinalValueId = ValueId<Resolved<'static>>;

pub(crate) trait IsResolved {}

impl<'a> IsResolved for Resolved<'a> {}

pub(crate) trait Resolution {
    fn is_resolved() -> bool;
}

impl Resolution for Unresolved {
    fn is_resolved() -> bool {
        false
    }
}

impl<R: IsResolved> Resolution for R {
    fn is_resolved() -> bool {
        true
    }
}

/// A raw value ID that can be used as a key in maps.
pub(crate) type RawValueId = Id<Value>;

/// A value ID that can either be unresolved or resolved. Before it's resolved it's
/// generally not safe to compare IDs with each other, as they might have been replaced
/// during SSA passes, without having updated all the other occurrences.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub(crate) struct ValueId<R = Unresolved> {
    id: Id<Value>,
    #[serde(skip)]
    _marker: PhantomData<R>,
}

impl<R> ValueId<R> {
    pub(crate) fn new(id: Id<Value>) -> Self {
        Self { id, _marker: PhantomData }
    }

    /// Access the underlying raw ID for indexing into data structures.
    pub(crate) fn raw(&self) -> RawValueId {
        Id::new(self.id.to_usize())
    }

    /// Demote an ID into an unresolved one.
    pub(crate) fn unresolved(self) -> ValueId<Unresolved> {
        ValueId::new(self.id)
    }
}

impl ValueId<Unresolved> {
    /// Be careful when using this comparison.
    /// Sure the IDs don't have to be resolved first?
    pub(crate) fn unresolved_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
    #[cfg(test)]
    pub(crate) fn resolved(self) -> ValueId<Resolved<'static>> {
        ValueId::new(self.id)
    }
}

impl<'a> ValueId<Resolved<'a>> {
    /// Change the lifetime of a resolution.
    ///
    /// This is typically used to detach the lifetime of a resolved value ID
    /// from the `DataFlowGraph` which was used to resolve it, so that it
    /// can live in a different context.
    pub(crate) fn detach<'b>(self) -> ValueId<Resolved<'b>> {
        ValueId::new(self.id)
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
impl<R> AsRef<Id<Value>> for ValueId<R> {
    fn as_ref(&self) -> &Id<Value> {
        &self.id
    }
}

/// Demote a resolved ID into an unresolved one.
impl<R: IsResolved> From<ValueId<R>> for ValueId<Unresolved> {
    fn from(value: ValueId<R>) -> Self {
        value.unresolved()
    }
}
impl From<Id<Value>> for ValueId<Unresolved> {
    fn from(value: Id<Value>) -> Self {
        ValueId::new(value)
    }
}
impl From<&Id<Value>> for ValueId<Unresolved> {
    fn from(value: &Id<Value>) -> Self {
        ValueId::new(*value)
    }
}

/// Value is the most basic type allowed in the IR.
/// Transition Note: A Id<Value> is similar to `NodeId` in our previous IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
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
    Instruction { instruction: InstructionId, position: usize, typ: Type },

    /// This Value originates from a block parameter. Since function parameters
    /// are also represented as block parameters, this includes function parameters as well.
    ///
    /// position -- the index of this Value in the block parameters list
    Param { block: BasicBlockId, position: usize, typ: Type },

    /// This Value originates from a numeric constant
    NumericConstant { constant: FieldElement, typ: Type },

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

impl Value {
    /// Retrieves the type of this Value
    pub(crate) fn get_type(&self) -> &Type {
        match self {
            Value::Instruction { typ, .. } => typ,
            Value::Param { typ, .. } => typ,
            Value::NumericConstant { typ, .. } => typ,
            Value::Function { .. } => &Type::Function,
            Value::Intrinsic { .. } => &Type::Function,
            Value::ForeignFunction { .. } => &Type::Function,
        }
    }
}
