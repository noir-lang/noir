use std::{rc::Rc, collections::{BTreeMap, BTreeSet}};

use crate::{macros_api::{NoirFunction, Expression, BlockExpression, Statement, StatementKind, ExpressionKind, NodeInterner, HirExpression, HirStatement, StructId, Literal, PrefixExpression, IndexExpression, CallExpression, MethodCallExpression, MemberAccessExpression, CastExpression, HirLiteral}, node_interner::{FuncId, StmtId, ExprId, DependencyId, TraitId, DefinitionKind}, ast::{FunctionKind, UnresolvedTraitConstraint, ConstructorExpression, InfixExpression, IfExpression, Lambda, ArrayLiteral, UnresolvedTypeExpression}, hir_def::{expr::{HirBlockExpression, HirIdent, HirArrayLiteral, HirLambda, HirIfExpression, HirPrefixExpression, HirIndexExpression, HirCallExpression}, traits::TraitConstraint}, hir::{resolution::{errors::ResolverError, path_resolver::PathResolver, resolver::LambdaContext}, def_collector::dc_crate::CompilationError, type_check::TypeCheckError, scope::ScopeForest as GenericScopeForest}, Type, TypeVariable};
use crate::graph::CrateId;
use crate::hir::def_map::CrateDefMap;

mod scope;
mod types;
mod patterns;
mod statements;

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Span, Location};
use regex::Regex;
use scope::Scope;

/// ResolverMetas are tagged onto each definition to track how many times they are used
#[derive(Debug, PartialEq, Eq)]
struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

type ScopeForest = GenericScopeForest<String, ResolverMeta>;

struct Elaborator {
    scopes: ScopeForest,
    globals: Scope,
    local_scopes: Vec<Scope>,

    errors: Vec<CompilationError>,

    interner: NodeInterner,
    file: FileId,

    in_unconstrained_fn: bool,
    nested_loops: usize,

    /// True if the current module is a contract.
    /// This is usually determined by self.path_resolver.module_id(), but it can
    /// be overridden for impls. Impls are an odd case since the methods within resolve
    /// as if they're in the parent module, but should be placed in a child module.
    /// Since they should be within a child module, in_contract is manually set to false
    /// for these so we can still resolve them in the parent module without them being in a contract.
    in_contract: bool,

    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    generics: Vec<(Rc<String>, TypeVariable, Span)>,

    /// When resolving lambda expressions, we need to keep track of the variables
    /// that are captured. We do this in order to create the hidden environment
    /// parameter for the lambda function.
    lambda_stack: Vec<LambdaContext>,

    /// Set to the current type if we're resolving an impl
    self_type: Option<Type>,

    /// The current dependency item we're resolving.
    /// Used to link items to their dependencies in the dependency graph
    current_item: Option<DependencyId>,

    trait_id: Option<TraitId>,

    path_resolver: Rc<dyn PathResolver>,
    def_maps: BTreeMap<CrateId, CrateDefMap>,

    /// In-resolution names
    ///
    /// This needs to be a set because we can have multiple in-resolution
    /// names when resolving structs that are declared in reverse order of their
    /// dependencies, such as in the following case:
    ///
    /// ```
    /// struct Wrapper {
    ///     value: Wrapped
    /// }
    /// struct Wrapped {
    /// }
    /// ```
    resolving_ids: BTreeSet<StructId>,

    trait_bounds: Vec<UnresolvedTraitConstraint>,

    /// All type variables created in the current function.
    /// This map is used to default any integer type variables at the end of
    /// a function (before checking trait constraints) if a type wasn't already chosen.
    type_variables: Vec<Type>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    trait_constraints: Vec<(TraitConstraint, ExprId)>,
}

impl Elaborator {
    fn elaborate_function(&mut self, function: NoirFunction, id: FuncId) {
        match function.kind {
            FunctionKind::LowLevel => todo!(),
            FunctionKind::Builtin => todo!(),
            FunctionKind::Oracle => todo!(),
            FunctionKind::Recursive => todo!(),
            FunctionKind::Normal => {
                let _body = self.elaborate_block(function.def.body);
            },
        }
    }

