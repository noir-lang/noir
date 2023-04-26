use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::{self, LocalId, Parameters};
use noirc_frontend::monomorphization::ast::{FuncId, Program};
use noirc_frontend::Signedness;

use crate::ssa_refactor::ir::instruction::BinaryOp;
use crate::ssa_refactor::ir::types::Type;
use crate::ssa_refactor::ir::value::ValueId;
use crate::ssa_refactor::ssa_builder::SharedBuilderContext;
use crate::ssa_refactor::{
    ir::function::FunctionId as IrFunctionId, ssa_builder::function_builder::FunctionBuilder,
};

use super::value::{Tree, Values};

// TODO: Make this a threadsafe queue so we can compile functions in parallel
type FunctionQueue = Vec<(ast::FuncId, IrFunctionId)>;

pub(super) struct FunctionContext<'a> {
    definitions: HashMap<LocalId, Values>,
    pub(super) builder: FunctionBuilder<'a>,
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
        function_name: String,
        parameters: &Parameters,
        shared_context: &'a SharedContext,
        shared_builder_context: &'a SharedBuilderContext,
    ) -> Self {
        let mut this = Self {
            definitions: HashMap::new(),
            builder: FunctionBuilder::new(function_name, shared_builder_context),
            shared_context,
        };
        this.add_parameters_to_scope(parameters);
        this
    }

    pub(super) fn new_function(&mut self, name: String, parameters: &Parameters) {
        self.definitions.clear();
        self.builder.new_function(name);
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
        let parameter_value =
            self.map_type(parameter_type, |this, typ| this.builder.add_parameter(typ).into());

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
        Self::map_type_helper(typ, &mut |typ| f(self, typ))
    }

    // This helper is needed because we need to take f by mutable reference,
    // otherwise we cannot move it multiple times each loop of vecmap.
    fn map_type_helper<T>(typ: &ast::Type, f: &mut impl FnMut(Type) -> T) -> Tree<T> {
        match typ {
            ast::Type::Tuple(fields) => {
                Tree::Branch(vecmap(fields, |field| Self::map_type_helper(field, f)))
            }
            other => Tree::Leaf(f(Self::convert_non_tuple_type(other))),
        }
    }

    /// Convert a monomorphized type to an SSA type, preserving the structure
    /// of any tuples within.
    pub(super) fn convert_type(typ: &ast::Type) -> Tree<Type> {
        // Do nothing in the closure here - map_type_helper already calls
        // convert_non_tuple_type internally.
        Self::map_type_helper(typ, &mut |x| x)
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

    /// Insert a unit constant into the current function if not already
    /// present, and return its value
    pub(super) fn unit_value(&mut self) -> Values {
        self.builder.numeric_constant(0u128.into(), Type::Unit).into()
    }

    /// Insert a binary instruction at the end of the current block.
    /// Converts the form of the binary instruction as necessary
    /// (e.g. swapping arguments, inserting a not) to represent it in the IR.
    /// For example, (a <= b) is represented as !(b < a)
    pub(super) fn insert_binary(
        &mut self,
        mut lhs: ValueId,
        operator: noirc_frontend::BinaryOpKind,
        mut rhs: ValueId,
    ) -> Values {
        let op = convert_operator(operator);

        if operator_requires_swapped_operands(operator) {
            std::mem::swap(&mut lhs, &mut rhs);
        }

        let mut result = self.builder.insert_binary(lhs, op, rhs);

        if operator_requires_not(operator) {
            result = self.builder.insert_not(result);
        }
        result.into()
    }
}

/// True if the given operator cannot be encoded directly and needs
/// to be represented as !(some other operator)
fn operator_requires_not(op: noirc_frontend::BinaryOpKind) -> bool {
    use noirc_frontend::BinaryOpKind::*;
    matches!(op, NotEqual | LessEqual | GreaterEqual)
}

/// True if the given operator cannot be encoded directly and needs
/// to have its lhs and rhs swapped to be represented with another operator.
/// Example: (a > b) needs to be represented as (b < a)
fn operator_requires_swapped_operands(op: noirc_frontend::BinaryOpKind) -> bool {
    use noirc_frontend::BinaryOpKind::*;
    matches!(op, Greater | LessEqual)
}

/// Converts the given operator to the appropriate BinaryOp.
/// Take care when using this to insert a binary instruction: this requires
/// checking operator_requires_not and operator_requires_swapped_operands
/// to represent the full operation correctly.
fn convert_operator(op: noirc_frontend::BinaryOpKind) -> BinaryOp {
    use noirc_frontend::BinaryOpKind;
    match op {
        BinaryOpKind::Add => BinaryOp::Add,
        BinaryOpKind::Subtract => BinaryOp::Sub,
        BinaryOpKind::Multiply => BinaryOp::Mul,
        BinaryOpKind::Divide => BinaryOp::Div,
        BinaryOpKind::Modulo => BinaryOp::Mod,
        BinaryOpKind::Equal => BinaryOp::Eq,
        BinaryOpKind::NotEqual => BinaryOp::Eq, // Requires not
        BinaryOpKind::Less => BinaryOp::Lt,
        BinaryOpKind::Greater => BinaryOp::Lt, // Requires operand swap
        BinaryOpKind::LessEqual => BinaryOp::Lt, // Requires operand swap and not
        BinaryOpKind::GreaterEqual => BinaryOp::Lt, // Requires not
        BinaryOpKind::And => BinaryOp::And,
        BinaryOpKind::Or => BinaryOp::Or,
        BinaryOpKind::Xor => BinaryOp::Xor,
        BinaryOpKind::ShiftRight => BinaryOp::Shr,
        BinaryOpKind::ShiftLeft => BinaryOp::Shl,
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
