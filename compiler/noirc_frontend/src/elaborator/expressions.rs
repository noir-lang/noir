use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, CastExpression, ConstructorExpression,
        Expression, ExpressionKind, Ident, IfExpression, IndexExpression, InfixExpression,
        ItemVisibility, Lambda, Literal, MatchExpression, MemberAccessExpression,
        MethodCallExpression, Path, PathSegment, PrefixExpression, StatementKind, UnaryOp,
        UnresolvedTypeData, UnresolvedTypeExpression,
    },
    hir::{
        comptime::{self, InterpreterError},
        resolution::{
            errors::ResolverError, import::PathResolutionError, visibility::method_call_is_visible,
        },
        type_check::{generics::TraitGenerics, TypeCheckError},
    },
    hir_def::{
        expr::{
            HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirExpression, HirIdent, HirIfExpression, HirIndexExpression,
            HirInfixExpression, HirLambda, HirLiteral, HirMemberAccess, HirMethodCallExpression,
            HirPrefixExpression,
        },
        stmt::{HirLetStatement, HirPattern, HirStatement},
        traits::{ResolvedTraitBound, TraitConstraint},
    },
    node_interner::{
        DefinitionId, DefinitionKind, ExprId, FuncId, InternedStatementKind, StmtId, TraitMethodId,
    },
    token::{FmtStrFragment, Tokens},
    DataType, Kind, QuotedType, Shared, Type,
};

use super::{Elaborator, LambdaContext, UnsafeBlockStatus};

impl<'context> Elaborator<'context> {
    pub(crate) fn elaborate_expression(&mut self, expr: Expression) -> (ExprId, Type) {
        self.elaborate_expression_with_target_type(expr, None)
    }

