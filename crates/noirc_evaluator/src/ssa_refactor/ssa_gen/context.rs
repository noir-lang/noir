use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::{self, LocalId, Parameters};
use noirc_frontend::monomorphization::ast::{FuncId, Program};
use noirc_frontend::Signedness;

use crate::ssa_refactor::ir::types::Type;
use crate::ssa_refactor::ssa_builder::SharedBuilderContext;
use crate::ssa_refactor::{
    ir::function::FunctionId as IrFunctionId, ssa_builder::function_builder::FunctionBuilder,
};

use super::value::{Tree, Values};

// TODO: Make this a threadsafe queue so we can compile functions in parallel
type FunctionQueue = Vec<(ast::FuncId, IrFunctionId)>;

pub(super) struct FunctionContext<'a> {
    definitions: HashMap<LocalId, Values>,
    function_builder: FunctionBuilder<'a>,
    shared_context: &'a SharedContext,
}

/// Shared context for all functions during ssa codegen
pub(super) struct SharedContext {
    functions: RwLock<HashMap<FuncId, IrFunctionId>>,
    function_queue: Mutex<FunctionQueue>,
    pub(super) program: Program,
}

impl<'a> FunctionContext<'a> {
    pub(super) fn new(
        parameters: &Parameters,
        shared_context: &'a SharedContext,
        shared_builder_context: &'a SharedBuilderContext,
    ) -> Self {
        let mut this = Self {
            definitions: HashMap::new(),
            function_builder: FunctionBuilder::new(shared_builder_context),
            shared_context,
        };
        this.add_parameters_to_scope(parameters);
        this
    }

    pub(super) fn new_function(&mut self, parameters: &Parameters) {
        self.definitions.clear();
        self.function_builder.new_function();
        self.add_parameters_to_scope(parameters);
    }

    /// Add each parameter to the current scope, and return the list of parameter types.
    ///
    /// The returned parameter type list will be flattened, so any struct parameters will
    /// be returned as one entry for each field (recursively).
    fn add_parameters_to_scope(&mut self, parameters: &Parameters) {
        for (id, _, _, typ) in parameters {
            self.add_parameter_to_scope(*id, typ);
        }
    }

    /// Adds a "single" parameter to scope.
    ///
    /// Single is in quotes here because in the case of tuple parameters, the tuple is flattened
    /// into a new parameter for each field recursively.
    fn add_parameter_to_scope(&mut self, parameter_id: LocalId, parameter_type: &ast::Type) {
        // Add a separate parameter for each field type in 'parameter_type'
        let parameter_value = self
            .map_type(parameter_type, |this, typ| this.function_builder.add_parameter(typ).into());

        self.definitions.insert(parameter_id, parameter_value);
    }

    /// Maps the given type to a Tree of the result type.
    ///
    /// This can be used to (for example) flatten a tuple type, creating
    /// and returning a new parameter for each field type.
    pub(super) fn map_type<T>(
        &mut self,
        typ: &ast::Type,
        mut f: impl FnMut(&mut Self, Type) -> T,
    ) -> Tree<T> {
        self.map_type_helper(typ, &mut f)
    }

    // This helper is needed because we need to take f by mutable reference,
    // otherwise we cannot move it multiple times each loop of vecmap.
    fn map_type_helper<T>(
        &mut self,
        typ: &ast::Type,
        f: &mut impl FnMut(&mut Self, Type) -> T,
    ) -> Tree<T> {
        match typ {
            ast::Type::Tuple(fields) => {
                Tree::Branch(vecmap(fields, |field| self.map_type_helper(field, f)))
            }
            other => Tree::Leaf(f(self, Self::convert_non_tuple_type(other))),
        }
    }

    pub(super) fn convert_non_tuple_type(typ: &ast::Type) -> Type {
        match typ {
            ast::Type::Field => Type::field(),
            ast::Type::Array(_, _) => Type::Reference,
            ast::Type::Integer(Signedness::Signed, bits) => Type::signed(*bits),
            ast::Type::Integer(Signedness::Unsigned, bits) => Type::unsigned(*bits),
            ast::Type::Bool => Type::unsigned(1),
            ast::Type::String(_) => Type::Reference,
            ast::Type::Unit => Type::Unit,
            ast::Type::Tuple(_) => panic!("convert_non_tuple_type called on a tuple: {typ}"),
            ast::Type::Function(_, _) => Type::Function,

            // How should we represent Vecs?
            // Are they a struct of array + length + capacity?
            // Or are they just references?
            ast::Type::Vec(_) => Type::Reference,
        }
    }
}

impl SharedContext {
    pub(super) fn new(program: Program) -> Self {
        Self { functions: Default::default(), function_queue: Default::default(), program }
    }

    pub(super) fn pop_next_function_in_queue(&self) -> Option<(ast::FuncId, IrFunctionId)> {
        self.function_queue.lock().expect("Failed to lock function_queue").pop()
    }
}
