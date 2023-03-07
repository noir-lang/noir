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
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::{
    hir_def::{
        expr::*,
        function::{Param, Parameters},
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
    Program::new(functions, function_sig)
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
        let body = self.expr_infer(*self.interner.function(&f).as_expr());

        let function = ast::Function { id, name, parameters, body, return_type };
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

                for (name, field) in fields {
                    let typ = &struct_field_types[&name.0.contents];
                    self.parameter(field, typ, new_params);
                }
            }
        }
    }

    fn expr_infer(&mut self, expr: node_interner::ExprId) -> ast::Expression {
        let expected_type = self.interner.id_type(expr);
        self.expr(expr, &expected_type)
    }

    fn expr(&mut self, expr: node_interner::ExprId, typ: &HirType) -> ast::Expression {
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
            HirExpression::Literal(HirLiteral::Array(array)) => {
                let element_type = Self::convert_type(&self.interner.id_type(array[0]));
                let contents = vecmap(array, |id| self.expr_infer(id));
                Literal(Array(ast::ArrayLiteral { contents, element_type }))
            }
            HirExpression::Block(block) => self.block(block.0),

            HirExpression::Prefix(prefix) => ast::Expression::Unary(ast::Unary {
                operator: prefix.operator,
                rhs: Box::new(self.expr_infer(prefix.rhs)),
            }),

            HirExpression::Infix(infix) => {
                let lhs = Box::new(self.expr_infer(infix.lhs));
                let rhs = Box::new(self.expr_infer(infix.rhs));
                let operator = infix.operator.kind;
                ast::Expression::Binary(ast::Binary { lhs, rhs, operator })
            }

            HirExpression::Index(index) => ast::Expression::Index(ast::Index {
                collection: Box::new(self.expr_infer(index.collection)),
                index: Box::new(self.expr_infer(index.index)),
            }),

            HirExpression::MemberAccess(access) => {
                let field_index = self.interner.get_field_index(expr);
                let expr = Box::new(self.expr_infer(access.lhs));
                ast::Expression::ExtractTupleField(expr, field_index)
            }

            HirExpression::Call(call) => self.function_call(call, expr),

            HirExpression::Cast(cast) => ast::Expression::Cast(ast::Cast {
                lhs: Box::new(self.expr_infer(cast.lhs)),
                r#type: Self::convert_type(&cast.r#type),
            }),

            HirExpression::For(for_expr) => {
                let start = self.expr_infer(for_expr.start_range);
                let end = self.expr_infer(for_expr.end_range);
                let index_variable = self.next_local_id();
                self.define_local(for_expr.identifier.id, index_variable);

                let block = Box::new(self.expr_infer(for_expr.block));

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
                let cond = self.expr(if_expr.condition, &HirType::Bool(CompTime::No(None)));
                let then = self.expr(if_expr.consequence, typ);
                let else_ = if_expr.alternative.map(|alt| Box::new(self.expr(alt, typ)));
                ast::Expression::If(ast::If {
                    condition: Box::new(cond),
                    consequence: Box::new(then),
                    alternative: else_,
                    typ: Self::convert_type(&self.interner.id_type(expr)),
                })
            }

            HirExpression::Tuple(fields) => {
                let fields = vecmap(fields, |id| self.expr(id, typ));
                ast::Expression::Tuple(fields)
            }
            HirExpression::Constructor(constructor) => self.constructor(constructor, typ),

            HirExpression::Lambda(lambda) => self.lambda(lambda),

            HirExpression::MethodCall(_) | HirExpression::Error => unreachable!(),
        }
    }

    fn statement(&mut self, id: StmtId) -> ast::Expression {
        match self.interner.statement(&id) {
            HirStatement::Let(let_statement) => self.let_statement(let_statement),
            HirStatement::Constrain(constrain) => {
                let expr = self.expr(constrain.0, &HirType::Bool(CompTime::No(None)));
                let location = self.interner.expr_location(&constrain.0);
                ast::Expression::Constrain(Box::new(expr), location)
            }
            HirStatement::Assign(assign) => self.assign(assign),
            HirStatement::Expression(expr) => self.expr_infer(expr),
            HirStatement::Semi(expr) => ast::Expression::Semi(Box::new(self.expr_infer(expr))),
            HirStatement::Error => unreachable!(),
        }
    }

    fn let_statement(&mut self, let_statement: HirLetStatement) -> ast::Expression {
        let expr = self.expr_infer(let_statement.expression);
        let expected_type = self.interner.id_type(let_statement.expression);
        self.unpack_pattern(let_statement.pattern, expr, &expected_type)
    }

    fn constructor(
        &mut self,
        constructor: HirConstructorExpression,
        typ: &HirType,
    ) -> ast::Expression {
        let field_types = unwrap_struct_type(typ);

        // Create let bindings for each field value first to preserve evaluation order before
        // they are reordered and packed into the resulting tuple
        let mut field_vars = BTreeMap::new();
        let mut new_exprs = Vec::with_capacity(constructor.fields.len());

        for (field_name, expr_id) in constructor.fields {
            let new_id = self.next_local_id();
            let field_type = field_types.get(&field_name.0.contents).unwrap();
            let typ = Self::convert_type(field_type);

            field_vars.insert(field_name.0.contents.clone(), (new_id, typ));
            let expression = Box::new(self.expr(expr_id, field_type));

            new_exprs.push(ast::Expression::Let(ast::Let {
                id: new_id,
                mutable: false,
                name: field_name.0.contents,
                expression,
            }));
        }

        let sorted_fields = vecmap(field_vars, |(name, (id, typ))| {
            let definition = Definition::Local(id);
            let mutable = false;
            ast::Expression::Ident(ast::Ident { definition, mutable, location: None, name, typ })
        });

        // Finally we can return the created Tuple from the new block
        new_exprs.push(ast::Expression::Tuple(sorted_fields));
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
                // We map each pattern to its respective field in a BTreeMap
                // Fields in struct types are ordered, and doing this map guarantees we extract the correct field index
                let patterns_map = btree_map(patterns, |(ident, pattern)| {
                    let typ = fields[&ident.0.contents].clone();
                    (ident.0.contents, (pattern, typ))
                });
                self.unpack_tuple_pattern(value, patterns_map.into_values())
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
        match definition.kind {
            DefinitionKind::Function(func_id) => {
                let mutable = definition.mutable;
                let location = Some(ident.location);
                let name = definition.name.clone();
                let typ = self.interner.id_type(expr_id);

                let definition = self.lookup_function(func_id, expr_id, &typ);
                let typ = Self::convert_type(&typ);
                let ident = ast::Ident { location, mutable, definition, name, typ };
                ast::Expression::Ident(ident)
            }
            DefinitionKind::Global(expr_id) => self.expr_infer(expr_id),
            DefinitionKind::Local(_) => {
                let ident = self.local_ident(&ident).unwrap();
                ast::Expression::Ident(ident)
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

            HirType::Array(size, element) => {
                let size = size.evaluate_to_u64().unwrap_or(0);
                let element = Self::convert_type(element.as_ref());
                ast::Type::Array(size, Box::new(element))
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

            HirType::Forall(_, _) | HirType::Constant(_) | HirType::Error => {
                unreachable!("Unexpected type {} found", typ)
            }
        }
    }

    fn function_call(
        &mut self,
        call: HirCallExpression,
        id: node_interner::ExprId,
    ) -> ast::Expression {
        let func = Box::new(self.expr_infer(call.func));
        let arguments = vecmap(&call.arguments, |id| self.expr_infer(*id));
        let return_type = self.interner.id_type(id);
        let return_type = Self::convert_type(&return_type);
        let location = call.location;

        self.try_evaluate_call(&func, &call.arguments).unwrap_or(ast::Expression::Call(ast::Call {
            func,
            arguments,
            return_type,
            location,
        }))
    }

    /// Try to evaluate certain builtin functions (currently only 'array_len' and field modulus methods)
    /// at their call site.
    /// NOTE: Evaluating at the call site means we cannot track aliased functions.
    ///       E.g. `let f = std::array::len; f(arr)` will fail to evaluate.
    ///       To fix this we need to evaluate on the identifier instead, which
    ///       requires us to evaluate to a Lambda value which isn't in noir yet.
    fn try_evaluate_call(
        &self,
        func: &ast::Expression,
        arguments: &[node_interner::ExprId],
    ) -> Option<ast::Expression> {
        match func {
            ast::Expression::Ident(ident) => match &ident.definition {
                Definition::Builtin(opcode) if opcode == "array_len" => {
                    let typ = self.interner.id_type(arguments[0]);
                    let len = typ.evaluate_to_u64().unwrap();
                    Some(ast::Expression::Literal(ast::Literal::Integer(
                        (len as u128).into(),
                        ast::Type::Field,
                    )))
                }
                Definition::Builtin(opcode) if opcode == "modulus_num_bits" => {
                    Some(ast::Expression::Literal(ast::Literal::Integer(
                        (FieldElement::max_num_bits() as u128).into(),
                        ast::Type::Field,
                    )))
                }
                Definition::Builtin(opcode) if opcode == "modulus_le_bits" => {
                    let modulus = FieldElement::modulus();
                    let bits = modulus.to_radix_le(2);
                    Some(self.modulus_array_literal(bits, 1))
                }
                Definition::Builtin(opcode) if opcode == "modulus_be_bits" => {
                    let modulus = FieldElement::modulus();
                    let bits = modulus.to_radix_be(2);
                    Some(self.modulus_array_literal(bits, 1))
                }
                Definition::Builtin(opcode) if opcode == "modulus_be_bytes" => {
                    let modulus = FieldElement::modulus();
                    let bytes = modulus.to_bytes_be();
                    Some(self.modulus_array_literal(bytes, 8))
                }
                Definition::Builtin(opcode) if opcode == "modulus_le_bytes" => {
                    let modulus = FieldElement::modulus();
                    let bytes = modulus.to_bytes_le();
                    Some(self.modulus_array_literal(bytes, 8))
                }
                _ => None,
            },
            _ => None,
        }
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
        let expression = Box::new(self.expr_infer(assign.expression));
        let lvalue = self.lvalue(assign.lvalue);
        ast::Expression::Assign(ast::Assign { lvalue, expression })
    }

    fn lvalue(&mut self, lvalue: HirLValue) -> ast::LValue {
        match lvalue {
            HirLValue::Ident(ident, _) => ast::LValue::Ident(self.local_ident(&ident).unwrap()),
            HirLValue::MemberAccess { object, field_index, .. } => {
                let object = Box::new(self.lvalue(*object));
                ast::LValue::MemberAccess { object, field_index: field_index.unwrap() }
            }
            HirLValue::Index { array, index, .. } => {
                let array = Box::new(self.lvalue(*array));
                let index = Box::new(self.expr_infer(index));
                ast::LValue::Index { array, index }
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
        let body = self.expr_infer(lambda.body);

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();

        let function = ast::Function { id, name, parameters, body, return_type };
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

fn unwrap_struct_type(typ: &HirType) -> BTreeMap<String, HirType> {
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
