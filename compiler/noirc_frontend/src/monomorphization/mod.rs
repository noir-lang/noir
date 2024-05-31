//! Coming after type checking, monomorphization is the last pass in Noir's frontend.
//! It accepts the type checked HIR as input and produces a monomorphized AST as output.
//! This file implements the pass itself, while the AST is defined in the ast module.
//!
//! Unlike the HIR, which is stored within the NodeInterner, the monomorphized AST is
//! self-contained and does not need an external context struct. As a result, the NodeInterner
//! can be safely discarded after monomorphization.
//!
//! The entry point to this pass is the `monomorphize` function which, starting from a given
//! function, will monomorphize the entire reachable program.
use crate::ast::{FunctionKind, IntegerBitSize, Signedness, UnaryOp, Visibility};
use crate::{
    debug::DebugInstrumenter,
    hir_def::{
        expr::*,
        function::{FuncMeta, FunctionSignature, Parameters},
        stmt::{HirAssignStatement, HirLValue, HirLetStatement, HirPattern, HirStatement},
        types,
    },
    node_interner::{self, DefinitionKind, NodeInterner, StmtId, TraitImplKind, TraitMethodId},
    token::FunctionAttribute,
    Type, TypeBinding, TypeBindings, TypeVariable, TypeVariableKind,
};
use acvm::{acir::AcirField, FieldElement};
use iter_extended::{btree_map, try_vecmap, vecmap};
use noirc_errors::Location;
use noirc_printable_type::PrintableType;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    unreachable,
};

use self::ast::InlineType;
use self::debug_types::DebugTypeTracker;
use self::{
    ast::{Definition, FuncId, Function, LocalId, Program},
    errors::MonomorphizationError,
};

pub mod ast;
mod debug;
pub mod debug_types;
pub mod errors;
pub mod printer;

struct LambdaContext {
    env_ident: ast::Ident,
    captures: Vec<HirCapturedVar>,
}

/// The context struct for the monomorphization pass.
///
/// This struct holds the FIFO queue of functions to monomorphize, which is added to
/// whenever a new (function, type) combination is encountered.
struct Monomorphizer<'interner> {
    /// Functions are keyed by their unique ID and expected type so that we can monomorphize
    /// a new version of the function for each type.
    /// We also key by any turbofish generics that are specified.
    /// This is necessary for a case where we may have a trait generic that can be instantiated
    /// outside of a function parameter or return value.
    ///
    /// Using nested HashMaps here lets us avoid cloning HirTypes when calling .get()
    functions: HashMap<node_interner::FuncId, HashMap<(HirType, Vec<HirType>), FuncId>>,

    /// Unlike functions, locals are only keyed by their unique ID because they are never
    /// duplicated during monomorphization. Doing so would allow them to be used polymorphically
    /// but would also cause them to be re-evaluated which is a performance trap that would
    /// confuse users.
    locals: HashMap<node_interner::DefinitionId, LocalId>,

    /// Queue of functions to monomorphize next each item in the queue is a tuple of:
    /// (old_id, new_monomorphized_id, any type bindings to apply, the trait method if old_id is from a trait impl)
    queue: VecDeque<(node_interner::FuncId, FuncId, TypeBindings, Option<TraitMethodId>)>,

    /// When a function finishes being monomorphized, the monomorphized ast::Function is
    /// stored here along with its FuncId.
    finished_functions: BTreeMap<FuncId, Function>,

    /// Used to reference existing definitions in the HIR
    interner: &'interner mut NodeInterner,

    lambda_envs_stack: Vec<LambdaContext>,

    next_local_id: u32,
    next_function_id: u32,

    is_range_loop: bool,

    return_location: Option<Location>,

    debug_type_tracker: DebugTypeTracker,
}

type HirType = crate::Type;

/// Starting from the given `main` function, monomorphize the entire program,
/// replacing all references to type variables and NamedGenerics with concrete
/// types, duplicating definitions as necessary to do so.
///
/// Instead of iterating over every function, this pass starts with the main function
/// and monomorphizes every function reachable from it via function calls and references.
/// Thus, if a function is not used in the program, it will not be monomorphized.
///
/// Note that there is no requirement on the `main` function that can be passed into
/// this function. Typically, this is the function named "main" in the source project,
/// but it can also be, for example, an arbitrary test function for running `nargo test`.
#[tracing::instrument(level = "trace", skip(main, interner))]
pub fn monomorphize(
    main: node_interner::FuncId,
    interner: &mut NodeInterner,
) -> Result<Program, MonomorphizationError> {
    monomorphize_debug(main, interner, &DebugInstrumenter::default())
}

pub fn monomorphize_debug(
    main: node_interner::FuncId,
    interner: &mut NodeInterner,
    debug_instrumenter: &DebugInstrumenter,
) -> Result<Program, MonomorphizationError> {
    let debug_type_tracker = DebugTypeTracker::build_from_debug_instrumenter(debug_instrumenter);
    let mut monomorphizer = Monomorphizer::new(interner, debug_type_tracker);
    let function_sig = monomorphizer.compile_main(main)?;

    while !monomorphizer.queue.is_empty() {
        let (next_fn_id, new_id, bindings, trait_method) = monomorphizer.queue.pop_front().unwrap();
        monomorphizer.locals.clear();

        perform_instantiation_bindings(&bindings);
        let impl_bindings = monomorphizer.perform_impl_bindings(trait_method, next_fn_id);
        monomorphizer.function(next_fn_id, new_id)?;
        undo_instantiation_bindings(impl_bindings);
        undo_instantiation_bindings(bindings);
    }

    let func_sigs = monomorphizer
        .finished_functions
        .iter()
        .flat_map(|(_, f)| {
            if f.inline_type.is_entry_point() || f.id == Program::main_id() {
                Some(f.func_sig.clone())
            } else {
                None
            }
        })
        .collect();

    let functions = vecmap(monomorphizer.finished_functions, |(_, f)| f);
    let FuncMeta { return_visibility, kind, .. } = monomorphizer.interner.function_meta(&main);

    let (debug_variables, debug_functions, debug_types) =
        monomorphizer.debug_type_tracker.extract_vars_and_types();
    let program = Program::new(
        functions,
        func_sigs,
        function_sig,
        monomorphizer.return_location,
        *return_visibility,
        *kind == FunctionKind::Recursive,
        debug_variables,
        debug_functions,
        debug_types,
    );
    Ok(program)
}

impl<'interner> Monomorphizer<'interner> {
    fn new(interner: &'interner mut NodeInterner, debug_type_tracker: DebugTypeTracker) -> Self {
        Monomorphizer {
            functions: HashMap::new(),
            locals: HashMap::new(),
            queue: VecDeque::new(),
            finished_functions: BTreeMap::new(),
            next_local_id: 0,
            next_function_id: 0,
            interner,
            lambda_envs_stack: Vec::new(),
            is_range_loop: false,
            return_location: None,
            debug_type_tracker,
        }
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = self.next_local_id;
        self.next_local_id += 1;
        LocalId(id)
    }

    fn next_function_id(&mut self) -> ast::FuncId {
        let id = self.next_function_id;
        self.next_function_id += 1;
        ast::FuncId(id)
    }

    fn lookup_local(&mut self, id: node_interner::DefinitionId) -> Option<Definition> {
        self.locals.get(&id).copied().map(Definition::Local)
    }

