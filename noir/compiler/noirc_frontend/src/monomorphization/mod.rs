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
use acvm::FieldElement;
use iter_extended::{btree_map, vecmap};
use noirc_errors::Location;
use noirc_printable_type::PrintableType;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    unreachable,
};

use crate::{
    hir_def::{
        expr::*,
        function::{FunctionSignature, Parameters},
        stmt::{HirAssignStatement, HirLValue, HirLetStatement, HirPattern, HirStatement},
        types,
    },
    node_interner::{self, DefinitionKind, NodeInterner, StmtId, TraitImplKind, TraitMethodId},
    token::FunctionAttribute,
    ContractFunctionType, FunctionKind, Type, TypeBinding, TypeBindings, TypeVariable,
    TypeVariableKind, UnaryOp, Visibility,
};

use self::ast::{Definition, FuncId, Function, LocalId, Program};

pub mod ast;
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
    /// Globals are keyed by their unique ID and expected type so that we can monomorphize
    /// a new version of the global for each type. Note that 'global' here means 'globally
    /// visible' and thus includes both functions and global variables.
    ///
    /// Using nested HashMaps here lets us avoid cloning HirTypes when calling .get()
    globals: HashMap<node_interner::FuncId, HashMap<HirType, FuncId>>,

    /// Unlike globals, locals are only keyed by their unique ID because they are never
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
    interner: &'interner NodeInterner,

    lambda_envs_stack: Vec<LambdaContext>,

    next_local_id: u32,
    next_function_id: u32,

    is_range_loop: bool,

    return_location: Option<Location>,
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
pub fn monomorphize(main: node_interner::FuncId, interner: &NodeInterner) -> Program {
    let mut monomorphizer = Monomorphizer::new(interner);
    let function_sig = monomorphizer.compile_main(main);

    while !monomorphizer.queue.is_empty() {
        let (next_fn_id, new_id, bindings, trait_method) = monomorphizer.queue.pop_front().unwrap();
        monomorphizer.locals.clear();

        perform_instantiation_bindings(&bindings);
        let impl_bindings = monomorphizer.perform_impl_bindings(trait_method, next_fn_id);
        monomorphizer.function(next_fn_id, new_id);
        undo_instantiation_bindings(impl_bindings);
        undo_instantiation_bindings(bindings);
    }

    let functions = vecmap(monomorphizer.finished_functions, |(_, f)| f);
    let meta = interner.function_meta(&main);

    Program::new(
        functions,
        function_sig,
        meta.return_distinctness,
        monomorphizer.return_location,
        meta.return_visibility,
    )
}

