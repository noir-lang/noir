use std::{cell::RefCell, rc::Rc};

/// A shared, mutable reference to some T: the storage behind the interpreter's arrays and
/// references, which every copy of a value aliases.
/// Wrapper is required for Hash impl of `RefCell`.
#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T: std::hash::Hash> std::hash::Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl<T: PartialEq> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        let ref1 = self.0.borrow();
        let ref2 = other.0.borrow();
        *ref1 == *ref2
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T> From<T> for Shared<T> {
    fn from(thing: T) -> Shared<T> {
        Shared::new(thing)
    }
}

impl<T> Shared<T> {
    pub fn new(thing: T) -> Shared<T> {
        Shared(Rc::new(RefCell::new(thing)))
    }

    /// A pointer identifying the shared allocation itself, for identity comparisons.
    /// Two `Shared` handles observe each other's mutations exactly when their
    /// `as_ptr` results are equal (note that `PartialEq` compares contents instead).
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn unwrap_or_clone(self) -> T
    where
        T: Clone,
    {
        match Rc::try_unwrap(self.0) {
            Ok(elem) => elem.into_inner(),
            Err(rc) => rc.as_ref().clone().into_inner(),
        }
    }
}