    pub(crate) fn elaborate_expression_with_target_type(
        &mut self,
        expr: Expression,
        target_type: Option<&Type>,
    ) -> (ExprId, Type) {
        let (hir_expr, typ) = match expr.kind {
            ExpressionKind::Literal(literal) => self.elaborate_literal(literal, expr.span),
            ExpressionKind::Block(block) => self.elaborate_block(block, target_type),
            ExpressionKind::Prefix(prefix) => return self.elaborate_prefix(*prefix, expr.span),
            ExpressionKind::Index(index) => self.elaborate_index(*index),
            ExpressionKind::Call(call) => self.elaborate_call(*call, expr.span),
            ExpressionKind::MethodCall(call) => self.elaborate_method_call(*call, expr.span),
            ExpressionKind::Constructor(constructor) => self.elaborate_constructor(*constructor),
            ExpressionKind::MemberAccess(access) => {
                return self.elaborate_member_access(*access, expr.span)
            }
            ExpressionKind::Cast(cast) => self.elaborate_cast(*cast, expr.span),
            ExpressionKind::Infix(infix) => return self.elaborate_infix(*infix, expr.span),
            ExpressionKind::If(if_) => self.elaborate_if(*if_, target_type),
            ExpressionKind::Match(match_) => self.elaborate_match(*match_, expr.span),
            ExpressionKind::Variable(variable) => return self.elaborate_variable(variable),
            ExpressionKind::Tuple(tuple) => self.elaborate_tuple(tuple, target_type),
            ExpressionKind::Lambda(lambda) => {
                self.elaborate_lambda_with_target_type(*lambda, target_type)
            }
            ExpressionKind::Parenthesized(expr) => {
                return self.elaborate_expression_with_target_type(*expr, target_type)
            }
            ExpressionKind::Quote(quote) => self.elaborate_quote(quote, expr.span),
            ExpressionKind::Comptime(comptime, _) => {
                return self.elaborate_comptime_block(comptime, expr.span, target_type)
            }
            ExpressionKind::Unsafe(block_expression, span) => {
                self.elaborate_unsafe_block(block_expression, span, target_type)
            }
            ExpressionKind::Resolved(id) => return (id, self.interner.id_type(id)),
            ExpressionKind::Interned(id) => {
                let expr_kind = self.interner.get_expression_kind(id);
                let expr = Expression::new(expr_kind.clone(), expr.span);
                return self.elaborate_expression(expr);
            }
            ExpressionKind::InternedStatement(id) => {
                return self.elaborate_interned_statement_as_expr(id, expr.span);
            }
            ExpressionKind::Error => (HirExpression::Error, Type::Error),
            ExpressionKind::Unquote(_) => {
                self.push_err(ResolverError::UnquoteUsedOutsideQuote { span: expr.span });
                (HirExpression::Error, Type::Error)
            }
            ExpressionKind::AsTraitPath(_) => {
                self.push_err(ResolverError::UnquoteUsedOutsideQuote { span: expr.span });
                (HirExpression::Error, Type::Error)
            }
            ExpressionKind::TypePath(path) => return self.elaborate_type_path(path),
        };
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, expr.span, self.file);
        self.interner.push_expr_type(id, typ.clone());
        (id, typ)
    }

    fn elaborate_interned_statement_as_expr(
        &mut self,
        id: InternedStatementKind,
        span: Span,
    ) -> (ExprId, Type) {
        match self.interner.get_statement_kind(id) {
            StatementKind::Expression(expr) | StatementKind::Semi(expr) => {
                self.elaborate_expression(expr.clone())
            }
            StatementKind::Interned(id) => self.elaborate_interned_statement_as_expr(*id, span),
            StatementKind::Error => {
                let expr = Expression::new(ExpressionKind::Error, span);
                self.elaborate_expression(expr)
            }
            other => {
                let statement = other.to_string();
                self.push_err(ResolverError::InvalidInternedStatementInExpr { statement, span });
                let expr = Expression::new(ExpressionKind::Error, span);
                self.elaborate_expression(expr)
            }
        }
    }

    pub(super) fn elaborate_block(
        &mut self,
        block: BlockExpression,
        target_type: Option<&Type>,
    ) -> (HirExpression, Type) {
        let (block, typ) = self.elaborate_block_expression(block, target_type);
        (HirExpression::Block(block), typ)
    }

    fn elaborate_block_expression(
        &mut self,
        block: BlockExpression,
        target_type: Option<&Type>,
    ) -> (HirBlockExpression, Type) {
        self.push_scope();
        let mut block_type = Type::Unit;
        let statements_len = block.statements.len();
        let mut statements = Vec::with_capacity(statements_len);

        for (i, statement) in block.statements.into_iter().enumerate() {
            let statement_target_type = if i == statements_len - 1 { target_type } else { None };
            let (id, stmt_type) =
                self.elaborate_statement_with_target_type(statement, statement_target_type);
            statements.push(id);

            if let HirStatement::Semi(expr) = self.interner.statement(&id) {
                let inner_expr_type = self.interner.id_type(expr);
                let span = self.interner.expr_span(&expr);

                self.unify(&inner_expr_type, &Type::Unit, || TypeCheckError::UnusedResultError {
                    expr_type: inner_expr_type.clone(),
                    expr_span: span,
                });
            }

            if i + 1 == statements.len() {
                block_type = stmt_type;
            }
        }

        self.pop_scope();
        (HirBlockExpression { statements }, block_type)
    }

    fn elaborate_unsafe_block(
        &mut self,
        block: BlockExpression,
        span: Span,
        target_type: Option<&Type>,
    ) -> (HirExpression, Type) {
        // Before entering the block we cache the old value of `in_unsafe_block` so it can be restored.
        let old_in_unsafe_block = self.unsafe_block_status;
        let is_nested_unsafe_block =
            !matches!(old_in_unsafe_block, UnsafeBlockStatus::NotInUnsafeBlock);
        if is_nested_unsafe_block {
            let span = Span::from(span.start()..span.start() + 6); // Only highlight the `unsafe` keyword
            self.push_err(TypeCheckError::NestedUnsafeBlock { span });
        }

        self.unsafe_block_status = UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls;

        let (hir_block_expression, typ) = self.elaborate_block_expression(block, target_type);

        if let UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls = self.unsafe_block_status
        {
            let span = Span::from(span.start()..span.start() + 6); // Only highlight the `unsafe` keyword
            self.push_err(TypeCheckError::UnnecessaryUnsafeBlock { span });
        }

        // Finally, we restore the original value of `self.in_unsafe_block`,
        // but only if this isn't a nested unsafe block (that way if we found an unconstrained call
        // for this unsafe block we'll also consider the outer one as finding one, and we don't double error)
        if !is_nested_unsafe_block {
            self.unsafe_block_status = old_in_unsafe_block;
        }

        (HirExpression::Unsafe(hir_block_expression), typ)
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
                let len = Type::Constant(str.len().into(), Kind::u32());
                (Lit(HirLiteral::Str(str)), Type::String(Box::new(len)))
            }
            Literal::FmtStr(fragments, length) => self.elaborate_fmt_string(fragments, length),
            Literal::Array(array_literal) => {
                self.elaborate_array_literal(array_literal, span, true)
            }
            Literal::Slice(array_literal) => {
                self.elaborate_array_literal(array_literal, span, false)
            }
        }
    }

    fn elaborate_array_literal(
        &mut self,
        array_literal: ArrayLiteral,
        span: Span,
        is_array: bool,
    ) -> (HirExpression, Type) {
        let (expr, elem_type, length) = match array_literal {
            ArrayLiteral::Standard(elements) => {
                let first_elem_type = self.interner.next_type_variable();
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

                let length = Type::Constant(elements.len().into(), Kind::u32());
                (HirArrayLiteral::Standard(elements), first_elem_type, length)
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                let span = length.span;
                let length =
                    UnresolvedTypeExpression::from_expr(*length, span).unwrap_or_else(|error| {
                        self.push_err(ResolverError::ParserError(Box::new(error)));
                        UnresolvedTypeExpression::Constant(FieldElement::zero(), span)
                    });

                let length = self.convert_expression_type(length, &Kind::u32(), span);
                let (repeated_element, elem_type) = self.elaborate_expression(*repeated_element);

                let length_clone = length.clone();
                (HirArrayLiteral::Repeated { repeated_element, length }, elem_type, length_clone)
            }
        };
        let constructor = if is_array { HirLiteral::Array } else { HirLiteral::Slice };
        let elem_type = Box::new(elem_type);
        let typ = if is_array {
            Type::Array(Box::new(length), elem_type)
        } else {
            Type::Slice(elem_type)
        };
        (HirExpression::Literal(constructor(expr)), typ)
    }

    fn elaborate_fmt_string(
        &mut self,
        fragments: Vec<FmtStrFragment>,
        length: u32,
    ) -> (HirExpression, Type) {
        let mut fmt_str_idents = Vec::new();
        let mut capture_types = Vec::new();

        for fragment in &fragments {
            if let FmtStrFragment::Interpolation(ident_name, string_span) = fragment {
                let scope_tree = self.scopes.current_scope_tree();
                let variable = scope_tree.find(ident_name);

                let hir_ident = if let Some((old_value, _)) = variable {
                    old_value.num_times_used += 1;
                    old_value.ident.clone()
                } else if let Ok((definition_id, _)) =
                    self.lookup_global(Path::from_single(ident_name.to_string(), *string_span))
                {
                    HirIdent::non_trait_method(
                        definition_id,
                        Location::new(*string_span, self.file),
                    )
                } else {
                    self.push_err(ResolverError::VariableNotDeclared {
                        name: ident_name.to_owned(),
                        span: *string_span,
                    });
                    continue;
                };

                let hir_expr = HirExpression::Ident(hir_ident.clone(), None);
                let expr_id = self.interner.push_expr(hir_expr);
                self.interner.push_expr_location(expr_id, *string_span, self.file);
                let typ = self.type_check_variable(hir_ident, expr_id, None);
                self.interner.push_expr_type(expr_id, typ.clone());
                capture_types.push(typ);
                fmt_str_idents.push(expr_id);
            }
        }

        let len = Type::Constant(length.into(), Kind::u32());
        let typ = Type::FmtString(Box::new(len), Box::new(Type::Tuple(capture_types)));
        (HirExpression::Literal(HirLiteral::FmtStr(fragments, fmt_str_idents, length)), typ)
    }

    fn elaborate_prefix(&mut self, prefix: PrefixExpression, span: Span) -> (ExprId, Type) {
        let rhs_span = prefix.rhs.span;

        let (rhs, rhs_type) = self.elaborate_expression(prefix.rhs);
        let trait_id = self.interner.get_prefix_operator_trait_method(&prefix.operator);

        let operator = prefix.operator;

        if let UnaryOp::MutableReference = operator {
            self.check_can_mutate(rhs, rhs_span);
        }

        let expr =
            HirExpression::Prefix(HirPrefixExpression { operator, rhs, trait_method_id: trait_id });
        let expr_id = self.interner.push_expr(expr);
        self.interner.push_expr_location(expr_id, span, self.file);

        let result = self.prefix_operand_type_rules(&operator, &rhs_type, span);
        let typ = self.handle_operand_type_rules_result(result, &rhs_type, trait_id, expr_id, span);

        self.interner.push_expr_type(expr_id, typ.clone());
        (expr_id, typ)
    }

    fn check_can_mutate(&mut self, expr_id: ExprId, span: Span) {
        let expr = self.interner.expression(&expr_id);
        match expr {
            HirExpression::Ident(hir_ident, _) => {
                if let Some(definition) = self.interner.try_definition(hir_ident.id) {
                    if !definition.mutable {
                        self.push_err(TypeCheckError::CannotMutateImmutableVariable {
                            name: definition.name.clone(),
                            span,
                        });
                    }
                }
            }
            HirExpression::MemberAccess(member_access) => {
                self.check_can_mutate(member_access.lhs, span);
            }
            _ => (),
        }
    }

    fn elaborate_index(&mut self, index_expr: IndexExpression) -> (HirExpression, Type) {
        let span = index_expr.index.span;
        let (index, index_type) = self.elaborate_expression(index_expr.index);

        let expected = self.polymorphic_integer_or_field();
        self.unify(&index_type, &expected, || TypeCheckError::TypeMismatch {
            expected_typ: "an integer".to_owned(),
            expr_typ: index_type.to_string(),
            expr_span: span,
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
            Type::TypeVariable(_) => {
                self.push_err(TypeCheckError::TypeAnnotationsNeededForIndex { span: lhs_span });
                Type::Error
            }
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
        let (func, func_type) = self.elaborate_expression(*call.func);
        let func_type = func_type.follow_bindings();
        let func_arg_types =
            if let Type::Function(args, _, _, _) = &func_type { Some(args) } else { None };

        let mut arguments = Vec::with_capacity(call.arguments.len());
        let args = vecmap(call.arguments.into_iter().enumerate(), |(arg_index, arg)| {
            let span = arg.span;
            let expected_type = func_arg_types.and_then(|args| args.get(arg_index));

            let (arg, typ) = if call.is_macro_call {
                self.elaborate_in_comptime_context(|this| {
                    this.elaborate_expression_with_type(arg, expected_type)
                })
            } else {
                self.elaborate_expression_with_type(arg, expected_type)
            };

            // Try to unify this argument type against the function's argument type
            // so that a potential lambda following this argument can have more concrete types.
            if let Some(expected_type) = expected_type {
                let _ = expected_type.unify(&typ);
            }

            arguments.push(arg);
            (typ, arg, span)
        });

        // Avoid cloning arguments unless this is a macro call
        let mut comptime_args = Vec::new();
        if call.is_macro_call {
            comptime_args = arguments.clone();
        }

        let location = Location::new(span, self.file);
        let is_macro_call = call.is_macro_call;
        let hir_call = HirCallExpression { func, arguments, location, is_macro_call };
        let mut typ = self.type_check_call(&hir_call, func_type, args, span);

        if is_macro_call {
            if self.in_comptime_context() {
                typ = self.interner.next_type_variable();
            } else {
                return self
                    .call_macro(func, comptime_args, location, typ)
                    .unwrap_or_else(|| (HirExpression::Error, Type::Error));
            }
        }

        (HirExpression::Call(hir_call), typ)
    }

    fn elaborate_method_call(
        &mut self,
        method_call: MethodCallExpression,
        span: Span,
    ) -> (HirExpression, Type) {
        let object_span = method_call.object.span;
        let (mut object, mut object_type) = self.elaborate_expression(method_call.object);
        object_type = object_type.follow_bindings();

        let method_name_span = method_call.method_name.span();
        let method_name = method_call.method_name.0.contents.as_str();
        match self.lookup_method(&object_type, method_name, span, true) {
            Some(method_ref) => {
                // Automatically add `&mut` if the method expects a mutable reference and
                // the object is not already one.
                let func_id = method_ref
                    .func_id(self.interner)
                    .expect("Expected trait function to be a DefinitionKind::Function");

                let generics = if func_id != FuncId::dummy_id() {
                    let function_type = self.interner.function_meta(&func_id).typ.clone();
                    self.try_add_mutable_reference_to_object(
                        &function_type,
                        &mut object_type,
                        &mut object,
                    );

                    self.resolve_function_turbofish_generics(&func_id, method_call.generics, span)
                } else {
                    None
                };

                let call_span = Span::from(object_span.start()..method_name_span.end());
                let location = Location::new(call_span, self.file);

                let (function_id, function_name) = method_ref.clone().into_function_id_and_name(
                    object_type.clone(),
                    generics.clone(),
                    location,
                    self.interner,
                );

                let func_type =
                    self.type_check_variable(function_name.clone(), function_id, generics.clone());
                self.interner.push_expr_type(function_id, func_type.clone());

                let func_arg_types =
                    if let Type::Function(args, _, _, _) = &func_type { Some(args) } else { None };

                // Try to unify the object type with the first argument of the function.
                // The reason to do this is that many methods that take a lambda will yield `self` or part of `self`
                // as a parameter. By unifying `self` with the first argument we'll potentially get more
                // concrete types in the arguments that are function types, which will later be passed as
                // lambda parameter hints.
                if let Some(first_arg_type) = func_arg_types.and_then(|args| args.first()) {
                    let _ = first_arg_type.unify(&object_type);
                }

                // These arguments will be given to the desugared function call.
                // Compared to the method arguments, they also contain the object.
                let mut function_args = Vec::with_capacity(method_call.arguments.len() + 1);
                let mut arguments = Vec::with_capacity(method_call.arguments.len());

                function_args.push((object_type.clone(), object, object_span));

                for (arg_index, arg) in method_call.arguments.into_iter().enumerate() {
                    let span = arg.span;
                    let expected_type = func_arg_types.and_then(|args| args.get(arg_index + 1));
                    let (arg, typ) = self.elaborate_expression_with_type(arg, expected_type);

                    // Try to unify this argument type against the function's argument type
                    // so that a potential lambda following this argument can have more concrete types.
                    if let Some(expected_type) = expected_type {
                        let _ = expected_type.unify(&typ);
                    }

                    arguments.push(arg);
                    function_args.push((typ, arg, span));
                }

                let method = method_call.method_name;
                let is_macro_call = method_call.is_macro_call;
                let method_call =
                    HirMethodCallExpression { method, object, arguments, location, generics };

                self.check_method_call_visibility(func_id, &object_type, &method_call.method);

                // Desugar the method call into a normal, resolved function call
                // so that the backend doesn't need to worry about methods
                // TODO: update object_type here?

                let function_call =
                    method_call.into_function_call(function_id, is_macro_call, location);

                self.interner
                    .add_function_reference(func_id, Location::new(method_name_span, self.file));

                // Type check the new call now that it has been changed from a method call
                // to a function call. This way we avoid duplicating code.
                let mut typ = self.type_check_call(&function_call, func_type, function_args, span);
                if is_macro_call {
                    if self.in_comptime_context() {
                        typ = self.interner.next_type_variable();
                    } else {
                        let args = function_call.arguments.clone();
                        return self
                            .call_macro(function_call.func, args, location, typ)
                            .unwrap_or_else(|| (HirExpression::Error, Type::Error));
                    }
                }
                (HirExpression::Call(function_call), typ)
            }
            None => (HirExpression::Error, Type::Error),
        }
    }

    /// Elaborates an expression knowing that it has to match a given type.
    fn elaborate_expression_with_type(
        &mut self,
        arg: Expression,
        typ: Option<&Type>,
    ) -> (ExprId, Type) {
        let ExpressionKind::Lambda(lambda) = arg.kind else {
            return self.elaborate_expression(arg);
        };

        let span = arg.span;
        let type_hint =
            if let Some(Type::Function(func_args, _, _, _)) = typ { Some(func_args) } else { None };
        let (hir_expr, typ) = self.elaborate_lambda_with_parameter_type_hints(*lambda, type_hint);
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, span, self.file);
        self.interner.push_expr_type(id, typ.clone());
        (id, typ)
    }

    fn check_method_call_visibility(&mut self, func_id: FuncId, object_type: &Type, name: &Ident) {
        if !method_call_is_visible(
            object_type,
            func_id,
            self.module_id(),
            self.interner,
            self.def_maps,
        ) {
            self.push_err(ResolverError::PathResolutionError(PathResolutionError::Private(
                name.clone(),
            )));
        }
    }

    fn elaborate_constructor(
        &mut self,
        constructor: ConstructorExpression,
    ) -> (HirExpression, Type) {
        let span = constructor.typ.span;

        // A constructor type can either be a Path or an interned UnresolvedType.
        // We represent both as UnresolvedType (with Path being a Named UnresolvedType)
        // and error if we don't get a Named path.
        let mut typ = constructor.typ.typ;
        if let UnresolvedTypeData::Interned(id) = typ {
            typ = self.interner.get_unresolved_type_data(id).clone();
        }
        if let UnresolvedTypeData::Resolved(id) = typ {
            // If this type is already resolved we can skip the rest of this function
            // which just resolves the type, and go straight to resolving the fields.
            let resolved = self.interner.get_quoted_type(id).clone();
            return self.elaborate_constructor_with_type(resolved, constructor.fields, span, None);
        }
        let UnresolvedTypeData::Named(mut path, generics, _) = typ else {
            self.push_err(ResolverError::NonStructUsedInConstructor { typ: typ.to_string(), span });
            return (HirExpression::Error, Type::Error);
        };

        let last_segment = path.segments.last_mut().unwrap();
        if !generics.ordered_args.is_empty() {
            last_segment.generics = Some(generics.ordered_args);
        }

        let last_segment = path.last_segment();

        let typ = if let Some(struct_id) = constructor.struct_type {
            let typ = self.interner.get_type(struct_id);
            let generics = typ.borrow().instantiate(self.interner);
            Type::DataType(typ, generics)
        } else {
            match self.lookup_type_or_error(path) {
                Some(typ) => typ,
                None => return (HirExpression::Error, Type::Error),
            }
        };

        self.elaborate_constructor_with_type(typ, constructor.fields, span, Some(last_segment))
    }

    fn elaborate_constructor_with_type(
        &mut self,
        typ: Type,
        fields: Vec<(Ident, Expression)>,
        span: Span,
        last_segment: Option<PathSegment>,
    ) -> (HirExpression, Type) {
        let typ = typ.follow_bindings_shallow();
        let (r#type, generics) = match typ.as_ref() {
            Type::DataType(r#type, struct_generics) if r#type.borrow().is_struct() => {
                (r#type, struct_generics)
            }
            typ => {
                self.push_err(ResolverError::NonStructUsedInConstructor {
                    typ: typ.to_string(),
                    span,
                });
                return (HirExpression::Error, Type::Error);
            }
        };
        self.mark_struct_as_constructed(r#type.clone());

        // `last_segment` is optional if this constructor was resolved from a quoted type
        let mut generics = generics.clone();
        let mut is_self_type = false;
        let mut constructor_type_span = span;

        if let Some(last_segment) = last_segment {
            let turbofish_span = last_segment.turbofish_span();
            is_self_type = last_segment.ident.is_self_type_name();
            constructor_type_span = last_segment.ident.span();

            generics = self.resolve_struct_turbofish_generics(
                &r#type.borrow(),
                generics,
                last_segment.generics,
                turbofish_span,
            );
        }

        let struct_type = r#type.clone();

        let field_types = r#type
            .borrow()
            .get_fields_with_visibility(&generics)
            .expect("This type should already be validated to be a struct");

        let fields =
            self.resolve_constructor_expr_fields(struct_type.clone(), field_types, fields, span);
        let expr = HirExpression::Constructor(HirConstructorExpression {
            fields,
            r#type: struct_type.clone(),
            struct_generics: generics.clone(),
        });

        let struct_id = struct_type.borrow().id;
        let reference_location = Location::new(constructor_type_span, self.file);
        self.interner.add_type_reference(struct_id, reference_location, is_self_type);

        (expr, Type::DataType(struct_type, generics))
    }

    pub(super) fn mark_struct_as_constructed(&mut self, struct_type: Shared<DataType>) {
        let struct_type = struct_type.borrow();
        let parent_module_id = struct_type.id.parent_module_id(self.def_maps);
        self.usage_tracker.mark_as_used(parent_module_id, &struct_type.name);
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    fn resolve_constructor_expr_fields(
        &mut self,
        struct_type: Shared<DataType>,
        field_types: Vec<(String, ItemVisibility, Type)>,
        fields: Vec<(Ident, Expression)>,
        span: Span,
    ) -> Vec<(Ident, ExprId)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::default();
        let mut unseen_fields = struct_type
            .borrow()
            .field_names()
            .expect("This type should already be validated to be a struct");

        for (field_name, field) in fields {
            let expected_field_with_index = field_types
                .iter()
                .enumerate()
                .find(|(_, (name, _, _))| name == &field_name.0.contents);
            let expected_index_and_visibility =
                expected_field_with_index.map(|(index, (_, visibility, _))| (index, visibility));
            let expected_type =
                expected_field_with_index.map(|(_, (_, _, typ))| typ).unwrap_or(&Type::Error);

            let field_span = field.span;
            let (resolved, field_type) = self.elaborate_expression(field);

            if unseen_fields.contains(&field_name) {
                unseen_fields.remove(&field_name);
                seen_fields.insert(field_name.clone());

                self.unify_with_coercions(&field_type, expected_type, resolved, field_span, || {
                    TypeCheckError::TypeMismatch {
                        expected_typ: expected_type.to_string(),
                        expr_typ: field_type.to_string(),
                        expr_span: field_span,
                    }
                });
            } else if seen_fields.contains(&field_name) {
                // duplicate field
                self.push_err(ResolverError::DuplicateField { field: field_name.clone() });
            } else {
                // field not required by struct
                self.push_err(ResolverError::NoSuchField {
                    field: field_name.clone(),
                    struct_definition: struct_type.borrow().name.clone(),
                });
            }

            if let Some((index, visibility)) = expected_index_and_visibility {
                let struct_type = struct_type.borrow();
                let field_span = field_name.span();
                let field_name = &field_name.0.contents;
                self.check_struct_field_visibility(
                    &struct_type,
                    field_name,
                    *visibility,
                    field_span,
                );

                self.interner.add_struct_member_reference(
                    struct_type.id,
                    index,
                    Location::new(field_span, self.file),
                );
            }

            ret.push((field_name, resolved));
        }

        if !unseen_fields.is_empty() {
            self.push_err(ResolverError::MissingFields {
                span,
                missing_fields: unseen_fields.into_iter().map(|field| field.to_string()).collect(),
                struct_definition: struct_type.borrow().name.clone(),
            });
        }

        ret
    }

    fn elaborate_member_access(
        &mut self,
        access: MemberAccessExpression,
        span: Span,
    ) -> (ExprId, Type) {
        let (lhs, lhs_type) = self.elaborate_expression(access.lhs);
        let rhs = access.rhs;
        let rhs_span = rhs.span();
        // `is_offset` is only used when lhs is a reference and we want to return a reference to rhs
        let access = HirMemberAccess { lhs, rhs, is_offset: false };
        let expr_id = self.intern_expr(HirExpression::MemberAccess(access.clone()), span);
        let typ = self.type_check_member_access(access, expr_id, lhs_type, rhs_span);
        self.interner.push_expr_type(expr_id, typ.clone());
        (expr_id, typ)
    }

    pub fn intern_expr(&mut self, expr: HirExpression, span: Span) -> ExprId {
        let id = self.interner.push_expr(expr);
        self.interner.push_expr_location(id, span, self.file);
        id
    }

    fn elaborate_cast(&mut self, cast: CastExpression, span: Span) -> (HirExpression, Type) {
        let (lhs, lhs_type) = self.elaborate_expression(cast.lhs);
        let r#type = self.resolve_type(cast.r#type);
        let result = self.check_cast(&lhs, &lhs_type, &r#type, span);
        let expr = HirExpression::Cast(HirCastExpression { lhs, r#type });
        (expr, result)
    }

    fn elaborate_infix(&mut self, infix: InfixExpression, span: Span) -> (ExprId, Type) {
        let (lhs, lhs_type) = self.elaborate_expression(infix.lhs);
        let (rhs, rhs_type) = self.elaborate_expression(infix.rhs);
        let trait_id = self.interner.get_operator_trait_method(infix.operator.contents);

        let operator = HirBinaryOp::new(infix.operator, self.file);
        let expr = HirExpression::Infix(HirInfixExpression {
            lhs,
            operator,
            trait_method_id: trait_id,
            rhs,
        });

        let expr_id = self.interner.push_expr(expr);
        self.interner.push_expr_location(expr_id, span, self.file);

        let result = self.infix_operand_type_rules(&lhs_type, &operator, &rhs_type, span);
        let typ =
            self.handle_operand_type_rules_result(result, &lhs_type, Some(trait_id), expr_id, span);

        self.interner.push_expr_type(expr_id, typ.clone());
        (expr_id, typ)
    }

    fn handle_operand_type_rules_result(
        &mut self,
        result: Result<(Type, bool), TypeCheckError>,
        operand_type: &Type,
        trait_id: Option<TraitMethodId>,
        expr_id: ExprId,
        span: Span,
    ) -> Type {
        match result {
            Ok((typ, use_impl)) => {
                if use_impl {
                    let trait_id =
                        trait_id.expect("ice: expected some trait_id when use_impl is true");

                    // Delay checking the trait constraint until the end of the function.
                    // Checking it now could bind an unbound type variable to any type
                    // that implements the trait.
                    let constraint = TraitConstraint {
                        typ: operand_type.clone(),
                        trait_bound: ResolvedTraitBound {
                            trait_id: trait_id.trait_id,
                            trait_generics: TraitGenerics::default(),
                            span,
                        },
                    };
                    self.push_trait_constraint(
                        constraint, expr_id,
                        true, // this constraint should lead to choosing a trait impl
                    );
                    self.type_check_operator_method(expr_id, trait_id, operand_type, span);
                }
                typ
            }
            Err(error) => {
                self.push_err(error);
                Type::Error
            }
        }
    }

    fn elaborate_if(
        &mut self,
        if_expr: IfExpression,
        target_type: Option<&Type>,
    ) -> (HirExpression, Type) {
        let expr_span = if_expr.condition.span;
        let consequence_span = if_expr.consequence.span;
        let (condition, cond_type) = self.elaborate_expression(if_expr.condition);
        let (consequence, mut ret_type) =
            self.elaborate_expression_with_target_type(if_expr.consequence, target_type);

        self.unify(&cond_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool.to_string(),
            expr_typ: cond_type.to_string(),
            expr_span,
        });

        let (alternative, else_type, error_span) = if let Some(alternative) = if_expr.alternative {
            let (else_, else_type) =
                self.elaborate_expression_with_target_type(alternative, target_type);
            (Some(else_), else_type, expr_span)
        } else {
            (None, Type::Unit, consequence_span)
        };

        self.unify(&ret_type, &else_type, || {
            let err = TypeCheckError::TypeMismatch {
                expected_typ: ret_type.to_string(),
                expr_typ: else_type.to_string(),
                expr_span: error_span,
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

        if alternative.is_none() {
            ret_type = Type::Unit;
        }

        let if_expr = HirIfExpression { condition, consequence, alternative };
        (HirExpression::If(if_expr), ret_type)
    }

    fn elaborate_match(
        &mut self,
        match_expr: MatchExpression,
        span: Span,
    ) -> (HirExpression, Type) {
        let (expression, typ) = self.elaborate_expression(match_expr.expression);
        let (let_, variable) = self.wrap_in_let(expression, typ);

        let (rows, result_type) = self.elaborate_match_rules(variable, match_expr.rules);
        let tree = HirExpression::Match(self.elaborate_match_rows(rows));
        let tree = self.interner.push_expr(tree);
        self.interner.push_expr_type(tree, result_type.clone());
        self.interner.push_expr_location(tree, span, self.file);

        let tree = self.interner.push_stmt(HirStatement::Expression(tree));
        self.interner.push_stmt_location(tree, span, self.file);

        let block = HirExpression::Block(HirBlockExpression { statements: vec![let_, tree] });
        (block, result_type)
    }

    fn wrap_in_let(&mut self, expr_id: ExprId, typ: Type) -> (StmtId, DefinitionId) {
        let location = self.interner.expr_location(&expr_id);
        let name = "internal variable".to_string();
        let definition = DefinitionKind::Local(None);
        let variable = self.interner.push_definition(name, false, false, definition, location);
        self.interner.push_definition_type(variable, typ.clone());

        let pattern = HirPattern::Identifier(HirIdent::non_trait_method(variable, location));
        let let_ = HirStatement::Let(HirLetStatement::basic(pattern, typ, expr_id));
        let let_ = self.interner.push_stmt(let_);
        self.interner.push_stmt_location(let_, location.span, location.file);
        (let_, variable)
    }

    fn elaborate_tuple(
        &mut self,
        tuple: Vec<Expression>,
        target_type: Option<&Type>,
    ) -> (HirExpression, Type) {
        let mut element_ids = Vec::with_capacity(tuple.len());
        let mut element_types = Vec::with_capacity(tuple.len());

        for (index, element) in tuple.into_iter().enumerate() {
            let target_type = target_type.map(|typ| typ.follow_bindings());
            let expr_target_type =
                if let Some(Type::Tuple(types)) = &target_type { types.get(index) } else { None };
            let (id, typ) = self.elaborate_expression_with_target_type(element, expr_target_type);
            element_ids.push(id);
            element_types.push(typ);
        }

        (HirExpression::Tuple(element_ids), Type::Tuple(element_types))
    }

    fn elaborate_lambda_with_target_type(
        &mut self,
        lambda: Lambda,
        target_type: Option<&Type>,
    ) -> (HirExpression, Type) {
        let target_type = target_type.map(|typ| typ.follow_bindings());

        if let Some(Type::Function(args, _, _, _)) = target_type {
            return self.elaborate_lambda_with_parameter_type_hints(lambda, Some(&args));
        }

        self.elaborate_lambda_with_parameter_type_hints(lambda, None)
    }

    /// For elaborating a lambda we might get `parameters_type_hints`. These come from a potential
    /// call that has this lambda as the argument.
    /// The parameter type hints will be the types of the function type corresponding to the lambda argument.
    fn elaborate_lambda_with_parameter_type_hints(
        &mut self,
        lambda: Lambda,
        parameters_type_hints: Option<&Vec<Type>>,
    ) -> (HirExpression, Type) {
        self.push_scope();
        let scope_index = self.scopes.current_scope_index();

        self.lambda_stack.push(LambdaContext { captures: Vec::new(), scope_index });

        let mut arg_types = Vec::with_capacity(lambda.parameters.len());
        let parameters =
            vecmap(lambda.parameters.into_iter().enumerate(), |(index, (pattern, typ))| {
                let parameter = DefinitionKind::Local(None);
                let typ = if let UnresolvedTypeData::Unspecified = typ.typ {
                    if let Some(parameter_type_hint) =
                        parameters_type_hints.and_then(|hints| hints.get(index))
                    {
                        parameter_type_hint.clone()
                    } else {
                        self.interner.next_type_variable_with_kind(Kind::Any)
                    }
                } else {
                    self.resolve_type(typ)
                };

                arg_types.push(typ.clone());
                (self.elaborate_pattern(pattern, typ.clone(), parameter, true), typ)
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
        (expr, Type::Function(arg_types, Box::new(body_type), Box::new(env_type), false))
    }

    fn elaborate_quote(&mut self, mut tokens: Tokens, span: Span) -> (HirExpression, Type) {
        tokens = self.find_unquoted_exprs_tokens(tokens);

        if self.in_comptime_context() {
            (HirExpression::Quote(tokens), Type::Quoted(QuotedType::Quoted))
        } else {
            self.push_err(ResolverError::QuoteInRuntimeCode { span });
            (HirExpression::Error, Type::Quoted(QuotedType::Quoted))
        }
    }

    fn elaborate_comptime_block(
        &mut self,
        block: BlockExpression,
        span: Span,
        target_type: Option<&Type>,
    ) -> (ExprId, Type) {
        let (block, _typ) = self.elaborate_in_comptime_context(|this| {
            this.elaborate_block_expression(block, target_type)
        });

        let mut interpreter = self.setup_interpreter();
        let value = interpreter.evaluate_block(block);
        let (id, typ) = self.inline_comptime_value(value, span);

        let location = self.interner.id_location(id);
        self.debug_comptime(location, |interner| {
            interner.expression(&id).to_display_ast(interner, location.span).kind
        });

        (id, typ)
    }

    pub fn inline_comptime_value(
        &mut self,
        value: Result<comptime::Value, InterpreterError>,
        span: Span,
    ) -> (ExprId, Type) {
        let make_error = |this: &mut Self, error: InterpreterError| {
            this.errors.push(error.into_compilation_error_pair());
            let error = this.interner.push_expr(HirExpression::Error);
            this.interner.push_expr_location(error, span, this.file);
            (error, Type::Error)
        };

        let value = match value {
            Ok(value) => value,
            Err(error) => return make_error(self, error),
        };

        let location = Location::new(span, self.file);
        match value.into_expression(self, location) {
            Ok(new_expr) => {
                // At this point the Expression was already elaborated and we got a Value.
                // We'll elaborate this value turned into Expression to inline it and get
                // an ExprId and Type, but we don't want any visibility errors to happen
                // here (they could if we have `Foo { inner: 5 }` and `inner` is not
                // accessible from where this expression is being elaborated).
                self.silence_field_visibility_errors += 1;
                let value = self.elaborate_expression(new_expr);
                self.silence_field_visibility_errors -= 1;
                value
            }
            Err(error) => make_error(self, error),
        }
    }

    fn try_get_comptime_function(
        &mut self,
        func: ExprId,
        location: Location,
    ) -> Result<Option<FuncId>, ResolverError> {
        match self.interner.expression(&func) {
            HirExpression::Ident(ident, _generics) => {
                if let Some(definition) = self.interner.try_definition(ident.id) {
                    if let DefinitionKind::Function(function) = definition.kind {
                        let meta = self.interner.function_modifiers(&function);
                        if meta.is_comptime {
                            Ok(Some(function))
                        } else {
                            Err(ResolverError::MacroIsNotComptime { span: location.span })
                        }
                    } else {
                        Err(ResolverError::InvalidSyntaxInMacroCall { span: location.span })
                    }
                } else {
                    // Assume a name resolution error has already been issued
                    Ok(None)
                }
            }
            _ => Err(ResolverError::InvalidSyntaxInMacroCall { span: location.span }),
        }
    }

    /// Call a macro function and inlines its code at the call site.
    /// This will also perform a type check to ensure that the return type is an `Expr` value.
    fn call_macro(
        &mut self,
        func: ExprId,
        arguments: Vec<ExprId>,
        location: Location,
        return_type: Type,
    ) -> Option<(HirExpression, Type)> {
        self.unify(&return_type, &Type::Quoted(QuotedType::Quoted), || {
            TypeCheckError::MacroReturningNonExpr { typ: return_type.clone(), span: location.span }
        });

        let function = match self.try_get_comptime_function(func, location) {
            Ok(function) => function?,
            Err(error) => {
                self.push_err(error);
                return None;
            }
        };

        let file = self.file;
        let mut interpreter = self.setup_interpreter();
        let mut comptime_args = Vec::new();
        let mut errors = Vec::new();

        for argument in arguments {
            match interpreter.evaluate(argument) {
                Ok(arg) => {
                    let location = interpreter.elaborator.interner.expr_location(&argument);
                    comptime_args.push((arg, location));
                }
                Err(error) => errors.push((error.into(), file)),
            }
        }

        let bindings = interpreter.elaborator.interner.get_instantiation_bindings(func).clone();
        let result = interpreter.call_function(function, comptime_args, bindings, location);

        if !errors.is_empty() {
            self.errors.append(&mut errors);
            return None;
        }

        let (expr_id, typ) = self.inline_comptime_value(result, location.span);
        Some((self.interner.expression(&expr_id), typ))
    }
}
