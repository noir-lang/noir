pub(crate) mod function_builder;

use crate::ssa_refactor::ir::{
    function::{Function, FunctionId},
    map::AtomicCounter,
};

/// The global context while building the ssa representation.
/// Because this may be shared across threads, it is synchronized internally as necessary.
#[derive(Default)]
pub(crate) struct SharedBuilderContext {
    function_count: AtomicCounter<Function>,
}

impl SharedBuilderContext {
    pub(super) fn next_function(&self) -> FunctionId {
        self.function_count.next()
    }
}
