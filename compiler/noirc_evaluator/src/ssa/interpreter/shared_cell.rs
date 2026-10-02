//! Defines [SharedCell], the shared mutable storage the SSA interpreter uses for array
//! contents, reference counts and reference pointees.
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

/// A shared, mutable handle to some `T`.
///
/// Cloning a `SharedCell` produces another handle to the same storage, so a write through
/// one handle is observed by every clone. Equality compares the contained values rather
/// than the storage identity; use [SharedCell::as_ptr] to compare identities.
///
/// Code outside the interpreter can only read through a `SharedCell`; creating and
/// mutating cells is internal to the interpreter.
#[derive(Debug)]
pub struct SharedCell<T>(Rc<RefCell<T>>);

impl<T> SharedCell<T> {
    pub(super) fn new(value: T) -> Self {
        SharedCell(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    pub(super) fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }

    /// Takes the contained value, cloning it only if other handles to it remain.
    pub fn unwrap_or_clone(self) -> T
    where
        T: Clone,
    {
        match Rc::try_unwrap(self.0) {
            Ok(value) => value.into_inner(),
            Err(rc) => rc.borrow().clone(),
        }
    }

    /// A pointer identifying the shared allocation itself. Two handles observe each
    /// other's mutations exactly when their `as_ptr` results are equal.
    pub(super) fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}

impl<T> Clone for SharedCell<T> {
    fn clone(&self) -> Self {
        SharedCell(self.0.clone())
    }
}

impl<T: PartialEq> PartialEq for SharedCell<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow() == *other.0.borrow()
    }
}

impl<T: Eq> Eq for SharedCell<T> {}