    fn lookup_function(
        &mut self,
        id: node_interner::FuncId,
        expr_id: node_interner::ExprId,
        typ: &HirType,
        turbofish_generics: Vec<HirType>,
        trait_method: Option<TraitMethodId>,
    ) -> Definition {
        let typ = typ.follow_bindings();
        match self
            .functions
            .get(&id)
            .and_then(|inner_map| inner_map.get(&(typ.clone(), turbofish_generics.clone())))
        {
            Some(id) => Definition::Function(*id),
            None => {
                // Function has not been monomorphized yet
                let attributes = self.interner.function_attributes(&id);
                match self.interner.function_meta(&id).kind {
                    FunctionKind::LowLevel => {
                        let attribute = attributes.function.clone().expect("all low level functions must contain a function attribute which contains the opcode which it links to");
                        let opcode = attribute.foreign().expect(
                            "ice: function marked as foreign, but attribute kind does not match this",
                        );
                        Definition::LowLevel(opcode)
                    }
                    FunctionKind::Builtin => {
                        let attribute = attributes.function.clone().expect("all low level functions must contain a function  attribute which contains the opcode which it links to");
                        let opcode = attribute.builtin().expect(
                            "ice: function marked as builtin, but attribute kind does not match this",
                        );
                        Definition::Builtin(opcode)
                    }
                    FunctionKind::Normal => {
                        let id =
                            self.queue_function(id, expr_id, typ, turbofish_generics, trait_method);
                        Definition::Function(id)
                    }
                    FunctionKind::Oracle => {
                        let attr = attributes
                            .function
                            .clone()
                            .expect("Oracle function must have an oracle attribute");

                        match attr {
                            FunctionAttribute::Oracle(name) => Definition::Oracle(name),
                            _ => unreachable!("Oracle function must have an oracle attribute"),
                        }
                    }
                    FunctionKind::Recursive => {
                        unreachable!("Only main can be specified as recursive, which should already be checked");
                    }
                }
            }
        }
    }

    fn define_local(&mut self, id: node_interner::DefinitionId, new_id: LocalId) {
        self.locals.insert(id, new_id);
    }

    /// Prerequisite: typ = typ.follow_bindings()
    fn define_function(
        &mut self,
        id: node_interner::FuncId,
        typ: HirType,
        turbofish_generics: Vec<HirType>,
        new_id: FuncId,
    ) {
        self.functions.entry(id).or_default().insert((typ, turbofish_generics), new_id);
    }

    fn compile_main(
        &mut self,
        main_id: node_interner::FuncId,
    ) -> Result<FunctionSignature, MonomorphizationError> {
        let new_main_id = self.next_function_id();
        assert_eq!(new_main_id, Program::main_id());
        self.function(main_id, new_main_id)?;
        self.return_location =
            self.interner.function(&main_id).block(self.interner).statements().last().and_then(
                |x| match self.interner.statement(x) {
                    HirStatement::Expression(id) => Some(self.interner.id_location(id)),
                    _ => None,
                },
            );
        let main_meta = self.interner.function_meta(&main_id);
        Ok(main_meta.function_signature())
    }

    fn function(
        &mut self,
        f: node_interner::FuncId,
        id: FuncId,
    ) -> Result<(), MonomorphizationError> {
        if let Some((self_type, trait_id)) = self.interner.get_function_trait(&f) {
            let the_trait = self.interner.get_trait(trait_id);
            the_trait.self_type_typevar.force_bind(self_type);
        }

        let meta = self.interner.function_meta(&f).clone();
        let mut func_sig = meta.function_signature();
        // Follow the bindings of the function signature for entry points
        // which are not `main` such as foldable functions.
        for param in func_sig.0.iter_mut() {
            param.1 = param.1.follow_bindings();
        }
        func_sig.1 = func_sig.1.map(|return_type| return_type.follow_bindings());

        let modifiers = self.interner.function_modifiers(&f);
        let name = self.interner.function_name(&f).to_owned();

        let body_expr_id = self.interner.function(&f).as_expr();
        let body_return_type = self.interner.id_type(body_expr_id);
        let return_type = match meta.return_type() {
            Type::TraitAsType(..) => &body_return_type,
            other => other,
        };

        let return_type = Self::convert_type(return_type, meta.location)?;
        let unconstrained = modifiers.is_unconstrained;

        let attributes = self.interner.function_attributes(&f);
        let inline_type = InlineType::from(attributes);

        let parameters = self.parameters(&meta.parameters)?;
        let body = self.expr(body_expr_id)?;
        let function = ast::Function {
            id,
            name,
            parameters,
            body,
            return_type,
            unconstrained,
            inline_type,
            func_sig,
        };

        self.push_function(id, function);
        Ok(())
    }

    fn push_function(&mut self, id: FuncId, function: ast::Function) {
        let existing = self.finished_functions.insert(id, function);
        assert!(existing.is_none());
    }

    /// Monomorphize each parameter, expanding tuple/struct patterns into multiple parameters
    /// and binding any generic types found.
    fn parameters(
        &mut self,
        params: &Parameters,
    ) -> Result<Vec<(ast::LocalId, bool, String, ast::Type)>, MonomorphizationError> {
        let mut new_params = Vec::with_capacity(params.len());
        for (parameter, typ, _) in &params.0 {
            self.parameter(parameter, typ, &mut new_params)?;
        }
        Ok(new_params)
    }

    fn parameter(
        &mut self,
        param: &HirPattern,
        typ: &HirType,
        new_params: &mut Vec<(ast::LocalId, bool, String, ast::Type)>,
    ) -> Result<(), MonomorphizationError> {
        match param {
            HirPattern::Identifier(ident) => {
                let new_id = self.next_local_id();
                let definition = self.interner.definition(ident.id);
                let name = definition.name.clone();
                let typ = Self::convert_type(typ, ident.location)?;
                new_params.push((new_id, definition.mutable, name, typ));
                self.define_local(ident.id, new_id);
            }
            HirPattern::Mutable(pattern, _) => self.parameter(pattern, typ, new_params)?,
            HirPattern::Tuple(fields, _) => {
                let tuple_field_types = unwrap_tuple_type(typ);

                for (field, typ) in fields.iter().zip(tuple_field_types) {
                    self.parameter(field, &typ, new_params)?;
                }
            }
            HirPattern::Struct(_, fields, _) => {
                let struct_field_types = unwrap_struct_type(typ);
                assert_eq!(struct_field_types.len(), fields.len());

                let mut fields =
                    btree_map(fields, |(name, field)| (name.0.contents.clone(), field));

                // Iterate over `struct_field_types` since `unwrap_struct_type` will always
                // return the fields in the order defined by the struct type.
                for (field_name, field_type) in struct_field_types {
                    let field = fields.remove(&field_name).unwrap_or_else(|| {
                        unreachable!("Expected a field named '{field_name}' in the struct pattern")
                    });

                    self.parameter(field, &field_type, new_params)?;
                }
            }
        }
        Ok(())
    }

