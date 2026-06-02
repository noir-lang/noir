//! Helper constructs to ensure that no parts are forgotten when pushing expressions and statements piecemeal.
use std::{marker::PhantomData, ops::Deref};

use noirc_errors::Location;

use crate::{
    Type,
    node_interner::{ExprId, NodeInterner},
};

pub struct HasNothing;
pub struct HasLocation;

pub struct PushedExpr<S = HasNothing> {
    id: ExprId,
    has_location: bool,
    has_type: bool,
    status: PhantomData<S>,
}

impl PushedExpr<HasNothing> {
    /// Create a new pusher that will need the location and the type pushed.
    pub fn new(id: ExprId) -> Self {
        Self { id, has_location: false, has_type: false, status: PhantomData }
    }

    /// Push the location first, then the type.
    pub fn push_location(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> PushedExpr<HasLocation> {
        let id = self.id;
        interner.push_expr_location(id, location);
        std::mem::forget(self);
        PushedExpr { id, has_location: true, has_type: false, status: PhantomData }
    }
}

impl PushedExpr<HasLocation> {
    /// Push the type after the location, returning the ID as there are no more missing pieces.
    pub fn push_type(mut self, interner: &mut NodeInterner, typ: Type) -> ExprId {
        interner.push_expr_type(self.id, typ);
        self.has_type = true;
        self.id
    }
}

/// Give convenient access to the ID in case it is needed to make the next piece available.
impl<S> Deref for PushedExpr<S> {
    type Target = ExprId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

/// Panic if we dropped the pusher without having pushed both location and type.
impl<S> Drop for PushedExpr<S> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Do not mask another panic with this; if it's already panicking,
            // that could be a reason why e.g. the type hasn't been pushed.
            return;
        }
        if !self.has_location {
            panic!("location hasn't been pushed for {:?}", self.id);
        }
        if !self.has_type {
            panic!("type hasn't been pushed for {:?}", self.id);
        }
    }
}
