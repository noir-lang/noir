use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use noirc_frontend::monomorphization::ast::{self, LocalId};
use noirc_frontend::monomorphization::ast::{FuncId, Program};

use crate::ssa_refactor::ssa_builder::Builder;
use crate::ssa_refactor::{
    ir::function::FunctionId as IrFunctionId, ssa_builder::function_builder::FunctionBuilder,
};

use super::value::Value;

// TODO: Make this a threadsafe queue so we can compile functions in parallel
type FunctionQueue = Vec<(ast::FuncId, IrFunctionId)>;

pub(super) struct FunctionContext<'a> {
    definitions: HashMap<LocalId, Value>,
    function_builder: FunctionBuilder<'a>,
    shared_context: &'a Context,
}

/// Shared context for all functions during ssa codegen
pub(super) struct Context {
    functions: RwLock<HashMap<FuncId, IrFunctionId>>,
    function_queue: Mutex<FunctionQueue>,
    pub(super) program: Program,
}

impl<'a> FunctionContext<'a> {
    pub(super) fn new(
        parameter_count: usize,
        shared_context: &'a Context,
        shared_builder_context: &'a Builder,
    ) -> Self {
        Self {
            definitions: HashMap::new(),
            function_builder: FunctionBuilder::new(parameter_count, shared_builder_context),
            shared_context,
        }
    }

    pub(super) fn new_function(&mut self, parameters: impl ExactSizeIterator<Item = LocalId>) {
        self.function_builder.new_function(parameters.len());

        for (_i, _parameter) in parameters.enumerate() {
            todo!("Add block param to definitions")
        }
    }
}

impl Context {
    pub(super) fn new(program: Program) -> Self {
        Self { functions: Default::default(), function_queue: Default::default(), program }
    }

    pub(super) fn pop_next_function_in_queue(&self) -> Option<(ast::FuncId, IrFunctionId)> {
        self.function_queue.lock().expect("Failed to lock function_queue").pop()
    }
}
