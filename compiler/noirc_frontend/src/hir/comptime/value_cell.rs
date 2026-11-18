//! Defines [ValueCell], the shared mutable storage the comptime interpreter uses for
//! tuple elements, struct fields and pointees.
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::Shared;

use super::Value;

/// A shared, mutable handle to a comptime [Value].
///
/// Cloning a `ValueCell` produces another handle to the same storage, so a write through
/// one handle is observed by every clone. This is what lets `&mut tuple.0` or
/// `&mut my_struct.field` alias the element stored inside the aggregate.
///
/// Equality compares the contained values rather than the storage identity.
///
/// Code outside the interpreter can only read through a `ValueCell`; creating and
/// mutating cells is internal to the interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueCell(Shared<Value>);

impl ValueCell {
    pub(in crate::hir::comptime) fn new(value: Value) -> Self {
        ValueCell(Shared::new(value))
    }

    /// Borrow the value. Panics if it is currently mutably borrowed.
    #[track_caller]
    pub fn borrow(&self) -> RwLockReadGuard<'_, Value> {
        self.0.borrow()
    }

    /// Mutably borrow the value. Panics if it is currently borrowed.
    #[track_caller]
    pub(in crate::hir::comptime) fn borrow_mut(&self) -> RwLockWriteGuard<'_, Value> {
        self.0.borrow_mut()
    }

    /// Takes the contained value, cloning it only if other handles to it remain.
    pub(in crate::hir::comptime) fn unwrap_or_clone(self) -> Value {
        self.0.unwrap_or_clone()
    }
}