impl<'interner> Monomorphizer<'interner> {
    fn new(interner: &'interner NodeInterner) -> Self {
        Monomorphizer {
            globals: HashMap::new(),
            locals: HashMap::new(),
            queue: VecDeque::new(),
            finished_functions: BTreeMap::new(),
            next_local_id: 0,
            next_function_id: 0,
            interner,
            lambda_envs_stack: Vec::new(),
            is_range_loop: false,
            return_location: None,
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
        trait_method: Option<TraitMethodId>,
    ) -> Definition {
        let typ = typ.follow_bindings();
        match self.globals.get(&id).and_then(|inner_map| inner_map.get(&typ)) {
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
                        let id = self.queue_function(id, expr_id, typ, trait_method);
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
                }
            }
        }
    }

    fn define_local(&mut self, id: node_interner::DefinitionId, new_id: LocalId) {
        self.locals.insert(id, new_id);
    }

    /// Prerequisite: typ = typ.follow_bindings()
    fn define_global(&mut self, id: node_interner::FuncId, typ: HirType, new_id: FuncId) {
        self.globals.entry(id).or_default().insert(typ, new_id);
    }

    fn compile_main(&mut self, main_id: node_interner::FuncId) -> FunctionSignature {
        let new_main_id = self.next_function_id();
        assert_eq!(new_main_id, Program::main_id());
        self.function(main_id, new_main_id);
        self.return_location =
            self.interner.function(&main_id).block(self.interner).statements().last().and_then(
                |x| match self.interner.statement(x) {
                    HirStatement::Expression(id) => Some(self.interner.id_location(id)),
                    _ => None,
                },
            );
        let main_meta = self.interner.function_meta(&main_id);
        main_meta.function_signature()
    }

    fn function(&mut self, f: node_interner::FuncId, id: FuncId) {
        if let Some((self_type, trait_id)) = self.interner.get_function_trait(&f) {
            let the_trait = self.interner.get_trait(trait_id);
            the_trait.self_type_typevar.force_bind(self_type);
        }

        let meta = self.interner.function_meta(&f);
        let modifiers = self.interner.function_modifiers(&f);
        let name = self.interner.function_name(&f).to_owned();

        let body_expr_id = *self.interner.function(&f).as_expr();
        let body_return_type = self.interner.id_type(body_expr_id);
        let return_type = self.convert_type(match meta.return_type() {
            Type::TraitAsType(..) => &body_return_type,
            _ => meta.return_type(),
        });

        let parameters = self.parameters(&meta.parameters);

        let body = self.expr(body_expr_id);
        let unconstrained = modifiers.is_unconstrained
            || matches!(modifiers.contract_function_type, Some(ContractFunctionType::Open));

        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
        self.push_function(id, function);
    }

    fn push_function(&mut self, id: FuncId, function: ast::Function) {
        let existing = self.finished_functions.insert(id, function);
        assert!(existing.is_none());
    }

    /// Monomorphize each parameter, expanding tuple/struct patterns into multiple parameters
    /// and binding any generic types found.
    fn parameters(&mut self, params: &Parameters) -> Vec<(ast::LocalId, bool, String, ast::Type)> {
        let mut new_params = Vec::with_capacity(params.len());
        for (parameter, typ, _) in &params.0 {
            self.parameter(parameter, typ, &mut new_params);
        }
        new_params
    }

    fn parameter(
        &mut self,
        param: &HirPattern,
        typ: &HirType,
        new_params: &mut Vec<(ast::LocalId, bool, String, ast::Type)>,
    ) {
        match param {
            HirPattern::Identifier(ident) => {
                let new_id = self.next_local_id();
                let definition = self.interner.definition(ident.id);
                let name = definition.name.clone();
                new_params.push((new_id, definition.mutable, name, self.convert_type(typ)));
                self.define_local(ident.id, new_id);
            }
            HirPattern::Mutable(pattern, _) => self.parameter(pattern, typ, new_params),
            HirPattern::Tuple(fields, _) => {
                let tuple_field_types = unwrap_tuple_type(typ);

                for (field, typ) in fields.iter().zip(tuple_field_types) {
                    self.parameter(field, &typ, new_params);
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

                    self.parameter(field, &field_type, new_params);
                }
            }
        }
    }

    fn expr(&mut self, expr: node_interner::ExprId) -> ast::Expression {
        use ast::Expression::Literal;
        use ast::Literal::*;

        match self.interner.expression(&expr) {
            HirExpression::Ident(ident) => self.ident(ident, expr),
            HirExpression::Literal(HirLiteral::Str(contents)) => Literal(Str(contents)),
            HirExpression::Literal(HirLiteral::FmtStr(contents, idents)) => {
                let fields = vecmap(idents, |ident| self.expr(ident));
                Literal(FmtStr(
                    contents,
                    fields.len() as u64,
                    Box::new(ast::Expression::Tuple(fields)),
                ))
            }
            HirExpression::Literal(HirLiteral::Bool(value)) => Literal(Bool(value)),
            HirExpression::Literal(HirLiteral::Integer(value, sign)) => {
                if sign {
                    let typ = self.convert_type(&self.interner.id_type(expr));
                    let location = self.interner.id_location(expr);
                    match typ {
                        ast::Type::Field => Literal(Integer(-value, typ, location)),
                        ast::Type::Integer(_, bit_size) => {
                            let base = 1_u128 << bit_size;
                            Literal(Integer(FieldElement::from(base) - value, typ, location))
                        }
                        _ => unreachable!("Integer literal must be numeric"),
                    }
                } else {
                    let typ = self.convert_type(&self.interner.id_type(expr));
                    let location = self.interner.id_location(expr);
                    Literal(Integer(value, typ, location))
                }
            }
            HirExpression::Literal(HirLiteral::Array(array)) => match array {
                HirArrayLiteral::Standard(array) => self.standard_array(expr, array),
                HirArrayLiteral::Repeated { repeated_element, length } => {
                    self.repeated_array(expr, repeated_element, length)
                }
            },
            HirExpression::Literal(HirLiteral::Unit) => ast::Expression::Block(vec![]),
            HirExpression::Block(block) => self.block(block.0),

            HirExpression::Prefix(prefix) => {
                let location = self.interner.expr_location(&expr);
                ast::Expression::Unary(ast::Unary {
                    operator: prefix.operator,
                    rhs: Box::new(self.expr(prefix.rhs)),
                    result_type: self.convert_type(&self.interner.id_type(expr)),
                    location,
                })
            }

            HirExpression::Infix(infix) => {
                let lhs = self.expr(infix.lhs);
                let rhs = self.expr(infix.rhs);
                let operator = infix.operator.kind;
                let location = self.interner.expr_location(&expr);

                if self.interner.get_selected_impl_for_expression(expr).is_some() {
                    // If an impl was selected for this infix operator, replace it
                    // with a method call to the appropriate trait impl method.
                    let lhs_type = self.interner.id_type(infix.lhs);
                    let args = vec![lhs_type.clone(), lhs_type];

                    // If this is a comparison operator, the result is a boolean but
                    // the actual method call returns an Ordering
                    use crate::BinaryOpKind::*;
                    let ret = if matches!(operator, Less | LessEqual | Greater | GreaterEqual) {
                        self.interner.ordering_type()
                    } else {
                        self.interner.id_type(expr)
                    };

                    let env = Box::new(Type::Unit);
                    let function_type = Type::Function(args, Box::new(ret.clone()), env);

                    let method = infix.trait_method_id;
                    let func = self.resolve_trait_method_reference(expr, function_type, method);
                    self.create_operator_impl_call(func, lhs, infix.operator, rhs, ret, location)
                } else {
                    let lhs = Box::new(lhs);
                    let rhs = Box::new(rhs);
                    ast::Expression::Binary(ast::Binary { lhs, rhs, operator, location })
                }
            }

            HirExpression::Index(index) => self.index(expr, index),

            HirExpression::MemberAccess(access) => {
                let field_index = self.interner.get_field_index(expr);
                let expr = Box::new(self.expr(access.lhs));
                ast::Expression::ExtractTupleField(expr, field_index)
            }

            HirExpression::Call(call) => self.function_call(call, expr),

            HirExpression::Cast(cast) => ast::Expression::Cast(ast::Cast {
                lhs: Box::new(self.expr(cast.lhs)),
                r#type: self.convert_type(&cast.r#type),
                location: self.interner.expr_location(&expr),
            }),

            HirExpression::If(if_expr) => {
                let cond = self.expr(if_expr.condition);
                let then = self.expr(if_expr.consequence);
                let else_ = if_expr.alternative.map(|alt| Box::new(self.expr(alt)));
                ast::Expression::If(ast::If {
                    condition: Box::new(cond),
                    consequence: Box::new(then),
                    alternative: else_,
                    typ: self.convert_type(&self.interner.id_type(expr)),
                })
            }

            HirExpression::Tuple(fields) => {
                let fields = vecmap(fields, |id| self.expr(id));
                ast::Expression::Tuple(fields)
            }
            HirExpression::Constructor(constructor) => self.constructor(constructor, expr),

            HirExpression::Lambda(lambda) => self.lambda(lambda, expr),

            HirExpression::MethodCall(hir_method_call) => {
                unreachable!("Encountered HirExpression::MethodCall during monomorphization {hir_method_call:?}")
            }
            HirExpression::Error => unreachable!("Encountered Error node during monomorphization"),
        }
    }

    fn standard_array(
        &mut self,
        array: node_interner::ExprId,
        array_elements: Vec<node_interner::ExprId>,
    ) -> ast::Expression {
        let typ = self.convert_type(&self.interner.id_type(array));
        let contents = vecmap(array_elements, |id| self.expr(id));
        ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral { contents, typ }))
    }

    fn repeated_array(
        &mut self,
        array: node_interner::ExprId,
        repeated_element: node_interner::ExprId,
        length: HirType,
    ) -> ast::Expression {
        let typ = self.convert_type(&self.interner.id_type(array));

        let length = length
            .evaluate_to_u64()
            .expect("Length of array is unknown when evaluating numeric generic");

        let contents = vecmap(0..length, |_| self.expr(repeated_element));
        ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral { contents, typ }))
    }

    fn index(&mut self, id: node_interner::ExprId, index: HirIndexExpression) -> ast::Expression {
        let element_type = self.convert_type(&self.interner.id_type(id));

        let collection = Box::new(self.expr(index.collection));
        let index = Box::new(self.expr(index.index));
        let location = self.interner.expr_location(&id);
        ast::Expression::Index(ast::Index { collection, index, element_type, location })
    }

    fn statement(&mut self, id: StmtId) -> ast::Expression {
        match self.interner.statement(&id) {
            HirStatement::Let(let_statement) => self.let_statement(let_statement),
            HirStatement::Constrain(constrain) => {
                let expr = self.expr(constrain.0);
                let location = self.interner.expr_location(&constrain.0);
                ast::Expression::Constrain(Box::new(expr), location, constrain.2)
            }
            HirStatement::Assign(assign) => self.assign(assign),
            HirStatement::For(for_loop) => {
                self.is_range_loop = true;
                let start = self.expr(for_loop.start_range);
                let end = self.expr(for_loop.end_range);
                self.is_range_loop = false;
                let index_variable = self.next_local_id();
                self.define_local(for_loop.identifier.id, index_variable);

                let block = Box::new(self.expr(for_loop.block));

                ast::Expression::For(ast::For {
                    index_variable,
                    index_name: self.interner.definition_name(for_loop.identifier.id).to_owned(),
                    index_type: self.convert_type(&self.interner.id_type(for_loop.start_range)),
                    start_range: Box::new(start),
                    end_range: Box::new(end),
                    start_range_location: self.interner.expr_location(&for_loop.start_range),
                    end_range_location: self.interner.expr_location(&for_loop.end_range),
                    block,
                })
            }
            HirStatement::Expression(expr) => self.expr(expr),
            HirStatement::Semi(expr) => ast::Expression::Semi(Box::new(self.expr(expr))),
            HirStatement::Error => unreachable!(),
        }
    }

    fn let_statement(&mut self, let_statement: HirLetStatement) -> ast::Expression {
        let expr = self.expr(let_statement.expression);
        let expected_type = self.interner.id_type(let_statement.expression);
        self.unpack_pattern(let_statement.pattern, expr, &expected_type)
    }

    fn constructor(
        &mut self,
        constructor: HirConstructorExpression,
        id: node_interner::ExprId,
    ) -> ast::Expression {
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
            let typ = self.convert_type(field_type);

            field_vars.insert(field_name.0.contents.clone(), (new_id, typ));
            let expression = Box::new(self.expr(expr_id));

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
        ast::Expression::Block(new_exprs)
    }

    fn block(&mut self, statement_ids: Vec<StmtId>) -> ast::Expression {
        ast::Expression::Block(vecmap(statement_ids, |id| self.statement(id)))
    }

    fn unpack_pattern(
        &mut self,
        pattern: HirPattern,
        value: ast::Expression,
        typ: &HirType,
    ) -> ast::Expression {
        match pattern {
            HirPattern::Identifier(ident) => {
                let new_id = self.next_local_id();
                self.define_local(ident.id, new_id);
                let definition = self.interner.definition(ident.id);

                ast::Expression::Let(ast::Let {
                    id: new_id,
                    mutable: definition.mutable,
                    name: definition.name.clone(),
                    expression: Box::new(value),
                })
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
    ) -> ast::Expression {
        let fresh_id = self.next_local_id();

        let mut definitions = vec![ast::Expression::Let(ast::Let {
            id: fresh_id,
            mutable: false,
            name: "_".into(),
            expression: Box::new(value),
        })];

        for (i, (field_pattern, field_type)) in fields.into_iter().enumerate() {
            let location = None;
            let mutable = false;
            let definition = Definition::Local(fresh_id);
            let name = i.to_string();
            let typ = self.convert_type(&field_type);

            let new_rhs =
                ast::Expression::Ident(ast::Ident { location, mutable, definition, name, typ });

            let new_rhs = ast::Expression::ExtractTupleField(Box::new(new_rhs), i);
            let new_expr = self.unpack_pattern(field_pattern, new_rhs, &field_type);
            definitions.push(new_expr);
        }

        ast::Expression::Block(definitions)
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

    /// A local (ie non-global) ident only
    fn local_ident(&mut self, ident: &HirIdent) -> Option<ast::Ident> {
        let definition = self.interner.definition(ident.id);
        let name = definition.name.clone();
        let mutable = definition.mutable;

        let definition = self.lookup_local(ident.id)?;
        let typ = self.convert_type(&self.interner.id_type(ident.id));

        Some(ast::Ident { location: Some(ident.location), mutable, definition, name, typ })
    }

    fn ident(&mut self, ident: HirIdent, expr_id: node_interner::ExprId) -> ast::Expression {
        let typ = self.interner.id_type(expr_id);

        if let ImplKind::TraitMethod(method, _, _) = ident.impl_kind {
            return self.resolve_trait_method_reference(expr_id, typ, method);
        }

        let definition = self.interner.definition(ident.id);
        match &definition.kind {
            DefinitionKind::Function(func_id) => {
                let mutable = definition.mutable;
                let location = Some(ident.location);
                let name = definition.name.clone();
                let typ = self.interner.id_type(expr_id);
                let definition = self.lookup_function(*func_id, expr_id, &typ, None);
                let typ = self.convert_type(&typ);
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
            DefinitionKind::Global(expr_id) => self.expr(*expr_id),
            DefinitionKind::Local(_) => self.lookup_captured_expr(ident.id).unwrap_or_else(|| {
                let ident = self.local_ident(&ident).unwrap();
                ast::Expression::Ident(ident)
            }),
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
                ast::Expression::Literal(ast::Literal::Integer(value, ast::Type::Field, location))
            }
        }
    }

    /// Convert a non-tuple/struct type to a monomorphized type
    fn convert_type(&self, typ: &HirType) -> ast::Type {
        match typ {
            HirType::FieldElement => ast::Type::Field,
            HirType::Integer(sign, bits) => ast::Type::Integer(*sign, *bits),
            HirType::Bool => ast::Type::Bool,
            HirType::String(size) => ast::Type::String(size.evaluate_to_u64().unwrap_or(0)),
            HirType::FmtString(size, fields) => {
                let size = size.evaluate_to_u64().unwrap_or(0);
                let fields = Box::new(self.convert_type(fields.as_ref()));
                ast::Type::FmtString(size, fields)
            }
            HirType::Unit => ast::Type::Unit,
            HirType::Array(length, element) => {
                let element = Box::new(self.convert_type(element.as_ref()));

                if let Some(length) = length.evaluate_to_u64() {
                    ast::Type::Array(length, element)
                } else {
                    ast::Type::Slice(element)
                }
            }
            HirType::TraitAsType(..) => {
                unreachable!("All TraitAsType should be replaced before calling convert_type");
            }
            HirType::NamedGeneric(binding, _) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return self.convert_type(binding);
                }

                // Default any remaining unbound type variables.
                // This should only happen if the variable in question is unused
                // and within a larger generic type.
                binding.bind(HirType::default_int_type());
                ast::Type::Field
            }

            HirType::TypeVariable(binding, kind) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return self.convert_type(binding);
                }

                // Default any remaining unbound type variables.
                // This should only happen if the variable in question is unused
                // and within a larger generic type.
                let default =
                    if self.is_range_loop && matches!(kind, TypeVariableKind::IntegerOrField) {
                        Type::default_range_loop_type()
                    } else {
                        kind.default_type()
                    };

                let monomorphized_default = self.convert_type(&default);
                binding.bind(default);
                monomorphized_default
            }

            HirType::Struct(def, args) => {
                let fields = def.borrow().get_fields(args);
                let fields = vecmap(fields, |(_, field)| self.convert_type(&field));
                ast::Type::Tuple(fields)
            }

            HirType::Tuple(fields) => {
                let fields = vecmap(fields, |x| self.convert_type(x));
                ast::Type::Tuple(fields)
            }

            HirType::Function(args, ret, env) => {
                let args = vecmap(args, |x| self.convert_type(x));
                let ret = Box::new(self.convert_type(ret));
                let env = self.convert_type(env);
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
                let element = self.convert_type(element);
                ast::Type::MutableReference(Box::new(element))
            }

            HirType::Forall(_, _)
            | HirType::Constant(_)
            | HirType::NotConstant
            | HirType::Error => {
                unreachable!("Unexpected type {} found", typ)
            }
        }
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
    ) -> ast::Expression {
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
                            let name = self.interner.get_trait(id).name.to_string();
                            format!("  {}: {name}", constraint.typ)
                        })
                        .join("\n");

                        unreachable!("Failed to find trait impl during monomorphization. The failed constraint(s) are:\n{failed_constraints}")
                    }
                }
            }
        };

        let func_id = match self.lookup_function(func_id, expr_id, &function_type, Some(method)) {
            Definition::Function(func_id) => func_id,
            _ => unreachable!(),
        };

        let the_trait = self.interner.get_trait(method.trait_id);
        ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(func_id),
            mutable: false,
            location: None,
            name: the_trait.methods[method.method_index].name.0.contents.clone(),
            typ: self.convert_type(&function_type),
        })
    }

    fn function_call(
        &mut self,
        call: HirCallExpression,
        id: node_interner::ExprId,
    ) -> ast::Expression {
        let original_func = Box::new(self.expr(call.func));
        let mut arguments = vecmap(&call.arguments, |id| self.expr(*id));
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let return_type = self.interner.id_type(id);
        let return_type = self.convert_type(&return_type);

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
        let func_type = self.convert_type(&func_type);
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
                typ: self.convert_type(&self.interner.id_type(call.func)),
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
            ast::Expression::Block(block_expressions)
        } else {
            call
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
            HirExpression::Ident(ident) => {
                let typ = self.interner.id_type(ident.id);
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
            _ => unreachable!("logging expr {:?} is not supported", arguments[0]),
        }
    }

    fn append_printable_type_info_inner(typ: &Type, arguments: &mut Vec<ast::Expression>) {
        // Disallow printing slices and mutable references for consistency,
        // since they cannot be passed from ACIR into Brillig
        if let HirType::Array(size, _) = typ {
            if let HirType::NotConstant = **size {
                unreachable!("println does not support slices. Convert the slice to an array before passing it to println");
            }
        } else if matches!(typ, HirType::MutableReference(_)) {
            unreachable!("println does not support mutable references.");
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
                        let typ = ast::Type::Field;
                        Some(ast::Expression::Literal(ast::Literal::Integer(bits, typ, location)))
                    }
                    "zeroed" => {
                        let location = self.interner.expr_location(expr_id);
                        Some(self.zeroed_value_of_type(result_type, location))
                    }
                    "modulus_le_bits" => {
                        let bits = FieldElement::modulus().to_radix_le(2);
                        Some(self.modulus_array_literal(bits, 1, location))
                    }
                    "modulus_be_bits" => {
                        let bits = FieldElement::modulus().to_radix_be(2);
                        Some(self.modulus_array_literal(bits, 1, location))
                    }
                    "modulus_be_bytes" => {
                        let bytes = FieldElement::modulus().to_bytes_be();
                        Some(self.modulus_array_literal(bytes, 8, location))
                    }
                    "modulus_le_bytes" => {
                        let bytes = FieldElement::modulus().to_bytes_le();
                        Some(self.modulus_array_literal(bytes, 8, location))
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
        arr_elem_bits: u32,
        location: Location,
    ) -> ast::Expression {
        use ast::*;
        let int_type = Type::Integer(crate::Signedness::Unsigned, arr_elem_bits);

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
        trait_method: Option<TraitMethodId>,
    ) -> FuncId {
        let new_id = self.next_function_id();
        self.define_global(id, function_type.clone(), new_id);

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

    fn assign(&mut self, assign: HirAssignStatement) -> ast::Expression {
        let expression = Box::new(self.expr(assign.expression));
        let lvalue = self.lvalue(assign.lvalue);
        ast::Expression::Assign(ast::Assign { expression, lvalue })
    }

    fn lvalue(&mut self, lvalue: HirLValue) -> ast::LValue {
        match lvalue {
            HirLValue::Ident(ident, _) => self
                .lookup_captured_lvalue(ident.id)
                .unwrap_or_else(|| ast::LValue::Ident(self.local_ident(&ident).unwrap())),
            HirLValue::MemberAccess { object, field_index, .. } => {
                let field_index = field_index.unwrap();
                let object = Box::new(self.lvalue(*object));
                ast::LValue::MemberAccess { object, field_index }
            }
            HirLValue::Index { array, index, typ } => {
                let location = self.interner.expr_location(&index);
                let array = Box::new(self.lvalue(*array));
                let index = Box::new(self.expr(index));
                let element_type = self.convert_type(&typ);
                ast::LValue::Index { array, index, element_type, location }
            }
            HirLValue::Dereference { lvalue, element_type } => {
                let reference = Box::new(self.lvalue(*lvalue));
                let element_type = self.convert_type(&element_type);
                ast::LValue::Dereference { reference, element_type }
            }
        }
    }

    fn lambda(&mut self, lambda: HirLambda, expr: node_interner::ExprId) -> ast::Expression {
        if lambda.captures.is_empty() {
            self.lambda_no_capture(lambda)
        } else {
            let (setup, closure_variable) = self.lambda_with_setup(lambda, expr);
            ast::Expression::Block(vec![setup, closure_variable])
        }
    }

    fn lambda_no_capture(&mut self, lambda: HirLambda) -> ast::Expression {
        let ret_type = self.convert_type(&lambda.return_type);
        let lambda_name = "lambda";
        let parameter_types = vecmap(&lambda.parameters, |(_, typ)| self.convert_type(typ));

        // Manually convert to Parameters type so we can reuse the self.parameters method
        let parameters =
            vecmap(lambda.parameters, |(pattern, typ)| (pattern, typ, Visibility::Private)).into();

        let parameters = self.parameters(&parameters);
        let body = self.expr(lambda.body);

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();
        let unconstrained = false;

        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
        self.push_function(id, function);

        let typ =
            ast::Type::Function(parameter_types, Box::new(ret_type), Box::new(ast::Type::Unit));

        let name = lambda_name.to_owned();
        ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None,
            name,
            typ,
        })
    }

    fn lambda_with_setup(
        &mut self,
        lambda: HirLambda,
        expr: node_interner::ExprId,
    ) -> (ast::Expression, ast::Expression) {
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
        let ret_type = self.convert_type(&lambda.return_type);
        let lambda_name = "lambda";
        let parameter_types = vecmap(&lambda.parameters, |(_, typ)| self.convert_type(typ));

        // Manually convert to Parameters type so we can reuse the self.parameters method
        let parameters =
            vecmap(lambda.parameters, |(pattern, typ)| (pattern, typ, Visibility::Private)).into();

        let mut converted_parameters = self.parameters(&parameters);

        let id = self.next_function_id();
        let name = lambda_name.to_owned();
        let return_type = ret_type.clone();

        let env_local_id = self.next_local_id();
        let env_name = "env";
        let env_tuple = ast::Expression::Tuple(vecmap(&lambda.captures, |capture| {
            match capture.transitive_capture_index {
                Some(field_index) => match self.lambda_envs_stack.last() {
                    Some(lambda_ctx) => ast::Expression::ExtractTupleField(
                        Box::new(ast::Expression::Ident(lambda_ctx.env_ident.clone())),
                        field_index,
                    ),
                    None => unreachable!(
                        "Expected to find a parent closure environment, but found none"
                    ),
                },
                None => {
                    let ident = self.local_ident(&capture.ident).unwrap();
                    ast::Expression::Ident(ident)
                }
            }
        }));
        let expr_type = self.interner.id_type(expr);
        let env_typ = if let types::Type::Function(_, _, function_env_type) = expr_type {
            self.convert_type(&function_env_type)
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
        let body = self.expr(lambda.body);
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
        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
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

        (block_let_stmt, closure_ident)
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
            // There is no unit literal currently. Replace it with 'false' since it should be ignored
            // anyway.
            ast::Type::Unit => ast::Expression::Literal(ast::Literal::Bool(false)),
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
                use crate::UnaryOp::MutableReference;
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
        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
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
    ) -> ast::Expression {
        let arguments = vec![lhs, rhs];
        let func = Box::new(func);
        let return_type = self.convert_type(&ret);

        let mut result =
            ast::Expression::Call(ast::Call { func, arguments, return_type, location });

        use crate::BinaryOpKind::*;
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

        result
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
    match typ {
        HirType::Tuple(fields) => fields.clone(),
        HirType::TypeVariable(binding, TypeVariableKind::Normal) => match &*binding.borrow() {
            TypeBinding::Bound(binding) => unwrap_tuple_type(binding),
            TypeBinding::Unbound(_) => unreachable!(),
        },
        other => unreachable!("unwrap_tuple_type: expected tuple, found {:?}", other),
    }
}

fn unwrap_struct_type(typ: &HirType) -> Vec<(String, HirType)> {
    match typ {
        HirType::Struct(def, args) => def.borrow().get_fields(args),
        HirType::TypeVariable(binding, TypeVariableKind::Normal) => match &*binding.borrow() {
            TypeBinding::Bound(binding) => unwrap_struct_type(binding),
            TypeBinding::Unbound(_) => unreachable!(),
        },
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