    fn expr(
        &mut self,
        expr: node_interner::ExprId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        use ast::Expression::Literal;
        use ast::Literal::*;

        let expr = match self.interner.expression(&expr) {
            HirExpression::Ident(ident, generics) => self.ident(ident, expr, generics)?,
            HirExpression::Literal(HirLiteral::Str(contents)) => Literal(Str(contents)),
            HirExpression::Literal(HirLiteral::FmtStr(contents, idents)) => {
                let fields = try_vecmap(idents, |ident| self.expr(ident))?;
                Literal(FmtStr(
                    contents,
                    fields.len() as u64,
                    Box::new(ast::Expression::Tuple(fields)),
                ))
            }
            HirExpression::Literal(HirLiteral::Bool(value)) => Literal(Bool(value)),
            HirExpression::Literal(HirLiteral::Integer(value, sign)) => {
                let location = self.interner.id_location(expr);
                let typ = Self::convert_type(&self.interner.id_type(expr), location)?;

                if sign {
                    match typ {
                        ast::Type::Field => Literal(Integer(-value, typ, location)),
                        ast::Type::Integer(_, bit_size) => {
                            let bit_size: u32 = bit_size.into();
                            let base = 1_u128 << bit_size;
                            Literal(Integer(FieldElement::from(base) - value, typ, location))
                        }
                        _ => unreachable!("Integer literal must be numeric"),
                    }
                } else {
                    Literal(Integer(value, typ, location))
                }
            }
            HirExpression::Literal(HirLiteral::Array(array)) => match array {
                HirArrayLiteral::Standard(array) => self.standard_array(expr, array, false)?,
                HirArrayLiteral::Repeated { repeated_element, length } => {
                    self.repeated_array(expr, repeated_element, length, false)?
                }
            },
            HirExpression::Literal(HirLiteral::Slice(array)) => match array {
                HirArrayLiteral::Standard(array) => self.standard_array(expr, array, true)?,
                HirArrayLiteral::Repeated { repeated_element, length } => {
                    self.repeated_array(expr, repeated_element, length, true)?
                }
            },
            HirExpression::Literal(HirLiteral::Unit) => ast::Expression::Block(vec![]),
            HirExpression::Block(block) => self.block(block.statements)?,

            HirExpression::Prefix(prefix) => {
                let location = self.interner.expr_location(&expr);
                ast::Expression::Unary(ast::Unary {
                    operator: prefix.operator,
                    rhs: Box::new(self.expr(prefix.rhs)?),
                    result_type: Self::convert_type(&self.interner.id_type(expr), location)?,
                    location,
                })
            }

            HirExpression::Infix(infix) => {
                let lhs = self.expr(infix.lhs)?;
                let rhs = self.expr(infix.rhs)?;
                let operator = infix.operator.kind;
                let location = self.interner.expr_location(&expr);
                if self.interner.get_selected_impl_for_expression(expr).is_some() {
                    // If an impl was selected for this infix operator, replace it
                    // with a method call to the appropriate trait impl method.
                    let lhs_type = self.interner.id_type(infix.lhs);
                    let args = vec![lhs_type.clone(), lhs_type];

                    // If this is a comparison operator, the result is a boolean but
                    // the actual method call returns an Ordering
                    use crate::ast::BinaryOpKind::*;
                    let ret = if matches!(operator, Less | LessEqual | Greater | GreaterEqual) {
                        self.interner.ordering_type()
                    } else {
                        self.interner.id_type(expr)
                    };

                    let env = Box::new(Type::Unit);
                    let function_type = Type::Function(args, Box::new(ret.clone()), env);

                    let method = infix.trait_method_id;
                    let func = self.resolve_trait_method_reference(expr, function_type, method)?;
                    self.create_operator_impl_call(func, lhs, infix.operator, rhs, ret, location)?
                } else {
                    let lhs = Box::new(lhs);
                    let rhs = Box::new(rhs);
                    ast::Expression::Binary(ast::Binary { lhs, rhs, operator, location })
                }
            }

            HirExpression::Index(index) => self.index(expr, index)?,

            HirExpression::MemberAccess(access) => {
                let field_index = self.interner.get_field_index(expr);
                let expr = Box::new(self.expr(access.lhs)?);
                ast::Expression::ExtractTupleField(expr, field_index)
            }

            HirExpression::Call(call) => self.function_call(call, expr)?,

            HirExpression::Cast(cast) => {
                let location = self.interner.expr_location(&expr);
                let typ = Self::convert_type(&cast.r#type, location)?;
                let lhs = Box::new(self.expr(cast.lhs)?);
                ast::Expression::Cast(ast::Cast { lhs, r#type: typ, location })
            }

            HirExpression::If(if_expr) => {
                let condition = Box::new(self.expr(if_expr.condition)?);
                let consequence = Box::new(self.expr(if_expr.consequence)?);
                let else_ =
                    if_expr.alternative.map(|alt| self.expr(alt)).transpose()?.map(Box::new);

                let location = self.interner.expr_location(&expr);
                let typ = Self::convert_type(&self.interner.id_type(expr), location)?;
                ast::Expression::If(ast::If { condition, consequence, alternative: else_, typ })
            }

            HirExpression::Tuple(fields) => {
                let fields = try_vecmap(fields, |id| self.expr(id))?;
                ast::Expression::Tuple(fields)
            }
            HirExpression::Constructor(constructor) => self.constructor(constructor, expr)?,

            HirExpression::Lambda(lambda) => self.lambda(lambda, expr)?,

            HirExpression::MethodCall(hir_method_call) => {
                unreachable!("Encountered HirExpression::MethodCall during monomorphization {hir_method_call:?}")
            }
            HirExpression::Error => unreachable!("Encountered Error node during monomorphization"),
            HirExpression::Quote(_) => unreachable!("quote expression remaining in runtime code"),
            HirExpression::Unquote(_) => {
                unreachable!("unquote expression remaining in runtime code")
            }
            HirExpression::Comptime(_) => {
                unreachable!("comptime expression remaining in runtime code")
            }
        };

        Ok(expr)
    }

    fn standard_array(
        &mut self,
        array: node_interner::ExprId,
        array_elements: Vec<node_interner::ExprId>,
        is_slice: bool,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let location = self.interner.expr_location(&array);
        let typ = Self::convert_type(&self.interner.id_type(array), location)?;
        let contents = try_vecmap(array_elements, |id| self.expr(id))?;
        if is_slice {
            Ok(ast::Expression::Literal(ast::Literal::Slice(ast::ArrayLiteral { contents, typ })))
        } else {
            Ok(ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral { contents, typ })))
        }
    }

    fn repeated_array(
        &mut self,
        array: node_interner::ExprId,
        repeated_element: node_interner::ExprId,
        length: HirType,
        is_slice: bool,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let location = self.interner.expr_location(&array);
        let typ = Self::convert_type(&self.interner.id_type(array), location)?;

        let length = length.evaluate_to_u64().ok_or_else(|| {
            let location = self.interner.expr_location(&array);
            MonomorphizationError::UnknownArrayLength { location }
        })?;

        let contents = try_vecmap(0..length, |_| self.expr(repeated_element))?;
        if is_slice {
            Ok(ast::Expression::Literal(ast::Literal::Slice(ast::ArrayLiteral { contents, typ })))
        } else {
            Ok(ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral { contents, typ })))
        }
    }

    fn index(
        &mut self,
        id: node_interner::ExprId,
        index: HirIndexExpression,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let location = self.interner.expr_location(&id);
        let element_type = Self::convert_type(&self.interner.id_type(id), location)?;

        let collection = Box::new(self.expr(index.collection)?);
        let index = Box::new(self.expr(index.index)?);
        let location = self.interner.expr_location(&id);
        Ok(ast::Expression::Index(ast::Index { collection, index, element_type, location }))
    }

    fn statement(&mut self, id: StmtId) -> Result<ast::Expression, MonomorphizationError> {
        match self.interner.statement(&id) {
            HirStatement::Let(let_statement) => self.let_statement(let_statement),
            HirStatement::Constrain(constrain) => {
                let expr = self.expr(constrain.0)?;
                let location = self.interner.expr_location(&constrain.0);
                let assert_message = constrain
                    .2
                    .map(|assert_msg_expr| {
                        self.expr(assert_msg_expr).map(|expr| {
                            (expr, self.interner.id_type(assert_msg_expr).follow_bindings())
                        })
                    })
                    .transpose()?
                    .map(Box::new);
                Ok(ast::Expression::Constrain(Box::new(expr), location, assert_message))
            }
            HirStatement::Assign(assign) => self.assign(assign),
            HirStatement::For(for_loop) => {
                self.is_range_loop = true;
                let start = self.expr(for_loop.start_range)?;
                let end = self.expr(for_loop.end_range)?;
                self.is_range_loop = false;
                let index_variable = self.next_local_id();
                self.define_local(for_loop.identifier.id, index_variable);

                let block = Box::new(self.expr(for_loop.block)?);
                let index_location = for_loop.identifier.location;
                let index_type = self.interner.id_type(for_loop.start_range);
                let index_type = Self::convert_type(&index_type, index_location)?;

                Ok(ast::Expression::For(ast::For {
                    index_variable,
                    index_name: self.interner.definition_name(for_loop.identifier.id).to_owned(),
                    index_type,
                    start_range: Box::new(start),
                    end_range: Box::new(end),
                    start_range_location: self.interner.expr_location(&for_loop.start_range),
                    end_range_location: self.interner.expr_location(&for_loop.end_range),
                    block,
                }))
            }
            HirStatement::Expression(expr) => self.expr(expr),
            HirStatement::Semi(expr) => {
                self.expr(expr).map(|expr| ast::Expression::Semi(Box::new(expr)))
            }
            HirStatement::Break => Ok(ast::Expression::Break),
            HirStatement::Continue => Ok(ast::Expression::Continue),
            HirStatement::Error => unreachable!(),

            // All `comptime` statements & expressions should be removed before runtime.
            HirStatement::Comptime(_) => unreachable!("comptime statement in runtime code"),
        }
    }

    fn let_statement(
        &mut self,
        let_statement: HirLetStatement,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let expr = self.expr(let_statement.expression)?;
        let expected_type = self.interner.id_type(let_statement.expression);
        self.unpack_pattern(let_statement.pattern, expr, &expected_type)
    }

    fn constructor(
        &mut self,
        constructor: HirConstructorExpression,
        id: node_interner::ExprId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let typ = self.interner.id_type(id);
        let field_types = unwrap_struct_type(&typ);

        let field_type_map = btree_map(&field_types, |x| x.clone());

        // Create let bindings for each field value first to preserve evaluation order before
        // they are reordered and packed into the resulting tuple
        let mut field_vars = BTreeMap::new();
        let mut new_exprs = Vec::with_capacity(constructor.fields.len());

        for (field_name, expr_id) in constructor.fields {
            let new_id = self.next_local_id();
            let field_type = field_type_map.get(&field_name.0.contents).unwrap();
            let location = self.interner.expr_location(&expr_id);
            let typ = Self::convert_type(field_type, location)?;

            field_vars.insert(field_name.0.contents.clone(), (new_id, typ));
            let expression = Box::new(self.expr(expr_id)?);

            new_exprs.push(ast::Expression::Let(ast::Let {
                id: new_id,
                mutable: false,
                name: field_name.0.contents,
                expression,
            }));
        }

        // We must ensure the tuple created from the variables here matches the order
        // of the fields as defined in the type. To do this, we iterate over field_types,
        // rather than field_type_map which is a sorted BTreeMap.
        let field_idents = vecmap(field_types, |(name, _)| {
            let (id, typ) = field_vars.remove(&name).unwrap_or_else(|| {
                unreachable!("Expected field {name} to be present in constructor for {typ}")
            });

            let definition = Definition::Local(id);
            let mutable = false;
            ast::Expression::Ident(ast::Ident { definition, mutable, location: None, name, typ })
        });

        // Finally we can return the created Tuple from the new block
        new_exprs.push(ast::Expression::Tuple(field_idents));
        Ok(ast::Expression::Block(new_exprs))
    }

    fn block(
        &mut self,
        statement_ids: Vec<StmtId>,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let stmts = try_vecmap(statement_ids, |id| self.statement(id));
        stmts.map(ast::Expression::Block)
    }

    fn unpack_pattern(
        &mut self,
        pattern: HirPattern,
        value: ast::Expression,
        typ: &HirType,
    ) -> Result<ast::Expression, MonomorphizationError> {
        match pattern {
            HirPattern::Identifier(ident) => {
                let new_id = self.next_local_id();
                self.define_local(ident.id, new_id);
                let definition = self.interner.definition(ident.id);

                Ok(ast::Expression::Let(ast::Let {
                    id: new_id,
                    mutable: definition.mutable,
                    name: definition.name.clone(),
                    expression: Box::new(value),
                }))
            }
            HirPattern::Mutable(pattern, _) => self.unpack_pattern(*pattern, value, typ),
            HirPattern::Tuple(patterns, _) => {
                let fields = unwrap_tuple_type(typ);
                self.unpack_tuple_pattern(value, patterns.into_iter().zip(fields))
            }
            HirPattern::Struct(_, patterns, _) => {
                let fields = unwrap_struct_type(typ);
                assert_eq!(patterns.len(), fields.len());

                let mut patterns =
                    btree_map(patterns, |(name, pattern)| (name.0.contents, pattern));

                // We iterate through the type's fields to match the order defined in the struct type
                let patterns_iter = fields.into_iter().map(|(field_name, field_type)| {
                    let pattern = patterns.remove(&field_name).unwrap();
                    (pattern, field_type)
                });

                self.unpack_tuple_pattern(value, patterns_iter)
            }
        }
    }

    fn unpack_tuple_pattern(
        &mut self,
        value: ast::Expression,
        fields: impl Iterator<Item = (HirPattern, HirType)>,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let fresh_id = self.next_local_id();

        let mut definitions = vec![ast::Expression::Let(ast::Let {
            id: fresh_id,
            mutable: false,
            name: "_".into(),
            expression: Box::new(value),
        })];

        for (i, (field_pattern, field_type)) in fields.into_iter().enumerate() {
            let location = field_pattern.location();
            let mutable = false;
            let definition = Definition::Local(fresh_id);
            let name = i.to_string();
            let typ = Self::convert_type(&field_type, location)?;

            let location = Some(location);
            let new_rhs =
                ast::Expression::Ident(ast::Ident { location, mutable, definition, name, typ });

            let new_rhs = ast::Expression::ExtractTupleField(Box::new(new_rhs), i);
            let new_expr = self.unpack_pattern(field_pattern, new_rhs, &field_type)?;
            definitions.push(new_expr);
        }

        Ok(ast::Expression::Block(definitions))
    }

    /// Find a captured variable in the innermost closure, and construct an expression
    fn lookup_captured_expr(&mut self, id: node_interner::DefinitionId) -> Option<ast::Expression> {
        let ctx = self.lambda_envs_stack.last()?;
        ctx.captures.iter().position(|capture| capture.ident.id == id).map(|index| {
            ast::Expression::ExtractTupleField(
                Box::new(ast::Expression::Ident(ctx.env_ident.clone())),
                index,
            )
        })
    }

    /// Find a captured variable in the innermost closure construct a LValue
    fn lookup_captured_lvalue(&mut self, id: node_interner::DefinitionId) -> Option<ast::LValue> {
        let ctx = self.lambda_envs_stack.last()?;
        ctx.captures.iter().position(|capture| capture.ident.id == id).map(|index| {
            ast::LValue::MemberAccess {
                object: Box::new(ast::LValue::Ident(ctx.env_ident.clone())),
                field_index: index,
            }
        })
    }

    /// A local (ie non-function) ident only
    fn local_ident(
        &mut self,
        ident: &HirIdent,
    ) -> Result<Option<ast::Ident>, MonomorphizationError> {
        let definition = self.interner.definition(ident.id);
        let name = definition.name.clone();
        let mutable = definition.mutable;

        let Some(definition) = self.lookup_local(ident.id) else {
            return Ok(None);
        };

        let typ = Self::convert_type(&self.interner.definition_type(ident.id), ident.location)?;
        Ok(Some(ast::Ident { location: Some(ident.location), mutable, definition, name, typ }))
    }

    fn ident(
        &mut self,
        ident: HirIdent,
        expr_id: node_interner::ExprId,
        generics: Option<Vec<HirType>>,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let typ = self.interner.id_type(expr_id);

        if let ImplKind::TraitMethod(method, _, _) = ident.impl_kind {
            return self.resolve_trait_method_reference(expr_id, typ, method);
        }

        let definition = self.interner.definition(ident.id);
        let ident = match &definition.kind {
            DefinitionKind::Function(func_id) => {
                let mutable = definition.mutable;
                let location = Some(ident.location);
                let name = definition.name.clone();
                let definition = self.lookup_function(
                    *func_id,
                    expr_id,
                    &typ,
                    generics.unwrap_or_default(),
                    None,
                );
                let typ = Self::convert_type(&typ, ident.location)?;
                let ident = ast::Ident { location, mutable, definition, name, typ: typ.clone() };
                let ident_expression = ast::Expression::Ident(ident);
                if self.is_function_closure_type(&typ) {
                    ast::Expression::Tuple(vec![
                        ast::Expression::ExtractTupleField(
                            Box::new(ident_expression.clone()),
                            0usize,
                        ),
                        ast::Expression::ExtractTupleField(Box::new(ident_expression), 1usize),
                    ])
                } else {
                    ident_expression
                }
            }
            DefinitionKind::Global(global_id) => {
                let Some(let_) = self.interner.get_global_let_statement(*global_id) else {
                    unreachable!(
                        "Globals should have a corresponding let statement by monomorphization"
                    )
                };
                self.expr(let_.expression)?
            }
            DefinitionKind::Local(_) => match self.lookup_captured_expr(ident.id) {
                Some(expr) => expr,
                None => {
                    let ident = self.local_ident(&ident)?.unwrap();
                    ast::Expression::Ident(ident)
                }
            },
            DefinitionKind::GenericType(type_variable) => {
                let value = match &*type_variable.borrow() {
                    TypeBinding::Unbound(_) => {
                        unreachable!("Unbound type variable used in expression")
                    }
                    TypeBinding::Bound(binding) => binding.evaluate_to_u64().unwrap_or_else(|| {
                        panic!("Non-numeric type variable used in expression expecting a value")
                    }),
                };

                let value = FieldElement::from(value as u128);
                let location = self.interner.id_location(expr_id);
                let typ = Self::convert_type(&typ, ident.location)?;
                ast::Expression::Literal(ast::Literal::Integer(value, typ, location))
            }
        };

        Ok(ident)
    }

    /// Convert a non-tuple/struct type to a monomorphized type
    fn convert_type(typ: &HirType, location: Location) -> Result<ast::Type, MonomorphizationError> {
        Ok(match typ {
            HirType::FieldElement => ast::Type::Field,
            HirType::Integer(sign, bits) => ast::Type::Integer(*sign, *bits),
            HirType::Bool => ast::Type::Bool,
            HirType::String(size) => ast::Type::String(size.evaluate_to_u64().unwrap_or(0)),
            HirType::FmtString(size, fields) => {
                let size = size.evaluate_to_u64().unwrap_or(0);
                let fields = Box::new(Self::convert_type(fields.as_ref(), location)?);
                ast::Type::FmtString(size, fields)
            }
            HirType::Unit => ast::Type::Unit,
            HirType::Array(length, element) => {
                let element = Box::new(Self::convert_type(element.as_ref(), location)?);
                let length = match length.evaluate_to_u64() {
                    Some(length) => length,
                    None => return Err(MonomorphizationError::TypeAnnotationsNeeded { location }),
                };
                ast::Type::Array(length, element)
            }
            HirType::Slice(element) => {
                let element = Box::new(Self::convert_type(element.as_ref(), location)?);
                ast::Type::Slice(element)
            }
            HirType::TraitAsType(..) => {
                unreachable!("All TraitAsType should be replaced before calling convert_type");
            }
            HirType::NamedGeneric(binding, _) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return Self::convert_type(binding, location);
                }

                // Default any remaining unbound type variables.
                // This should only happen if the variable in question is unused
                // and within a larger generic type.
                binding.bind(HirType::default_int_or_field_type());
                ast::Type::Field
            }

            HirType::TypeVariable(binding, kind) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return Self::convert_type(binding, location);
                }

                // Default any remaining unbound type variables.
                // This should only happen if the variable in question is unused
                // and within a larger generic type.
                let default = match kind.default_type() {
                    Some(typ) => typ,
                    None => return Err(MonomorphizationError::TypeAnnotationsNeeded { location }),
                };

                let monomorphized_default = Self::convert_type(&default, location)?;
                binding.bind(default);
                monomorphized_default
            }

            HirType::Struct(def, args) => {
                let fields = def.borrow().get_fields(args);
                let fields = try_vecmap(fields, |(_, field)| Self::convert_type(&field, location))?;
                ast::Type::Tuple(fields)
            }

            HirType::Alias(def, args) => {
                Self::convert_type(&def.borrow().get_type(args), location)?
            }

            HirType::Tuple(fields) => {
                let fields = try_vecmap(fields, |x| Self::convert_type(x, location))?;
                ast::Type::Tuple(fields)
            }

            HirType::Function(args, ret, env) => {
                let args = try_vecmap(args, |x| Self::convert_type(x, location))?;
                let ret = Box::new(Self::convert_type(ret, location)?);
                let env = Self::convert_type(env, location)?;
                match &env {
                    ast::Type::Unit => ast::Type::Function(args, ret, Box::new(env)),
                    ast::Type::Tuple(_elements) => ast::Type::Tuple(vec![
                        env.clone(),
                        ast::Type::Function(args, ret, Box::new(env)),
                    ]),
                    _ => {
                        unreachable!(
                            "internal Type::Function env should be either a Unit or a Tuple, not {env}"
                        )
                    }
                }
            }

            HirType::MutableReference(element) => {
                let element = Self::convert_type(element, location)?;
                ast::Type::MutableReference(Box::new(element))
            }

            HirType::Forall(_, _) | HirType::Constant(_) | HirType::Error => {
                unreachable!("Unexpected type {} found", typ)
            }
            HirType::Code => unreachable!("Tried to translate Code type into runtime code"),
        })
    }

    fn is_function_closure(&self, t: ast::Type) -> bool {
        if self.is_function_closure_type(&t) {
            true
        } else if let ast::Type::Tuple(elements) = t {
            if elements.len() == 2 {
                matches!(elements[1], ast::Type::Function(_, _, _))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_function_closure_type(&self, t: &ast::Type) -> bool {
        if let ast::Type::Function(_, _, env) = t {
            let e = (*env).clone();
            matches!(*e, ast::Type::Tuple(_captures))
        } else {
            false
        }
    }

    fn resolve_trait_method_reference(
        &mut self,
        expr_id: node_interner::ExprId,
        function_type: HirType,
        method: TraitMethodId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let trait_impl = self
            .interner
            .get_selected_impl_for_expression(expr_id)
            .expect("ICE: missing trait impl - should be caught during type checking");

        let func_id = match trait_impl {
            node_interner::TraitImplKind::Normal(impl_id) => {
                self.interner.get_trait_implementation(impl_id).borrow().methods
                    [method.method_index]
            }
            node_interner::TraitImplKind::Assumed { object_type, trait_generics } => {
                match self.interner.lookup_trait_implementation(
                    &object_type,
                    method.trait_id,
                    &trait_generics,
                ) {
                    Ok(TraitImplKind::Normal(impl_id)) => {
                        self.interner.get_trait_implementation(impl_id).borrow().methods
                            [method.method_index]
                    }
                    Ok(TraitImplKind::Assumed { .. }) => unreachable!(
                        "There should be no remaining Assumed impls during monomorphization"
                    ),
                    Err(constraints) => {
                        let failed_constraints = vecmap(constraints, |constraint| {
                            let id = constraint.trait_id;
                            let mut name = self.interner.get_trait(id).name.to_string();
                            if !constraint.trait_generics.is_empty() {
                                let types =
                                    vecmap(&constraint.trait_generics, |t| format!("{t:?}"));
                                name += &format!("<{}>", types.join(", "));
                            }
                            format!("  {}: {name}", constraint.typ)
                        })
                        .join("\n");

                        unreachable!("Failed to find trait impl during monomorphization. The failed constraint(s) are:\n{failed_constraints}")
                    }
                }
            }
        };

        let func_id =
            match self.lookup_function(func_id, expr_id, &function_type, vec![], Some(method)) {
                Definition::Function(func_id) => func_id,
                _ => unreachable!(),
            };

        let the_trait = self.interner.get_trait(method.trait_id);
        let location = self.interner.expr_location(&expr_id);

        Ok(ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(func_id),
            mutable: false,
            location: None,
            name: the_trait.methods[method.method_index].name.0.contents.clone(),
            typ: Self::convert_type(&function_type, location)?,
        }))
    }

    fn function_call(
        &mut self,
        call: HirCallExpression,
        id: node_interner::ExprId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let original_func = Box::new(self.expr(call.func)?);
        let mut arguments = try_vecmap(&call.arguments, |id| self.expr(*id))?;
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));

        self.patch_debug_instrumentation_call(&call, &mut arguments)?;

        let return_type = self.interner.id_type(id);
        let location = self.interner.expr_location(&id);
        let return_type = Self::convert_type(&return_type, location)?;

        let location = call.location;

        if let ast::Expression::Ident(ident) = original_func.as_ref() {
            if let Definition::Oracle(name) = &ident.definition {
                if name.as_str() == "print" {
                    // Oracle calls are required to be wrapped in an unconstrained function
                    // The first argument to the `print` oracle is a bool, indicating a newline to be inserted at the end of the input
                    // The second argument is expected to always be an ident
                    self.append_printable_type_info(&hir_arguments[1], &mut arguments);
                }
            }
        }

        let mut block_expressions = vec![];
        let func_type = self.interner.id_type(call.func);
        let func_type = Self::convert_type(&func_type, location)?;
        let is_closure = self.is_function_closure(func_type);

        let func = if is_closure {
            let local_id = self.next_local_id();

            // store the function in a temporary variable before calling it
            // this is needed for example if call.func is of the form `foo()()`
            // without this, we would translate it to `foo().1(foo().0)`
            let let_stmt = ast::Expression::Let(ast::Let {
                id: local_id,
                mutable: false,
                name: "tmp".to_string(),
                expression: Box::new(*original_func),
            });
            block_expressions.push(let_stmt);

            let extracted_func = ast::Expression::Ident(ast::Ident {
                location: None,
                definition: Definition::Local(local_id),
                mutable: false,
                name: "tmp".to_string(),
                typ: Self::convert_type(&self.interner.id_type(call.func), location)?,
            });

            let env_argument =
                ast::Expression::ExtractTupleField(Box::new(extracted_func.clone()), 0usize);
            arguments.insert(0, env_argument);

            Box::new(ast::Expression::ExtractTupleField(Box::new(extracted_func), 1usize))
        } else {
            original_func.clone()
        };

        let call = self
            .try_evaluate_call(&func, &id, &return_type)
            .unwrap_or(ast::Expression::Call(ast::Call { func, arguments, return_type, location }));

        if !block_expressions.is_empty() {
            block_expressions.push(call);
            Ok(ast::Expression::Block(block_expressions))
        } else {
            Ok(call)
        }
    }

    /// Adds a function argument that contains type metadata that is required to tell
    /// `println` how to convert values passed to an foreign call back to a human-readable string.
    /// The values passed to an foreign call will be a simple list of field elements,
    /// thus requiring extra metadata to correctly decode this list of elements.
    ///
    /// The Noir compiler has a `PrintableType` that handles encoding/decoding a list
    /// of field elements to/from JSON. The type metadata attached in this method
    /// is the serialized `PrintableType` for the argument passed to the function.
    /// The caller that is running a Noir program should then deserialize the `PrintableType`,
    /// and accurately decode the list of field elements passed to the foreign call.
    fn append_printable_type_info(
        &mut self,
        hir_argument: &HirExpression,
        arguments: &mut Vec<ast::Expression>,
    ) {
        match hir_argument {
            HirExpression::Ident(ident, _) => {
                let typ = self.interner.definition_type(ident.id);
                let typ: Type = typ.follow_bindings();
                let is_fmt_str = match typ {
                    // A format string has many different possible types that need to be handled.
                    // Loop over each element in the format string to fetch each type's relevant metadata
                    Type::FmtString(_, elements) => {
                        match *elements {
                            Type::Tuple(element_types) => {
                                for typ in element_types {
                                    Self::append_printable_type_info_inner(&typ, arguments);
                                }
                            }
                            _ => unreachable!(
                                "ICE: format string type should be a tuple but got a {elements}"
                            ),
                        }
                        true
                    }
                    _ => {
                        Self::append_printable_type_info_inner(&typ, arguments);
                        false
                    }
                };
                // The caller needs information as to whether it is handling a format string or a single type
                arguments.push(ast::Expression::Literal(ast::Literal::Bool(is_fmt_str)));
            }
            _ => unreachable!("logging expr {:?} is not supported", hir_argument),
        }
    }

    fn append_printable_type_info_inner(typ: &Type, arguments: &mut Vec<ast::Expression>) {
        // Disallow printing slices and mutable references for consistency,
        // since they cannot be passed from ACIR into Brillig
        if matches!(typ, HirType::MutableReference(_)) {
            unreachable!("println and format strings do not support mutable references.");
        }

        let printable_type: PrintableType = typ.into();
        let abi_as_string = serde_json::to_string(&printable_type)
            .expect("ICE: expected PrintableType to serialize");

        arguments.push(ast::Expression::Literal(ast::Literal::Str(abi_as_string)));
    }

    /// Try to evaluate certain builtin functions (currently only 'array_len' and field modulus methods)
    /// at their call site.
    /// NOTE: Evaluating at the call site means we cannot track aliased functions.
    ///       E.g. `let f = std::array::len; f(arr)` will fail to evaluate.
    ///       To fix this we need to evaluate on the identifier instead, which
    ///       requires us to evaluate to a Lambda value which isn't in noir yet.
    fn try_evaluate_call(
        &mut self,
        func: &ast::Expression,
        expr_id: &node_interner::ExprId,
        result_type: &ast::Type,
    ) -> Option<ast::Expression> {
        if let ast::Expression::Ident(ident) = func {
            if let Definition::Builtin(opcode) = &ident.definition {
                // TODO(#1736): Move this builtin to the SSA pass
                let location = self.interner.expr_location(expr_id);
                return match opcode.as_str() {
                    "modulus_num_bits" => {
                        let bits = (FieldElement::max_num_bits() as u128).into();
                        let typ =
                            ast::Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour);
                        Some(ast::Expression::Literal(ast::Literal::Integer(bits, typ, location)))
                    }
                    "zeroed" => {
                        let location = self.interner.expr_location(expr_id);
                        Some(self.zeroed_value_of_type(result_type, location))
                    }
                    "modulus_le_bits" => {
                        let bits = FieldElement::modulus().to_radix_le(2);
                        Some(self.modulus_array_literal(bits, IntegerBitSize::One, location))
                    }
                    "modulus_be_bits" => {
                        let bits = FieldElement::modulus().to_radix_be(2);
                        Some(self.modulus_array_literal(bits, IntegerBitSize::One, location))
                    }
                    "modulus_be_bytes" => {
                        let bytes = FieldElement::modulus().to_bytes_be();
                        Some(self.modulus_array_literal(bytes, IntegerBitSize::Eight, location))
                    }
                    "modulus_le_bytes" => {
                        let bytes = FieldElement::modulus().to_bytes_le();
                        Some(self.modulus_array_literal(bytes, IntegerBitSize::Eight, location))
                    }
                    _ => None,
                };
            }
        }
        None
    }

    fn modulus_array_literal(
        &self,
        bytes: Vec<u8>,
        arr_elem_bits: IntegerBitSize,
        location: Location,
    ) -> ast::Expression {
        use ast::*;

        let int_type = Type::Integer(crate::ast::Signedness::Unsigned, arr_elem_bits);

        let bytes_as_expr = vecmap(bytes, |byte| {
            Expression::Literal(Literal::Integer((byte as u128).into(), int_type.clone(), location))
        });

        let typ = Type::Array(bytes_as_expr.len() as u64, Box::new(int_type));

        let arr_literal = ArrayLiteral { typ, contents: bytes_as_expr };
        Expression::Literal(Literal::Array(arr_literal))
    }

    fn queue_function(
        &mut self,
        id: node_interner::FuncId,
        expr_id: node_interner::ExprId,
        function_type: HirType,
        turbofish_generics: Vec<HirType>,
        trait_method: Option<TraitMethodId>,
    ) -> FuncId {
        let new_id = self.next_function_id();
        self.define_function(id, function_type.clone(), turbofish_generics, new_id);

        let bindings = self.interner.get_instantiation_bindings(expr_id);
        let bindings = self.follow_bindings(bindings);
        self.queue.push_back((id, new_id, bindings, trait_method));
        new_id
    }

    /// Follow any type variable links within the given TypeBindings to produce
    /// a new TypeBindings that won't be changed when bindings are pushed or popped
    /// during {perform,undo}_monomorphization_bindings.
    ///
    /// Without this, a monomorphized type may fail to propagate passed more than 2
    /// function calls deep since it is possible for a previous link in the chain to
    /// unbind a type variable that was previously bound.
    fn follow_bindings(&self, bindings: &TypeBindings) -> TypeBindings {
        bindings
            .iter()
            .map(|(id, (var, binding))| {
                let binding2 = binding.follow_bindings();
                (*id, (var.clone(), binding2))
            })
            .collect()
    }

    fn assign(
        &mut self,
        assign: HirAssignStatement,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let expression = Box::new(self.expr(assign.expression)?);
        let lvalue = self.lvalue(assign.lvalue)?;
        Ok(ast::Expression::Assign(ast::Assign { expression, lvalue }))
    }

    fn lvalue(&mut self, lvalue: HirLValue) -> Result<ast::LValue, MonomorphizationError> {
        let value = match lvalue {
            HirLValue::Ident(ident, _) => match self.lookup_captured_lvalue(ident.id) {
                Some(value) => value,
                None => ast::LValue::Ident(self.local_ident(&ident)?.unwrap()),
            },
            HirLValue::MemberAccess { object, field_index, .. } => {
                let field_index = field_index.unwrap();
                let object = Box::new(self.lvalue(*object)?);
                ast::LValue::MemberAccess { object, field_index }
            }
            HirLValue::Index { array, index, typ, location } => {
                let array = Box::new(self.lvalue(*array)?);
                let index = Box::new(self.expr(index)?);
                let element_type = Self::convert_type(&typ, location)?;
                ast::LValue::Index { array, index, element_type, location }
            }
            HirLValue::Dereference { lvalue, element_type, location } => {
                let reference = Box::new(self.lvalue(*lvalue)?);
                let element_type = Self::convert_type(&element_type, location)?;
                ast::LValue::Dereference { reference, element_type }
            }
        };

        Ok(value)
    }

    fn lambda(
        &mut self,
        lambda: HirLambda,
        expr: node_interner::ExprId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        if lambda.captures.is_empty() {
            self.lambda_no_capture(lambda, expr)
        } else {
            let (setup, closure_variable) = self.lambda_with_setup(lambda, expr)?;
            Ok(ast::Expression::Block(vec![setup, closure_variable]))
        }
    }

    fn lambda_no_capture(
        &mut self,
        lambda: HirLambda,
        expr: node_interner::ExprId,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let location = self.interner.expr_location(&expr);
        let ret_type = Self::convert_type(&lambda.return_type, location)?;
        let lambda_name = "lambda";
        let parameter_types =
            try_vecmap(&lambda.parameters, |(_, typ)| Self::convert_type(typ, location))?;

        // Manually convert to Parameters type so we can reuse the self.parameters method
        let parameters =
            vecmap(lambda.parameters, |(pattern, typ)| (pattern, typ, Visibility::Private)).into();

        let parameters = self.parameters(&parameters)?;
        let body = self.expr(lambda.body)?;

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();
        let unconstrained = false;

        let function = ast::Function {
            id,
            name,
            parameters,
            body,
            return_type,
            unconstrained,
            inline_type: InlineType::default(),
            func_sig: FunctionSignature::default(),
        };
        self.push_function(id, function);

        let typ =
            ast::Type::Function(parameter_types, Box::new(ret_type), Box::new(ast::Type::Unit));

        let name = lambda_name.to_owned();
        Ok(ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None,
            name,
            typ,
        }))
    }

    fn lambda_with_setup(
        &mut self,
        lambda: HirLambda,
        expr: node_interner::ExprId,
    ) -> Result<(ast::Expression, ast::Expression), MonomorphizationError> {
        // returns (<closure setup>, <closure variable>)
        //   which can be used directly in callsites or transformed
        //   directly to a single `Expression`
        // for other cases by `lambda` which is called by `expr`
        //
        // it solves the problem of detecting special cases where
        // we call something like
        // `{let env$.. = ..;}.1({let env$.. = ..;}.0, ..)`
        // which was leading to redefinition errors
        //
        // instead of detecting and extracting
        // patterns in the resulting tree,
        // which seems more fragile, we directly reuse the return parameters
        // of this function in those cases
        let location = self.interner.expr_location(&expr);
        let ret_type = Self::convert_type(&lambda.return_type, location)?;
        let lambda_name = "lambda";
        let parameter_types =
            try_vecmap(&lambda.parameters, |(_, typ)| Self::convert_type(typ, location))?;

        // Manually convert to Parameters type so we can reuse the self.parameters method
        let parameters =
            vecmap(lambda.parameters, |(pattern, typ)| (pattern, typ, Visibility::Private)).into();

        let mut converted_parameters = self.parameters(&parameters)?;

        let id = self.next_function_id();
        let name = lambda_name.to_owned();
        let return_type = ret_type.clone();

        let env_local_id = self.next_local_id();
        let env_name = "env";
        let env_tuple =
            ast::Expression::Tuple(try_vecmap(&lambda.captures, |capture| {
                match capture.transitive_capture_index {
                    Some(field_index) => {
                        let lambda_ctx = self.lambda_envs_stack.last().expect(
                            "Expected to find a parent closure environment, but found none",
                        );

                        let ident = Box::new(ast::Expression::Ident(lambda_ctx.env_ident.clone()));
                        Ok(ast::Expression::ExtractTupleField(ident, field_index))
                    }
                    None => {
                        let ident = self.local_ident(&capture.ident)?.unwrap();
                        Ok(ast::Expression::Ident(ident))
                    }
                }
            })?);

        let expr_type = self.interner.id_type(expr);
        let env_typ = if let types::Type::Function(_, _, function_env_type) = expr_type {
            Self::convert_type(&function_env_type, location)?
        } else {
            unreachable!("expected a Function type for a Lambda node")
        };

        let env_let_stmt = ast::Expression::Let(ast::Let {
            id: env_local_id,
            mutable: false,
            name: env_name.to_string(),
            expression: Box::new(env_tuple),
        });

        let location = None; // TODO: This should match the location of the lambda expression
        let mutable = true;
        let definition = Definition::Local(env_local_id);

        let env_ident = ast::Ident {
            location,
            mutable,
            definition,
            name: env_name.to_string(),
            typ: env_typ.clone(),
        };

        self.lambda_envs_stack
            .push(LambdaContext { env_ident: env_ident.clone(), captures: lambda.captures });
        let body = self.expr(lambda.body)?;
        self.lambda_envs_stack.pop();

        let lambda_fn_typ: ast::Type =
            ast::Type::Function(parameter_types, Box::new(ret_type), Box::new(env_typ.clone()));
        let lambda_fn = ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None, // TODO: This should match the location of the lambda expression
            name: name.clone(),
            typ: lambda_fn_typ.clone(),
        });

        let mut parameters = vec![];
        parameters.push((env_local_id, true, env_name.to_string(), env_typ.clone()));
        parameters.append(&mut converted_parameters);

        let unconstrained = false;
        let function = ast::Function {
            id,
            name,
            parameters,
            body,
            return_type,
            unconstrained,
            inline_type: InlineType::default(),
            func_sig: FunctionSignature::default(),
        };
        self.push_function(id, function);

        let lambda_value =
            ast::Expression::Tuple(vec![ast::Expression::Ident(env_ident), lambda_fn]);
        let block_local_id = self.next_local_id();
        let block_ident_name = "closure_variable";
        let block_let_stmt = ast::Expression::Let(ast::Let {
            id: block_local_id,
            mutable: false,
            name: block_ident_name.to_string(),
            expression: Box::new(ast::Expression::Block(vec![env_let_stmt, lambda_value])),
        });

        let closure_definition = Definition::Local(block_local_id);

        let closure_ident = ast::Expression::Ident(ast::Ident {
            location,
            mutable: false,
            definition: closure_definition,
            name: block_ident_name.to_string(),
            typ: ast::Type::Tuple(vec![env_typ, lambda_fn_typ]),
        });

        Ok((block_let_stmt, closure_ident))
    }

    /// Implements std::unsafe::zeroed by returning an appropriate zeroed
    /// ast literal or collection node for the given type. Note that for functions
    /// there is no obvious zeroed value so this should be considered unsafe to use.
    fn zeroed_value_of_type(
        &mut self,
        typ: &ast::Type,
        location: noirc_errors::Location,
    ) -> ast::Expression {
        match typ {
            ast::Type::Field | ast::Type::Integer(..) => {
                let typ = typ.clone();
                ast::Expression::Literal(ast::Literal::Integer(0_u128.into(), typ, location))
            }
            ast::Type::Bool => ast::Expression::Literal(ast::Literal::Bool(false)),
            ast::Type::Unit => ast::Expression::Literal(ast::Literal::Unit),
            ast::Type::Array(length, element_type) => {
                let element = self.zeroed_value_of_type(element_type.as_ref(), location);
                ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral {
                    contents: vec![element; *length as usize],
                    typ: ast::Type::Array(*length, element_type.clone()),
                }))
            }
            ast::Type::String(length) => {
                ast::Expression::Literal(ast::Literal::Str("\0".repeat(*length as usize)))
            }
            ast::Type::FmtString(length, fields) => {
                let zeroed_tuple = self.zeroed_value_of_type(fields, location);
                let fields_len = match &zeroed_tuple {
                    ast::Expression::Tuple(fields) => fields.len() as u64,
                    _ => unreachable!("ICE: format string fields should be structured in a tuple, but got a {zeroed_tuple}"),
                };
                ast::Expression::Literal(ast::Literal::FmtStr(
                    "\0".repeat(*length as usize),
                    fields_len,
                    Box::new(zeroed_tuple),
                ))
            }
            ast::Type::Tuple(fields) => ast::Expression::Tuple(vecmap(fields, |field| {
                self.zeroed_value_of_type(field, location)
            })),
            ast::Type::Function(parameter_types, ret_type, env) => {
                self.create_zeroed_function(parameter_types, ret_type, env, location)
            }
            ast::Type::Slice(element_type) => {
                ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral {
                    contents: vec![],
                    typ: ast::Type::Slice(element_type.clone()),
                }))
            }
            ast::Type::MutableReference(element) => {
                use crate::ast::UnaryOp::MutableReference;
                let rhs = Box::new(self.zeroed_value_of_type(element, location));
                let result_type = typ.clone();
                ast::Expression::Unary(ast::Unary {
                    rhs,
                    result_type,
                    operator: MutableReference,
                    location,
                })
            }
        }
    }

    // Creating a zeroed function value is almost always an error if it is used later,
    // Hence why std::unsafe::zeroed is unsafe.
    //
    // To avoid confusing later passes, we arbitrarily choose to construct a function
    // that satisfies the input type by discarding all its parameters and returning a
    // zeroed value of the result type.
    fn create_zeroed_function(
        &mut self,
        parameter_types: &[ast::Type],
        ret_type: &ast::Type,
        env_type: &ast::Type,
        location: noirc_errors::Location,
    ) -> ast::Expression {
        let lambda_name = "zeroed_lambda";

        let parameters = vecmap(parameter_types, |parameter_type| {
            (self.next_local_id(), false, "_".into(), parameter_type.clone())
        });

        let body = self.zeroed_value_of_type(ret_type, location);

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();

        let unconstrained = false;
        let function = ast::Function {
            id,
            name,
            parameters,
            body,
            return_type,
            unconstrained,
            inline_type: InlineType::default(),
            func_sig: FunctionSignature::default(),
        };
        self.push_function(id, function);

        ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None,
            name: lambda_name.to_owned(),
            typ: ast::Type::Function(
                parameter_types.to_owned(),
                Box::new(ret_type.clone()),
                Box::new(env_type.clone()),
            ),
        })
    }

    /// Call an operator overloading method for the given operator.
    /// This function handles the special cases some operators have which don't map
    /// 1 to 1 onto their operator function. For example: != requires a negation on
    /// the result of its `eq` method, and the comparison operators each require a
    /// conversion from the `Ordering` result to a boolean.
    fn create_operator_impl_call(
        &self,
        func: ast::Expression,
        lhs: ast::Expression,
        operator: HirBinaryOp,
        rhs: ast::Expression,
        ret: Type,
        location: Location,
    ) -> Result<ast::Expression, MonomorphizationError> {
        let arguments = vec![lhs, rhs];
        let func = Box::new(func);
        let return_type = Self::convert_type(&ret, location)?;

        let mut result =
            ast::Expression::Call(ast::Call { func, arguments, return_type, location });

        use crate::ast::BinaryOpKind::*;
        match operator.kind {
            // Negate the result of the == operation
            NotEqual => {
                result = ast::Expression::Unary(ast::Unary {
                    operator: UnaryOp::Not,
                    rhs: Box::new(result),
                    result_type: ast::Type::Bool,
                    location,
                });
            }
            // All the comparison operators require special handling since their `cmp` method
            // returns an `Ordering` rather than a boolean value.
            //
            // (a < b) => a.cmp(b) == Ordering::Less
            // (a <= b) => a.cmp(b) != Ordering::Greater
            // (a > b) => a.cmp(b) == Ordering::Greater
            // (a >= b) => a.cmp(b) != Ordering::Less
            Less | LessEqual | Greater | GreaterEqual => {
                // Comparing an Ordering directly to a field value in this way takes advantage
                // of the fact the Ordering struct contains a single Field type, and our SSA
                // pass will automatically unpack tuple values.
                let ordering_value = if matches!(operator.kind, Less | GreaterEqual) {
                    FieldElement::zero() // Ordering::Less
                } else {
                    2u128.into() // Ordering::Greater
                };

                let operator =
                    if matches!(operator.kind, Less | Greater) { Equal } else { NotEqual };

                let int_value = ast::Literal::Integer(ordering_value, ast::Type::Field, location);
                let rhs = Box::new(ast::Expression::Literal(int_value));
                let lhs = Box::new(ast::Expression::ExtractTupleField(Box::new(result), 0));

                result = ast::Expression::Binary(ast::Binary { lhs, operator, rhs, location });
            }
            _ => (),
        }

        Ok(result)
    }

    /// Call sites are instantiated against the trait method, but when an impl is later selected,
    /// the corresponding method in the impl will have a different set of generics. `perform_impl_bindings`
    /// is needed to apply the generics from the trait method to the impl method. Without this,
    /// static method references to generic impls (e.g. `Eq::eq` for `[T; N]`) will fail to re-apply
    /// the correct type bindings during monomorphization.
    fn perform_impl_bindings(
        &self,
        trait_method: Option<TraitMethodId>,
        impl_method: node_interner::FuncId,
    ) -> TypeBindings {
        let mut bindings = TypeBindings::new();

        if let Some(trait_method) = trait_method {
            let the_trait = self.interner.get_trait(trait_method.trait_id);

            let trait_method_type = the_trait.methods[trait_method.method_index].typ.as_monotype();

            // Make each NamedGeneric in this type bindable by replacing it with a TypeVariable
            // with the same internal id and binding.
            let (generics, impl_method_type) =
                self.interner.function_meta(&impl_method).typ.unwrap_forall();

            let replace_type_variable = |var: &TypeVariable| {
                (var.id(), (var.clone(), Type::TypeVariable(var.clone(), TypeVariableKind::Normal)))
            };

            // Replace each NamedGeneric with a TypeVariable containing the same internal type variable
            let type_bindings = generics.iter().map(replace_type_variable).collect();
            let impl_method_type = impl_method_type.force_substitute(&type_bindings);

            trait_method_type.try_unify(&impl_method_type, &mut bindings).unwrap_or_else(|_| {
                unreachable!("Impl method type {} does not unify with trait method type {} during monomorphization", impl_method_type, trait_method_type)
            });

            perform_instantiation_bindings(&bindings);
        }

        bindings
    }
}

fn unwrap_tuple_type(typ: &HirType) -> Vec<HirType> {
    match typ.follow_bindings() {
        HirType::Tuple(fields) => fields.clone(),
        other => unreachable!("unwrap_tuple_type: expected tuple, found {:?}", other),
    }
}

fn unwrap_struct_type(typ: &HirType) -> Vec<(String, HirType)> {
    match typ.follow_bindings() {
        HirType::Struct(def, args) => def.borrow().get_fields(&args),
        other => unreachable!("unwrap_struct_type: expected struct, found {:?}", other),
    }
}

fn perform_instantiation_bindings(bindings: &TypeBindings) {
    for (var, binding) in bindings.values() {
        var.force_bind(binding.clone());
    }
}

fn undo_instantiation_bindings(bindings: TypeBindings) {
    for (id, (var, _)) in bindings {
        var.unbind(id);
    }
}
