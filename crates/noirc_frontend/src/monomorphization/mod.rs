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
use noirc_abi::FunctionSignature;
use noirc_errors::Location;
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::{
    hir_def::{
        expr::*,
        function::{FuncMeta, Param, Parameters},
        stmt::{HirAssignStatement, HirLValue, HirLetStatement, HirPattern, HirStatement},
    },
    node_interner::{self, DefinitionKind, NodeInterner, StmtId},
    CompTime, FunctionKind, TypeBinding, TypeBindings,
};

use self::ast::{Definition, FuncId, Function, LocalId, Program};

pub mod ast;
pub mod printer;

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

    /// Queue of functions to monomorphize next
    queue: VecDeque<(node_interner::FuncId, FuncId, TypeBindings)>,

    /// When a function finishes being monomorphized, the monomorphized ast::Function is
    /// stored here along with its FuncId.
    finished_functions: BTreeMap<FuncId, Function>,

    /// Used to reference existing definitions in the HIR
    interner: &'interner NodeInterner,

    next_local_id: u32,
    next_function_id: u32,
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
pub fn monomorphize(main: node_interner::FuncId, interner: &NodeInterner) -> Program {
    let mut monomorphizer = Monomorphizer::new(interner);
    let function_sig = monomorphizer.compile_main(main);

    while !monomorphizer.queue.is_empty() {
        let (next_fn_id, new_id, bindings) = monomorphizer.queue.pop_front().unwrap();
        monomorphizer.locals.clear();

        perform_instantiation_bindings(&bindings);
        monomorphizer.function(next_fn_id, new_id);
        undo_instantiation_bindings(bindings);
    }

    let functions = vecmap(monomorphizer.finished_functions, |(_, f)| f);
    let FuncMeta { return_distinctness, .. } = interner.function_meta(&main);
    Program::new(functions, function_sig, return_distinctness)
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
    ) -> Definition {
        let typ = typ.follow_bindings();
        match self.globals.get(&id).and_then(|inner_map| inner_map.get(&typ)) {
            Some(id) => Definition::Function(*id),
            None => {
                // Function has not been monomorphized yet
                let meta = self.interner.function_meta(&id);
                match meta.kind {
                    FunctionKind::LowLevel => {
                        let attribute = meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                        let opcode = attribute.foreign().expect(
                            "ice: function marked as foreign, but attribute kind does not match this",
                        );
                        Definition::LowLevel(opcode)
                    }
                    FunctionKind::Builtin => {
                        let attribute = meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                        let opcode = attribute.builtin().expect(
                            "ice: function marked as builtin, but attribute kind does not match this",
                        );
                        Definition::Builtin(opcode)
                    }
                    FunctionKind::Normal => {
                        let id = self.queue_function(id, expr_id, typ);
                        Definition::Function(id)
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

        let main_meta = self.interner.function_meta(&main_id);
        main_meta.into_function_signature(self.interner)
    }

    fn function(&mut self, f: node_interner::FuncId, id: FuncId) {
        let meta = self.interner.function_meta(&f);
        let name = self.interner.function_name(&f).to_owned();

        let return_type = Self::convert_type(meta.return_type());
        let parameters = self.parameters(meta.parameters);
        let body = self.expr(*self.interner.function(&f).as_expr());
        let unconstrained = meta.is_unconstrained;

        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
        self.push_function(id, function);
    }

    fn push_function(&mut self, id: FuncId, function: ast::Function) {
        let existing = self.finished_functions.insert(id, function);
        assert!(existing.is_none());
    }

    /// Monomorphize each parameter, expanding tuple/struct patterns into multiple parameters
    /// and binding any generic types found.
    fn parameters(&mut self, params: Parameters) -> Vec<(ast::LocalId, bool, String, ast::Type)> {
        let mut new_params = Vec::with_capacity(params.len());
        for parameter in params {
            self.parameter(parameter.0, &parameter.1, &mut new_params);
        }
        new_params
    }

    fn parameter(
        &mut self,
        param: HirPattern,
        typ: &HirType,
        new_params: &mut Vec<(ast::LocalId, bool, String, ast::Type)>,
    ) {
        match param {
            HirPattern::Identifier(ident) => {
                //let value = self.expand_parameter(typ, new_params);
                let new_id = self.next_local_id();
                let definition = self.interner.definition(ident.id);
                let name = definition.name.clone();
                new_params.push((new_id, definition.mutable, name, Self::convert_type(typ)));
                self.define_local(ident.id, new_id);
            }
            HirPattern::Mutable(pattern, _) => self.parameter(*pattern, typ, new_params),
            HirPattern::Tuple(fields, _) => {
                let tuple_field_types = unwrap_tuple_type(typ);

                for (field, typ) in fields.into_iter().zip(tuple_field_types) {
                    self.parameter(field, &typ, new_params);
                }
            }
            HirPattern::Struct(_, fields, _) => {
                let struct_field_types = unwrap_struct_type(typ);
                assert_eq!(struct_field_types.len(), fields.len());

                let mut fields = btree_map(fields, |(name, field)| (name.0.contents, field));

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
            HirExpression::Literal(HirLiteral::Bool(value)) => Literal(Bool(value)),
            HirExpression::Literal(HirLiteral::Integer(value)) => {
                let typ = Self::convert_type(&self.interner.id_type(expr));
                Literal(Integer(value, typ))
            }
            HirExpression::Literal(HirLiteral::Array(array)) => match array {
                HirArrayLiteral::Standard(array) => self.standard_array(array),
                HirArrayLiteral::Repeated { repeated_element, length } => {
                    self.repeated_array(repeated_element, length)
                }
            },
            HirExpression::Block(block) => self.block(block.0),

            HirExpression::Prefix(prefix) => ast::Expression::Unary(ast::Unary {
                operator: prefix.operator,
                rhs: Box::new(self.expr(prefix.rhs)),
            }),

            HirExpression::Infix(infix) => {
                let lhs = Box::new(self.expr(infix.lhs));
                let rhs = Box::new(self.expr(infix.rhs));
                let operator = infix.operator.kind;
                let location = self.interner.expr_location(&expr);
                ast::Expression::Binary(ast::Binary { lhs, rhs, operator, location })
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
                r#type: Self::convert_type(&cast.r#type),
            }),

            HirExpression::For(for_expr) => {
                let start = self.expr(for_expr.start_range);
                let end = self.expr(for_expr.end_range);
                let index_variable = self.next_local_id();
                self.define_local(for_expr.identifier.id, index_variable);

                let block = Box::new(self.expr(for_expr.block));

                ast::Expression::For(ast::For {
                    index_variable,
                    index_name: self.interner.definition_name(for_expr.identifier.id).to_owned(),
                    index_type: Self::convert_type(&self.interner.id_type(for_expr.start_range)),
                    start_range: Box::new(start),
                    end_range: Box::new(end),
                    block,
                })
            }

            HirExpression::If(if_expr) => {
                let cond = self.expr(if_expr.condition);
                let then = self.expr(if_expr.consequence);
                let else_ = if_expr.alternative.map(|alt| Box::new(self.expr(alt)));
                ast::Expression::If(ast::If {
                    condition: Box::new(cond),
                    consequence: Box::new(then),
                    alternative: else_,
                    typ: Self::convert_type(&self.interner.id_type(expr)),
                })
            }

            HirExpression::Tuple(fields) => {
                let fields = vecmap(fields, |id| self.expr(id));
                ast::Expression::Tuple(fields)
            }
            HirExpression::Constructor(constructor) => self.constructor(constructor, expr),

            HirExpression::Lambda(lambda) => self.lambda(lambda),

            HirExpression::MethodCall(_) => {
                unreachable!("Encountered HirExpression::MethodCall during monomorphization")
            }
            HirExpression::Error => unreachable!("Encountered Error node during monomorphization"),
        }
    }

    fn standard_array(&mut self, array: Vec<node_interner::ExprId>) -> ast::Expression {
        let element_type = Self::convert_type(&self.interner.id_type(array[0]));
        let contents = vecmap(array, |id| self.expr(id));
        Self::aos_to_soa(contents, element_type)
    }

    fn repeated_array(
        &mut self,
        repeated_element: node_interner::ExprId,
        length: HirType,
    ) -> ast::Expression {
        let element_type = Self::convert_type(&self.interner.id_type(repeated_element));
        let contents = self.expr(repeated_element);
        let length = length
            .evaluate_to_u64()
            .expect("Length of array is unknown when evaluating numeric generic");

        let contents = vec![contents; length as usize];
        Self::aos_to_soa(contents, element_type)
    }

    /// Convert an array in (potentially) array of structs form into struct of arrays form.
    /// This will do nothing if the given array element type is a primitive type like Field.
    ///
    ///
    /// TODO Remove side effects from clones
    fn aos_to_soa(
        array_contents: Vec<ast::Expression>,
        element_type: ast::Type,
    ) -> ast::Expression {
        match element_type {
            ast::Type::Field
            | ast::Type::Integer(_, _)
            | ast::Type::Bool
            | ast::Type::Unit
            | ast::Type::Function(_, _) => {
                ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral {
                    contents: array_contents,
                    element_type,
                }))
            }

            ast::Type::Tuple(elements) => ast::Expression::Tuple(vecmap(
                elements.into_iter().enumerate(),
                |(i, element_type)| {
                    let contents = vecmap(&array_contents, |element| {
                        ast::Expression::ExtractTupleField(Box::new(element.clone()), i)
                    });

                    Self::aos_to_soa(contents, element_type)
                },
            )),

            ast::Type::Array(_, _) | ast::Type::String(_) | ast::Type::Vec(_) => {
                unreachable!("Nested arrays, arrays of strings, and Vecs are not supported")
            }
        }
    }

    fn index(&mut self, id: node_interner::ExprId, index: HirIndexExpression) -> ast::Expression {
        let element_type = Self::convert_type(&self.interner.id_type(id));

        let collection = Box::new(self.expr(index.collection));
        let index = Box::new(self.expr(index.index));
        let location = self.interner.expr_location(&id);
        Self::aos_to_soa_index(collection, index, element_type, location)
    }

    /// Unpack an array index into an array of structs into a struct of arrays index if needed.
    /// E.g. transforms my_pair_array[i] into (my_pair1_array[i], my_pair2_array[i])
    fn aos_to_soa_index(
        collection: Box<ast::Expression>,
        index: Box<ast::Expression>,
        element_type: ast::Type,
        location: Location,
    ) -> ast::Expression {
        match element_type {
            ast::Type::Field
            | ast::Type::Integer(_, _)
            | ast::Type::Bool
            | ast::Type::Unit
            | ast::Type::Function(_, _) => {
                ast::Expression::Index(ast::Index { collection, index, element_type, location })
            }

            ast::Type::Tuple(elements) => {
                let elements = elements.into_iter().enumerate();
                ast::Expression::Tuple(vecmap(elements, |(i, element_type)| {
                    // collection should itself be a tuple of arrays
                    let collection =
                        Box::new(ast::Expression::ExtractTupleField(collection.clone(), i));

                    Self::aos_to_soa_index(collection, index.clone(), element_type, location)
                }))
            }

            ast::Type::Array(_, _) | ast::Type::String(_) | ast::Type::Vec(_) => {
                unreachable!("Nested arrays and arrays of strings or Vecs are not supported")
            }
        }
    }

    fn statement(&mut self, id: StmtId) -> ast::Expression {
        match self.interner.statement(&id) {
            HirStatement::Let(let_statement) => self.let_statement(let_statement),
            HirStatement::Constrain(constrain) => {
                let expr = self.expr(constrain.0);
                let location = self.interner.expr_location(&constrain.0);
                ast::Expression::Constrain(Box::new(expr), location)
            }
            HirStatement::Assign(assign) => self.assign(assign),
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
            let typ = Self::convert_type(field_type);

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
            let typ = Self::convert_type(&field_type);

            let new_rhs =
                ast::Expression::Ident(ast::Ident { location, mutable, definition, name, typ });

            let new_rhs = ast::Expression::ExtractTupleField(Box::new(new_rhs), i);
            let new_expr = self.unpack_pattern(field_pattern, new_rhs, &field_type);
            definitions.push(new_expr);
        }

        ast::Expression::Block(definitions)
    }

    /// A local (ie non-global) ident only
    fn local_ident(&mut self, ident: &HirIdent) -> Option<ast::Ident> {
        let definition = self.interner.definition(ident.id);
        let name = definition.name.clone();
        let mutable = definition.mutable;

        let definition = self.lookup_local(ident.id)?;
        let typ = Self::convert_type(&self.interner.id_type(ident.id));

        Some(ast::Ident { location: Some(ident.location), mutable, definition, name, typ })
    }

    fn ident(&mut self, ident: HirIdent, expr_id: node_interner::ExprId) -> ast::Expression {
        let definition = self.interner.definition(ident.id);
        match &definition.kind {
            DefinitionKind::Function(func_id) => {
                let mutable = definition.mutable;
                let location = Some(ident.location);
                let name = definition.name.clone();
                let typ = self.interner.id_type(expr_id);

                let definition = self.lookup_function(*func_id, expr_id, &typ);
                let typ = Self::convert_type(&typ);
                let ident = ast::Ident { location, mutable, definition, name, typ };
                ast::Expression::Ident(ident)
            }
            DefinitionKind::Global(expr_id) => self.expr(*expr_id),
            DefinitionKind::Local(_) => {
                let ident = self.local_ident(&ident).unwrap();
                ast::Expression::Ident(ident)
            }
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
                ast::Expression::Literal(ast::Literal::Integer(value, ast::Type::Field))
            }
        }
    }

    /// Convert a non-tuple/struct type to a monomorphized type
    fn convert_type(typ: &HirType) -> ast::Type {
        match typ {
            HirType::FieldElement(_) => ast::Type::Field,
            HirType::Integer(_, sign, bits) => ast::Type::Integer(*sign, *bits),
            HirType::Bool(_) => ast::Type::Bool,
            HirType::String(size) => ast::Type::String(size.evaluate_to_u64().unwrap_or(0)),
            HirType::Unit => ast::Type::Unit,

            HirType::Array(length, element) => {
                let length = length.evaluate_to_u64().unwrap_or(0);
                let element = Self::convert_type(element.as_ref());
                Self::aos_to_soa_type(length, element)
            }

            HirType::PolymorphicInteger(_, binding)
            | HirType::TypeVariable(binding)
            | HirType::NamedGeneric(binding, _) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return Self::convert_type(binding);
                }

                // Default any remaining unbound type variables to Field.
                // This should only happen if the variable in question is unused
                // and within a larger generic type.
                // NOTE: Make sure to review this if there is ever type-directed dispatch,
                // like automatic solving of traits. It should be fine since it is strictly
                // after type checking, but care should be taken that it doesn't change which
                // impls are chosen.
                *binding.borrow_mut() =
                    TypeBinding::Bound(HirType::FieldElement(CompTime::No(None)));
                ast::Type::Field
            }

            HirType::Struct(def, args) => {
                let fields = def.borrow().get_fields(args);
                let fields = vecmap(fields, |(_, field)| Self::convert_type(&field));
                ast::Type::Tuple(fields)
            }

            HirType::Tuple(fields) => {
                let fields = vecmap(fields, Self::convert_type);
                ast::Type::Tuple(fields)
            }

            HirType::Function(args, ret) => {
                let args = vecmap(args, Self::convert_type);
                let ret = Box::new(Self::convert_type(ret));
                ast::Type::Function(args, ret)
            }

            HirType::Vec(element) => {
                let element = Self::convert_type(element);
                ast::Type::Vec(Box::new(element))
            }

            HirType::Forall(_, _) | HirType::Constant(_) | HirType::Error => {
                unreachable!("Unexpected type {} found", typ)
            }
        }
    }

    /// Converts arrays of structs (AOS) into structs of arrays (SOA).
    /// This is required since our SSA pass does not support arrays of structs.
    fn aos_to_soa_type(length: u64, element: ast::Type) -> ast::Type {
        match element {
            ast::Type::Field
            | ast::Type::Integer(_, _)
            | ast::Type::Bool
            | ast::Type::Unit
            | ast::Type::Function(_, _) => ast::Type::Array(length, Box::new(element)),

            ast::Type::Tuple(elements) => {
                ast::Type::Tuple(vecmap(elements, |typ| Self::aos_to_soa_type(length, typ)))
            }

            ast::Type::Array(_, _) | ast::Type::String(_) | ast::Type::Vec(_) => {
                unreachable!("Nested arrays and arrays of strings are not supported")
            }
        }
    }

    fn function_call(
        &mut self,
        call: HirCallExpression,
        id: node_interner::ExprId,
    ) -> ast::Expression {
        let func = Box::new(self.expr(call.func));
        let arguments = vecmap(&call.arguments, |id| self.expr(*id));
        let return_type = self.interner.id_type(id);
        let return_type = Self::convert_type(&return_type);
        let location = call.location;

        self.try_evaluate_call(&func, &call.arguments, &return_type)
            .unwrap_or(ast::Expression::Call(ast::Call { func, arguments, return_type, location }))
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
        arguments: &[node_interner::ExprId],
        result_type: &ast::Type,
    ) -> Option<ast::Expression> {
        if let ast::Expression::Ident(ident) = func {
            if let Definition::Builtin(opcode) = &ident.definition {
                if opcode == "array_len" {
                    let typ = self.interner.id_type(arguments[0]);
                    let len = typ.evaluate_to_u64().unwrap();
                    return Some(ast::Expression::Literal(ast::Literal::Integer(
                        (len as u128).into(),
                        ast::Type::Field,
                    )));
                } else if opcode == "modulus_num_bits" {
                    return Some(ast::Expression::Literal(ast::Literal::Integer(
                        (FieldElement::max_num_bits() as u128).into(),
                        ast::Type::Field,
                    )));
                } else if opcode == "zeroed" {
                    return Some(self.zeroed_value_of_type(result_type));
                }

                let modulus = FieldElement::modulus();

                if opcode == "modulus_le_bits" {
                    let bits = modulus.to_radix_le(2);
                    return Some(self.modulus_array_literal(bits, 1));
                } else if opcode == "modulus_be_bits" {
                    let bits = modulus.to_radix_be(2);
                    return Some(self.modulus_array_literal(bits, 1));
                } else if opcode == "modulus_be_bytes" {
                    let bytes = modulus.to_bytes_be();
                    return Some(self.modulus_array_literal(bytes, 8));
                } else if opcode == "modulus_le_bytes" {
                    let bytes = modulus.to_bytes_le();
                    return Some(self.modulus_array_literal(bytes, 8));
                }
            }
        }
        None
    }

    fn modulus_array_literal(&self, bytes: Vec<u8>, arr_elem_bits: u32) -> ast::Expression {
        let bytes_as_expr = vecmap(bytes, |byte| {
            ast::Expression::Literal(ast::Literal::Integer(
                (byte as u128).into(),
                ast::Type::Integer(crate::Signedness::Unsigned, arr_elem_bits),
            ))
        });
        let arr_literal = ast::ArrayLiteral {
            contents: bytes_as_expr,
            element_type: ast::Type::Integer(crate::Signedness::Unsigned, arr_elem_bits),
        };
        ast::Expression::Literal(ast::Literal::Array(arr_literal))
    }

    fn queue_function(
        &mut self,
        id: node_interner::FuncId,
        expr_id: node_interner::ExprId,
        function_type: HirType,
    ) -> FuncId {
        let new_id = self.next_function_id();
        self.define_global(id, function_type, new_id);

        let bindings = self.interner.get_instantiation_bindings(expr_id);
        let bindings = self.follow_bindings(bindings);

        self.queue.push_back((id, new_id, bindings));
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
        let (lvalue, index_lvalue) = self.lvalue(assign.lvalue);

        match index_lvalue {
            Some((index, element_type, location)) => {
                Self::aos_to_soa_assign(expression, Box::new(lvalue), index, element_type, location)
            }
            None => ast::Expression::Assign(ast::Assign { expression, lvalue }),
        }
    }

    /// Returns the lvalue along with an optional LValue::Index to add to the end, if needed.
    /// This is added to the end separately as part of converting arrays of structs to structs
    /// of arrays.
    fn lvalue(
        &mut self,
        lvalue: HirLValue,
    ) -> (ast::LValue, Option<(Box<ast::Expression>, ast::Type, Location)>) {
        match lvalue {
            HirLValue::Ident(ident, _) => {
                let lvalue = ast::LValue::Ident(self.local_ident(&ident).unwrap());
                (lvalue, None)
            }
            HirLValue::MemberAccess { object, field_index, .. } => {
                let field_index = field_index.unwrap();
                let (object, index) = self.lvalue(*object);
                let object = Box::new(object);
                let lvalue = ast::LValue::MemberAccess { object, field_index };
                (lvalue, index)
            }
            HirLValue::Index { array, index, typ } => {
                let location = self.interner.expr_location(&index);

                let (array, prev_index) = self.lvalue(*array);
                assert!(
                    prev_index.is_none(),
                    "Nested arrays are currently unsupported in noir: location is {location:?}"
                );

                let index = Box::new(self.expr(index));
                let element_type = Self::convert_type(&typ);
                (array, Some((index, element_type, location)))
            }
        }
    }

    fn aos_to_soa_assign(
        expression: Box<ast::Expression>,
        lvalue: Box<ast::LValue>,
        index: Box<ast::Expression>,
        typ: ast::Type,
        location: Location,
    ) -> ast::Expression {
        match typ {
            ast::Type::Tuple(fields) => {
                let fields = fields.into_iter().enumerate();
                ast::Expression::Block(vecmap(fields, |(i, field)| {
                    let expression = ast::Expression::ExtractTupleField(expression.clone(), i);
                    let lvalue =
                        ast::LValue::MemberAccess { object: lvalue.clone(), field_index: i };
                    Self::aos_to_soa_assign(
                        Box::new(expression),
                        Box::new(lvalue),
                        index.clone(),
                        field,
                        location,
                    )
                }))
            }

            // No changes if the element_type is not a tuple
            element_type => {
                let lvalue = ast::LValue::Index { array: lvalue, index, element_type, location };
                ast::Expression::Assign(ast::Assign { lvalue, expression })
            }
        }
    }

    fn lambda(&mut self, lambda: HirLambda) -> ast::Expression {
        let ret_type = Self::convert_type(&lambda.return_type);
        let lambda_name = "lambda";
        let parameter_types = vecmap(&lambda.parameters, |(_, typ)| Self::convert_type(typ));

        // Manually convert to Parameters type so we can reuse the self.parameters method
        let parameters = Parameters(vecmap(lambda.parameters, |(pattern, typ)| {
            Param(pattern, typ, noirc_abi::AbiVisibility::Private)
        }));

        let parameters = self.parameters(parameters);
        let body = self.expr(lambda.body);

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();
        let unconstrained = false;

        let function = ast::Function { id, name, parameters, body, return_type, unconstrained };
        self.push_function(id, function);

        let typ = ast::Type::Function(parameter_types, Box::new(ret_type));

        let name = lambda_name.to_owned();
        ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None,
            name,
            typ,
        })
    }

    /// Implements std::unsafe::zeroed by returning an appropriate zeroed
    /// ast literal or collection node for the given type. Note that for functions
    /// there is no obvious zeroed value so this should be considered unsafe to use.
    fn zeroed_value_of_type(&mut self, typ: &ast::Type) -> ast::Expression {
        match typ {
            ast::Type::Field | ast::Type::Integer(..) => {
                ast::Expression::Literal(ast::Literal::Integer(0_u128.into(), typ.clone()))
            }
            ast::Type::Bool => ast::Expression::Literal(ast::Literal::Bool(false)),
            // There is no unit literal currently. Replace it with 'false' since it should be ignored
            // anyway.
            ast::Type::Unit => ast::Expression::Literal(ast::Literal::Bool(false)),
            ast::Type::Array(length, element_type) => {
                let element = self.zeroed_value_of_type(element_type.as_ref());
                ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral {
                    contents: vec![element; *length as usize],
                    element_type: element_type.as_ref().clone(),
                }))
            }
            ast::Type::String(length) => {
                ast::Expression::Literal(ast::Literal::Str("\0".repeat(*length as usize)))
            }
            ast::Type::Tuple(fields) => {
                ast::Expression::Tuple(vecmap(fields, |field| self.zeroed_value_of_type(field)))
            }
            ast::Type::Function(parameter_types, ret_type) => {
                self.create_zeroed_function(parameter_types, ret_type)
            }
            ast::Type::Vec(_) => panic!("Cannot create a zeroed Vec value. This type is currently unimplemented and meant to be unusable outside of unconstrained functions"),
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
    ) -> ast::Expression {
        let lambda_name = "zeroed_lambda";

        let parameters = vecmap(parameter_types, |parameter_type| {
            (self.next_local_id(), false, "_".into(), parameter_type.clone())
        });

        let body = self.zeroed_value_of_type(ret_type);

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
            typ: ast::Type::Function(parameter_types.to_owned(), Box::new(ret_type.clone())),
        })
    }
}