    fn elaborate_expression(&mut self, expr: Expression) -> (ExprId, Type) {
        let (hir_expr, typ) = match expr.kind {
            ExpressionKind::Literal(literal) => self.elaborate_literal(literal, expr.span),
            ExpressionKind::Block(block) => self.elaborate_block(block),
            ExpressionKind::Prefix(prefix) => self.elaborate_prefix(*prefix),
            ExpressionKind::Index(index) => self.elaborate_index(*index),
            ExpressionKind::Call(call) => self.elaborate_call(*call, expr.span),
            ExpressionKind::MethodCall(methodCall) => self.elaborate_method_call(*methodCall),
            ExpressionKind::Constructor(constructor) => self.elaborate_constructor(*constructor),
            ExpressionKind::MemberAccess(memberAccess) => self.elaborate_member_access(*memberAccess),
            ExpressionKind::Cast(cast) => self.elaborate_cast(*cast),
            ExpressionKind::Infix(infix) => self.elaborate_infix(*infix),
            ExpressionKind::If(if_) => self.elaborate_if(*if_),
            ExpressionKind::Variable(variable) => return self.elaborate_variable(variable),
            ExpressionKind::Tuple(tuple) => self.elaborate_tuple(tuple),
            ExpressionKind::Lambda(lambda) => self.elaborate_lambda(*lambda),
            ExpressionKind::Parenthesized(expr) => return self.elaborate_expression(*expr),
            ExpressionKind::Quote(quote) => self.elaborate_quote(quote),
            ExpressionKind::Comptime(comptime) => self.elaborate_comptime_block(comptime),
            ExpressionKind::Error => (HirExpression::Error, Type::Error),
        };
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, expr.span, self.file);
        self.interner.push_expr_type(id, typ.clone());
        (id, typ)
    }

    fn elaborate_statement_value(&mut self, statement: Statement) -> (HirStatement, Type) {
        match statement.kind {
            StatementKind::Let(let_stmt) => self.elaborate_let(let_stmt),
            StatementKind::Constrain(constrain) => self.elaborate_constrain(constrain),
            StatementKind::Assign(assign) => self.elaborate_assign(assign),
            StatementKind::For(for_stmt) => self.elaborate_for(for_stmt),
            StatementKind::Break => self.elaborate_jump(true, statement.span),
            StatementKind::Continue => self.elaborate_jump(false, statement.span),
            StatementKind::Comptime(statement) => self.elaborate_comptime(*statement),
            StatementKind::Expression(expr) => {
                let (expr, typ) = self.elaborate_expression(expr);
                (HirStatement::Expression(expr), typ)
            },
            StatementKind::Semi(expr) => {
                let (expr, _typ) = self.elaborate_expression(expr);
                (HirStatement::Semi(expr), Type::Unit)
            }
            StatementKind::Error => (HirStatement::Error, Type::Error),
        }
    }

    fn elaborate_statement(&mut self, statement: Statement) -> (StmtId, Type) {
        let (hir_statement, typ) = self.elaborate_statement_value(statement);
        let id = self.interner.push_stmt(hir_statement);
        self.interner.push_stmt_location(id, statement.span, self.file);
        (id, typ)
    }

    fn elaborate_block(&mut self, block: BlockExpression) -> (HirExpression, Type) {
        self.push_scope();
        let mut block_type = Type::Unit;
        let mut statements = Vec::with_capacity(block.statements.len());

        for (i, statement) in block.statements.into_iter().enumerate() {
            let (id, stmt_type) = self.elaborate_statement(statement);

            if let HirStatement::Semi(expr) = self.interner.statement(&id) {
                let inner_expr_type = self.interner.id_type(expr);
                let span = self.interner.expr_span(&expr);

                self.unify(&inner_expr_type, &Type::Unit, || TypeCheckError::UnusedResultError {
                    expr_type: inner_expr_type.clone(),
                    expr_span: span,
                });

                if i + 1 == statements.len() {
                    block_type = stmt_type;
                }
            }
        }

        self.pop_scope();
        (HirExpression::Block(HirBlockExpression { statements }), block_type)
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn elaborate_jump(&mut self, is_break: bool, span: noirc_errors::Span) -> (HirStatement, Type) {
        if !self.in_unconstrained_fn {
            self.push_err(ResolverError::JumpInConstrainedFn { is_break, span });
        }
        if self.nested_loops == 0 {
            self.push_err(ResolverError::JumpOutsideLoop { is_break, span });
        }

        let expr = if is_break {
            HirStatement::Break
        } else {
            HirStatement::Continue
        };
        (expr, self.interner.next_type_variable())
    }

    fn push_err(&mut self, error: impl Into<CompilationError>) {
        self.errors.push(error.into());
    }

    fn elaborate_literal(&mut self, literal: Literal, span: Span) -> (HirExpression, Type) {
        use HirExpression::Literal as Lit;
        match literal {
            Literal::Unit => (Lit(HirLiteral::Unit), Type::Unit),
            Literal::Bool(b) => (Lit(HirLiteral::Bool(b)), Type::Bool),
            Literal::Integer(integer, sign) => {
                let int = HirLiteral::Integer(integer, sign);
                (Lit(int), self.polymorphic_integer_or_field())
            }
            Literal::Str(str) | Literal::RawStr(str, _) => {
                let len = Type::Constant(str.len() as u64);
                (Lit(HirLiteral::Str(str)), Type::String(Box::new(len)))
            }
            Literal::FmtStr(str) => self.elaborate_fmt_string(str, span),
            Literal::Array(array_literal) => self.elaborate_array_literal(array_literal, span, true),
            Literal::Slice(array_literal) => self.elaborate_array_literal(array_literal, span, false),
        }
    }

    fn elaborate_array_literal(&mut self, array_literal: ArrayLiteral, span: Span, is_array: bool) -> (HirExpression, Type) {
        let (expr, elem_type, length) = match array_literal {
            ArrayLiteral::Standard(elements) => {
                let mut first_elem_type = self.interner.next_type_variable();
                let first_span = elements.first().map(|elem| elem.span).unwrap_or(span);

                let elements = vecmap(elements.into_iter().enumerate(), |(i, elem)| {
                    let span = elem.span;
                    let (elem_id, elem_type) = self.elaborate_expression(elem);

                    self.unify(&elem_type, &first_elem_type, || {
                        TypeCheckError::NonHomogeneousArray {
                            first_span,
                            first_type: first_elem_type.to_string(),
                            first_index: 0,
                            second_span: span,
                            second_type: elem_type.to_string(),
                            second_index: i,
                        }
                        .add_context("elements in an array must have the same type")
                    });
                    elem_id
                });

                let length = Type::Constant(elements.len() as u64);
                (HirArrayLiteral::Standard(elements), first_elem_type, length)
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                let span = length.span;
                let length =
                    UnresolvedTypeExpression::from_expr(*length, span).unwrap_or_else(|error| {
                        self.push_err(ResolverError::ParserError(Box::new(error)));
                        UnresolvedTypeExpression::Constant(0, span)
                    });

                let length = self.convert_expression_type(length);
                let (repeated_element, elem_type) = self.elaborate_expression(*repeated_element);

                let length_clone = length.clone();
                (HirArrayLiteral::Repeated { repeated_element, length }, elem_type, length_clone)
            }
        };
        let constructor = if is_array { HirLiteral::Array } else { HirLiteral::Slice };
        let elem_type = Box::new(elem_type);
        let typ = if is_array { Type::Array(Box::new(length), elem_type) } else { Type::Slice(elem_type) };
        (HirExpression::Literal(constructor(expr)), typ)
    }

    fn elaborate_fmt_string(&mut self, str: String, call_expr_span: Span) -> (HirExpression, Type) {
        let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}")
            .expect("ICE: an invalid regex pattern was used for checking format strings");

        let mut fmt_str_idents = Vec::new();
        let mut capture_types = Vec::new();

        for field in re.find_iter(&str) {
            let matched_str = field.as_str();
            let ident_name = &matched_str[1..(matched_str.len() - 1)];

            let scope_tree = self.scopes.current_scope_tree();
            let variable = scope_tree.find(ident_name);
            if let Some((old_value, _)) = variable {
                old_value.num_times_used += 1;
                let ident = HirExpression::Ident(old_value.ident.clone());
                let expr_id = self.interner.push_expr(ident);
                self.interner.push_expr_location(expr_id, call_expr_span, self.file);
                fmt_str_idents.push(expr_id);
            } else if ident_name.parse::<usize>().is_ok() {
                self.push_err(ResolverError::NumericConstantInFormatString {
                    name: ident_name.to_owned(),
                    span: call_expr_span,
                });
            } else {
                self.push_err(ResolverError::VariableNotDeclared {
                    name: ident_name.to_owned(),
                    span: call_expr_span,
                });
            }
        }

        let len = Type::Constant(str.len() as u64);
        let typ = Type::FmtString(Box::new(len), Box::new(Type::Tuple(capture_types)));
        (HirExpression::Literal(HirLiteral::FmtStr(str, fmt_str_idents)), typ)
    }

    fn elaborate_prefix(&mut self, prefix: PrefixExpression) -> (HirExpression, Type) {
        let span = prefix.rhs.span;
        let (rhs, rhs_type) = self.elaborate_expression(prefix.rhs);
        let ret_type = self.type_check_prefix_operand(&prefix.operator, &rhs_type, span);
        (HirExpression::Prefix(HirPrefixExpression { operator: prefix.operator, rhs  }), ret_type)
    }

    fn elaborate_index(&mut self, index_expr: IndexExpression) -> (HirExpression, Type) {
        let (index, index_type) = self.elaborate_expression(index_expr.index);
        let span = index_expr.index.span;

        let expected = self.polymorphic_integer_or_field();
        self.unify(&index_type, &expected, || {
            TypeCheckError::TypeMismatch {
                expected_typ: "an integer".to_owned(),
                expr_typ: index_type.to_string(),
                expr_span: span,
            }
        });

        // When writing `a[i]`, if `a : &mut ...` then automatically dereference `a` as many
        // times as needed to get the underlying array.
        let lhs_span = index_expr.collection.span;
        let (lhs, lhs_type) = self.elaborate_expression(index_expr.collection);
        let (collection, lhs_type) = self.insert_auto_dereferences(lhs, lhs_type);

        let typ = match lhs_type.follow_bindings() {
            // XXX: We can check the array bounds here also, but it may be better to constant fold first
            // and have ConstId instead of ExprId for constants
            Type::Array(_, base_type) => *base_type,
            Type::Slice(base_type) => *base_type,
            Type::Error => Type::Error,
            typ => {
                self.push_err(TypeCheckError::TypeMismatch {
                    expected_typ: "Array".to_owned(),
                    expr_typ: typ.to_string(),
                    expr_span: lhs_span,
                });
                Type::Error
            }
        };

        let expr = HirExpression::Index(HirIndexExpression { collection, index });
        (expr, typ)
    }

    fn elaborate_call(&mut self, call: CallExpression, span: Span) -> (HirExpression, Type) {
        // Get the span and name of path for error reporting
        let (func, func_type) = self.elaborate_expression(*call.func);

        let arguments = vecmap(call.arguments, |arg| self.elaborate_expression(arg));
        let location = Location::new(span, self.file);
        let expr = HirExpression::Call(HirCallExpression { func, arguments, location });
        (expr, typ)


        // Need to setup these flags here as `self` is borrowed mutably to type check the rest of the call expression
        // These flags are later used to type check calls to unconstrained functions from constrained functions
        let current_func = self.current_function;
        let func_mod = current_func.map(|func| self.interner.function_modifiers(&func));
        let is_current_func_constrained =
            func_mod.map_or(true, |func_mod| !func_mod.is_unconstrained);
        let is_unconstrained_call = self.is_unconstrained_call(&call_expr.func);

        self.check_if_deprecated(&call_expr.func);

        let function = self.check_expression(&call_expr.func);

        let args = vecmap(&call_expr.arguments, |arg| {
            let typ = self.check_expression(arg);
            (typ, *arg, self.interner.expr_span(arg))
        });

        // Check that we are not passing a mutable reference from a constrained runtime to an unconstrained runtime
        if is_current_func_constrained && is_unconstrained_call {
            for (typ, _, _) in args.iter() {
                if matches!(&typ.follow_bindings(), Type::MutableReference(_)) {
                    self.errors.push(TypeCheckError::ConstrainedReferenceToUnconstrained {
                        span: self.interner.expr_span(expr_id),
                    });
                    return Type::Error;
                }
            }
        }

        let span = self.interner.expr_span(expr_id);
        let return_type = self.bind_function_type(function, args, span);

        // Check that we are not passing a slice from an unconstrained runtime to a constrained runtime
        if is_current_func_constrained && is_unconstrained_call {
            if return_type.contains_slice() {
                self.errors.push(TypeCheckError::UnconstrainedSliceReturnToConstrained {
                    span: self.interner.expr_span(expr_id),
                });
                return Type::Error;
            } else if matches!(&return_type.follow_bindings(), Type::MutableReference(_)) {
                self.errors.push(TypeCheckError::UnconstrainedReferenceToConstrained {
                    span: self.interner.expr_span(expr_id),
                });
                return Type::Error;
            }
        };

        return_type
    }

    fn elaborate_method_call(&mut self, method_call: MethodCallExpression) -> (HirExpression, Type) {
        todo!()
    }

    fn elaborate_constructor(&mut self, constructor: ConstructorExpression) -> (HirExpression, Type) {
        todo!()
    }

    fn elaborate_member_access(&mut self, member_access: MemberAccessExpression) -> (HirExpression, Type) {
        todo!()
    }

    fn elaborate_cast(&mut self, cast: CastExpression) -> (HirExpression, Type) {
        todo!()
    }

    fn elaborate_infix(&mut self, infix: InfixExpression) -> (HirExpression, Type) {
        todo!()
    }

    fn elaborate_if(&mut self, if_expr: IfExpression) -> (HirExpression, Type) {
        let expr_span = if_expr.condition.span;
        let (condition, cond_type) = self.elaborate_expression(if_expr.condition);
        let (consequence, mut ret_type) = self.elaborate_expression(if_expr.consequence);

        self.unify(&cond_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool.to_string(),
            expr_typ: cond_type.to_string(),
            expr_span,
        });
        
        let alternative = if_expr.alternative.map(|alternative| {
            let expr_span = alternative.span;
            let (else_, else_type) = self.elaborate_expression(alternative);

            self.unify(&ret_type, &else_type, || {
                let err = TypeCheckError::TypeMismatch {
                    expected_typ: ret_type.to_string(),
                    expr_typ: else_type.to_string(),
                    expr_span,
                };

                let context = if ret_type == Type::Unit {
                    "Are you missing a semicolon at the end of your 'else' branch?"
                } else if else_type == Type::Unit {
                    "Are you missing a semicolon at the end of the first block of this 'if'?"
                } else {
                    "Expected the types of both if branches to be equal"
                };

                err.add_context(context)
            });
            else_
        });

        if alternative.is_none() {
            ret_type = Type::Unit;
        }

        let if_expr = HirIfExpression { condition, consequence, alternative };
        (HirExpression::If(if_expr), ret_type)
    }

    fn elaborate_tuple(&mut self, tuple: Vec<Expression>) -> (HirExpression, Type) {
        let mut element_ids = Vec::with_capacity(tuple.len());
        let mut element_types = Vec::with_capacity(tuple.len());

        for element in tuple {
            let (id, typ) = self.elaborate_expression(element);
            element_ids.push(id);
            element_types.push(typ);
        }

        (HirExpression::Tuple(element_ids), Type::Tuple(element_types))
    }

    fn elaborate_lambda(&mut self, lambda: Lambda) -> (HirExpression, Type) {
        self.push_scope();
        let scope_index = self.scopes.current_scope_index();

        self.lambda_stack.push(LambdaContext { captures: Vec::new(), scope_index });

        let mut arg_types = Vec::with_capacity(lambda.parameters.len());
        let parameters = vecmap(lambda.parameters, |(pattern, typ)| {
            let parameter = DefinitionKind::Local(None);
            let typ = self.resolve_inferred_type(typ);
            arg_types.push(typ.clone());
            (self.elaborate_pattern(pattern, typ.clone(), parameter), typ)
        });

        let return_type = self.resolve_inferred_type(lambda.return_type);
        let body_span = lambda.body.span;
        let (body, body_type) = self.elaborate_expression(lambda.body);

        let lambda_context = self.lambda_stack.pop().unwrap();
        self.pop_scope();

        self.unify(&body_type, &return_type, || TypeCheckError::TypeMismatch {
            expected_typ: return_type.to_string(),
            expr_typ: body_type.to_string(),
            expr_span: body_span,
        });

        let captured_vars = vecmap(&lambda_context.captures, |capture| {
            self.interner.definition_type(capture.ident.id)
        });

        let env_type =
            if captured_vars.is_empty() { Type::Unit } else { Type::Tuple(captured_vars) };

        let captures = lambda_context.captures;
        let expr = HirExpression::Lambda(HirLambda { parameters, return_type, body, captures });
        (expr, Type::Function(arg_types, Box::new(body_type), Box::new(env_type)))
    }

    fn elaborate_quote(&mut self, block: BlockExpression) -> (HirExpression, Type) {
        (HirExpression::Quote(block), Type::Code)
    }

    fn elaborate_comptime_block(&mut self, comptime: BlockExpression) -> (HirExpression, Type) {
        todo!("Elaborate comptime block")
    }
}
