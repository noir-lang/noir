//! Defines [ValueCell], the shared mutable storage the comptime interpreter uses for
//! tuple elements, struct fields and pointees.
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

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
#[derive(Clone)]
pub struct ValueCell(Arc<RwLock<Value>>);

impl ValueCell {
    pub(in crate::hir::comptime) fn new(value: Value) -> Self {
        ValueCell(Arc::new(RwLock::new(value)))
    }

    /// Borrow the value. Panics if it is currently mutably borrowed.
    pub fn borrow(&self) -> RwLockReadGuard<'_, Value> {
        match self.0.try_read() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => panic!("ValueCell already mutably borrowed"),
        }
    }

    /// Mutably borrow the value. Panics if it is currently borrowed.
    pub(in crate::hir::comptime) fn borrow_mut(&self) -> RwLockWriteGuard<'_, Value> {
        match self.0.try_write() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => panic!("ValueCell already borrowed"),
        }
    }

    /// Takes the contained value, cloning it only if other handles to it remain.
    pub(in crate::hir::comptime) fn unwrap_or_clone(self) -> Value {
        match Arc::try_unwrap(self.0) {
            Ok(value) => value.into_inner().unwrap_or_else(|poisoned| poisoned.into_inner()),
            Err(rc) => ValueCell(rc).borrow().clone(),
        }
    }
}

impl PartialEq for ValueCell {
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl Eq for ValueCell {}

impl std::fmt::Debug for ValueCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValueCell").field(&*self.borrow()).finish()
    }
}