fn unwrap_tuple_type(typ: &HirType) -> Vec<HirType> {
    match typ {
        HirType::Tuple(fields) => fields.clone(),
        HirType::TypeVariable(binding) => match &*binding.borrow() {
            TypeBinding::Bound(binding) => unwrap_tuple_type(binding),
            TypeBinding::Unbound(_) => unreachable!(),
        },
        other => unreachable!("unwrap_tuple_type: expected tuple, found {:?}", other),
    }
}

fn unwrap_struct_type(typ: &HirType) -> Vec<(String, HirType)> {
    match typ {
        HirType::Struct(def, args) => def.borrow().get_fields(args),
        HirType::TypeVariable(binding) => match &*binding.borrow() {
            TypeBinding::Bound(binding) => unwrap_struct_type(binding),
            TypeBinding::Unbound(_) => unreachable!(),
        },
        other => unreachable!("unwrap_struct_type: expected struct, found {:?}", other),
    }
}

fn perform_instantiation_bindings(bindings: &TypeBindings) {
    for (var, binding) in bindings.values() {
        *var.borrow_mut() = TypeBinding::Bound(binding.clone());
    }
}

fn undo_instantiation_bindings(bindings: TypeBindings) {
    for (id, (var, _)) in bindings {
        *var.borrow_mut() = TypeBinding::Unbound(id);
    }
}
