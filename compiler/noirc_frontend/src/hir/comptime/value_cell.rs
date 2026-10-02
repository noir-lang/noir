//! Defines [ValueCell], the shared mutable storage the comptime interpreter uses for
//! tuple elements, struct fields and pointees.
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

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
#[derive(Debug, Clone)]
pub struct ValueCell(Rc<RefCell<Value>>);

impl ValueCell {
    pub(in crate::hir::comptime) fn new(value: Value) -> Self {
        ValueCell(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<'_, Value> {
        self.0.borrow()
    }

    pub(in crate::hir::comptime) fn borrow_mut(&self) -> RefMut<'_, Value> {
        self.0.borrow_mut()
    }

    /// Takes the contained value, cloning it only if other handles to it remain.
    pub(in crate::hir::comptime) fn unwrap_or_clone(self) -> Value {
        match Rc::try_unwrap(self.0) {
            Ok(value) => value.into_inner(),
            Err(rc) => rc.borrow().clone(),
        }
    }
}

impl PartialEq for ValueCell {
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow() == *other.0.borrow()
    }
}

impl Eq for ValueCell {}
