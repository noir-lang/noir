use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;

use crate::{
    ast::{
        ArrayLiteral, ConstructorExpression, IfExpression, InfixExpression, Lambda,
        UnresolvedTypeExpression,
    },
    hir::{
        comptime::{self, InterpreterError},
        resolution::{errors::ResolverError, resolver::LambdaContext},
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::{
            HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirExpression, HirIfExpression, HirIndexExpression,
            HirInfixExpression, HirLambda, HirMemberAccess, HirMethodCallExpression,
            HirMethodReference, HirPrefixExpression,
        },
        traits::TraitConstraint,
    },
    macros_api::{
        BlockExpression, CallExpression, CastExpression, Expression, ExpressionKind, HirLiteral,
        HirStatement, Ident, IndexExpression, Literal, MemberAccessExpression,
        MethodCallExpression, PrefixExpression,
    },
    node_interner::{DefinitionKind, ExprId, FuncId, TraitMethodId},
    token::Tokens,
    QuotedType, Shared, StructType, Type,
};

use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub(super) fn elaborate_expression(&mut self, expr: Expression) -> (ExprId, Type) {
        let (hir_expr, typ) = match expr.kind {
            ExpressionKind::Literal(literal) => self.elaborate_literal(literal, expr.span),
            ExpressionKind::Block(block) => self.elaborate_block(block),
            ExpressionKind::Prefix(prefix) => return self.elaborate_prefix(*prefix),
            ExpressionKind::Index(index) => self.elaborate_index(*index),
            ExpressionKind::Call(call) => self.elaborate_call(*call, expr.span),
            ExpressionKind::MethodCall(call) => self.elaborate_method_call(*call, expr.span),
            ExpressionKind::Constructor(constructor) => self.elaborate_constructor(*constructor),
            ExpressionKind::MemberAccess(access) => {
                return self.elaborate_member_access(*access, expr.span)
            }
            ExpressionKind::Cast(cast) => self.elaborate_cast(*cast, expr.span),
            ExpressionKind::Infix(infix) => return self.elaborate_infix(*infix, expr.span),
            ExpressionKind::If(if_) => self.elaborate_if(*if_),
            ExpressionKind::Variable(variable, generics) => {
                return self.elaborate_variable(variable, generics)
            }
            ExpressionKind::Tuple(tuple) => self.elaborate_tuple(tuple),
            ExpressionKind::Lambda(lambda) => self.elaborate_lambda(*lambda),
            ExpressionKind::Parenthesized(expr) => return self.elaborate_expression(*expr),
            ExpressionKind::Quote(quote) => self.elaborate_quote(quote),
            ExpressionKind::Comptime(comptime, _) => {
                return self.elaborate_comptime_block(comptime, expr.span)
            }
            ExpressionKind::Resolved(id) => return (id, self.interner.id_type(id)),
            ExpressionKind::Error => (HirExpression::Error, Type::Error),
            ExpressionKind::Unquote(_) => {
                self.push_err(ResolverError::UnquoteUsedOutsideQuote { span: expr.span });
                (HirExpression::Error, Type::Error)
            }
        };
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, expr.span, self.file);
        self.interner.push_expr_type(id, typ.clone());
        (id, typ)
    }

    pub(super) fn elaborate_block(&mut self, block: BlockExpression) -> (HirExpression, Type) {
        let (block, typ) = self.elaborate_block_expression(block);
        (HirExpression::Block(block), typ)
    }

    fn elaborate_block_expression(&mut self, block: BlockExpression) -> (HirBlockExpression, Type) {
        self.push_scope();
        let mut block_type = Type::Unit;
        let mut statements = Vec::with_capacity(block.statements.len());

        for (i, statement) in block.statements.into_iter().enumerate() {
            let (id, stmt_type) = self.elaborate_statement(statement);
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
                let len = Type::Constant(str.len() as u32);
                (Lit(HirLiteral::Str(str)), Type::String(Box::new(len)))
            }
            Literal::FmtStr(str) => self.elaborate_fmt_string(str, span),
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

                let length = Type::Constant(elements.len() as u32);
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
        let typ = if is_array {
            Type::Array(Box::new(length), elem_type)
        } else {
            Type::Slice(elem_type)
        };
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
                let ident = HirExpression::Ident(old_value.ident.clone(), None);
                let expr_id = self.interner.push_expr(ident);
                self.interner.push_expr_location(expr_id, call_expr_span, self.file);
                let ident = old_value.ident.clone();
                let typ = self.type_check_variable(ident, expr_id, None);
                self.interner.push_expr_type(expr_id, typ.clone());
                capture_types.push(typ);
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

        let len = Type::Constant(str.len() as u32);
        let typ = Type::FmtString(Box::new(len), Box::new(Type::Tuple(capture_types)));
        (HirExpression::Literal(HirLiteral::FmtStr(str, fmt_str_idents)), typ)
    }

    fn elaborate_prefix(&mut self, prefix: PrefixExpression) -> (ExprId, Type) {
        let span = prefix.rhs.span;
        let (rhs, rhs_type) = self.elaborate_expression(prefix.rhs);
        let trait_id = self.interner.get_prefix_operator_trait_method(&prefix.operator);

        let operator = prefix.operator;
        let expr =
            HirExpression::Prefix(HirPrefixExpression { operator, rhs, trait_method_id: trait_id });
        let expr_id = self.interner.push_expr(expr);
        self.interner.push_expr_location(expr_id, span, self.file);

        let result = self.prefix_operand_type_rules(&operator, &rhs_type, span);
        let typ = self.handle_operand_type_rules_result(result, &rhs_type, trait_id, expr_id, span);

        self.interner.push_expr_type(expr_id, typ.clone());
        (expr_id, typ)
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

        let mut arguments = Vec::with_capacity(call.arguments.len());
        let args = vecmap(call.arguments, |arg| {
            let span = arg.span;
            let (arg, typ) = self.elaborate_expression(arg);
            arguments.push(arg);
            (typ, arg, span)
        });

        // Avoid cloning arguments unless this is a macro call
        let mut comptime_args = Vec::new();
        if call.is_macro_call {
            comptime_args = arguments.clone();
        }

        let location = Location::new(span, self.file);
        let hir_call = HirCallExpression { func, arguments, location };
        let typ = self.type_check_call(&hir_call, func_type, args, span);

        if call.is_macro_call {
            self.call_macro(func, comptime_args, location, typ)
                .unwrap_or_else(|| (HirExpression::Error, Type::Error))
        } else {
            (HirExpression::Call(hir_call), typ)
        }
    }

    fn elaborate_method_call(
        &mut self,
        method_call: MethodCallExpression,
        span: Span,
    ) -> (HirExpression, Type) {
        let object_span = method_call.object.span;
        let (mut object, mut object_type) = self.elaborate_expression(method_call.object);
        object_type = object_type.follow_bindings();

        let method_name = method_call.method_name.0.contents.as_str();
        match self.lookup_method(&object_type, method_name, span) {
            Some(method_ref) => {
                // Automatically add `&mut` if the method expects a mutable reference and
                // the object is not already one.
                let func_id = match &method_ref {
                    HirMethodReference::FuncId(func_id) => *func_id,
                    HirMethodReference::TraitMethodId(method_id, _) => {
                        let id = self.interner.trait_method_id(*method_id);
                        let definition = self.interner.definition(id);
                        let DefinitionKind::Function(func_id) = definition.kind else {
                            unreachable!("Expected trait function to be a DefinitionKind::Function")
                        };
                        func_id
                    }
                };

                let generics = if func_id != FuncId::dummy_id() {
                    let function_type = self.interner.function_meta(&func_id).typ.clone();
                    self.try_add_mutable_reference_to_object(
                        &function_type,
                        &mut object_type,
                        &mut object,
                    );

                    self.resolve_turbofish_generics(&func_id, method_call.generics, span)
                } else {
                    None
                };

                // These arguments will be given to the desugared function call.
                // Compared to the method arguments, they also contain the object.
                let mut function_args = Vec::with_capacity(method_call.arguments.len() + 1);
                let mut arguments = Vec::with_capacity(method_call.arguments.len());

                function_args.push((object_type.clone(), object, object_span));

                for arg in method_call.arguments {
                    let span = arg.span;
                    let (arg, typ) = self.elaborate_expression(arg);
                    arguments.push(arg);
                    function_args.push((typ, arg, span));
                }

                let location = Location::new(span, self.file);
                let method = method_call.method_name;
                let turbofish_generics = generics.clone();
                let method_call =
                    HirMethodCallExpression { method, object, arguments, location, generics };

                // Desugar the method call into a normal, resolved function call
                // so that the backend doesn't need to worry about methods
                // TODO: update object_type here?
                let ((function_id, function_name), function_call) = method_call.into_function_call(
                    &method_ref,
                    object_type,
                    location,
                    self.interner,
                );

                let func_type =
                    self.type_check_variable(function_name, function_id, turbofish_generics);

                self.interner.push_expr_type(function_id, func_type.clone());

                // Type check the new call now that it has been changed from a method call
                // to a function call. This way we avoid duplicating code.
                let typ = self.type_check_call(&function_call, func_type, function_args, span);
                (HirExpression::Call(function_call), typ)
            }
            None => (HirExpression::Error, Type::Error),
        }
    }

    fn elaborate_constructor(
        &mut self,
        constructor: ConstructorExpression,
    ) -> (HirExpression, Type) {
        let span = constructor.type_name.span();
        let is_self_type = constructor.type_name.last_segment().is_self_type_name();

        let (r#type, struct_generics) = if let Some(struct_id) = constructor.struct_type {
            let typ = self.interner.get_struct(struct_id);
            let generics = typ.borrow().instantiate(self.interner);
            (typ, generics)
        } else {
            match self.lookup_type_or_error(constructor.type_name) {
                Some(Type::Struct(r#type, struct_generics)) => (r#type, struct_generics),
                Some(typ) => {
                    self.push_err(ResolverError::NonStructUsedInConstructor { typ, span });
                    return (HirExpression::Error, Type::Error);
                }
                None => return (HirExpression::Error, Type::Error),
            }
        };

        let struct_type = r#type.clone();
        let generics = struct_generics.clone();

        let fields = constructor.fields;
        let field_types = r#type.borrow().get_fields(&struct_generics);
        let fields =
            self.resolve_constructor_expr_fields(struct_type.clone(), field_types, fields, span);
        let expr = HirExpression::Constructor(HirConstructorExpression {
            fields,
            r#type,
            struct_generics,
        });

        let struct_id = struct_type.borrow().id;
        let reference_location = Location::new(span, self.file);
        self.interner.add_struct_reference(struct_id, reference_location, is_self_type);

        (expr, Type::Struct(struct_type, generics))
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    fn resolve_constructor_expr_fields(
        &mut self,
        struct_type: Shared<StructType>,
        field_types: Vec<(String, Type)>,
        fields: Vec<(Ident, Expression)>,
        span: Span,
    ) -> Vec<(Ident, ExprId)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::default();
        let mut unseen_fields = struct_type.borrow().field_names();

        for (field_name, field) in fields {
            let expected_field_with_index = field_types
                .iter()
                .enumerate()
                .find(|(_, (name, _))| name == &field_name.0.contents);
            let expected_index = expected_field_with_index.map(|(index, _)| index);
            let expected_type =
                expected_field_with_index.map(|(_, (_, typ))| typ).unwrap_or(&Type::Error);

            let field_span = field.span;
            let (resolved, field_type) = self.elaborate_expression(field);

            if unseen_fields.contains(&field_name) {
                unseen_fields.remove(&field_name);
                seen_fields.insert(field_name.clone());

                self.unify_with_coercions(&field_type, expected_type, resolved, || {
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

            if let Some(expected_index) = expected_index {
                self.interner.add_struct_member_reference(
                    struct_type.borrow().id,
                    expected_index,
                    Location::new(field_name.span(), self.file),
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
        let result = self.check_cast(lhs_type, &r#type, span);
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
                        trait_id: trait_id.trait_id,
                        trait_generics: Vec::new(),
                    };
                    self.push_trait_constraint(constraint, expr_id);
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

    fn elaborate_quote(&mut self, mut tokens: Tokens) -> (HirExpression, Type) {
        tokens = self.find_unquoted_exprs_tokens(tokens);
        (HirExpression::Quote(tokens), Type::Quoted(QuotedType::Quoted))
    }

    fn elaborate_comptime_block(&mut self, block: BlockExpression, span: Span) -> (ExprId, Type) {
        // We have to push a new FunctionContext so that we can resolve any constraints
        // in this comptime block early before the function as a whole finishes elaborating.
        // Otherwise the interpreter below may find expressions for which the underlying trait
        // call is not yet solved for.
        self.function_context.push(Default::default());
        let (block, _typ) = self.elaborate_block_expression(block);

        self.check_and_pop_function_context();
        let mut interpreter_errors = vec![];
        let mut interpreter = self.setup_interpreter(&mut interpreter_errors);
        let value = interpreter.evaluate_block(block);
        self.include_interpreter_errors(interpreter_errors);
        let (id, typ) = self.inline_comptime_value(value, span);

        let location = self.interner.id_location(id);
        self.debug_comptime(location, |interner| {
            interner.expression(&id).to_display_ast(interner, location.span).kind
        });

        (id, typ)
    }

    pub(super) fn inline_comptime_value(
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
        match value.into_expression(self.interner, location) {
            Ok(new_expr) => self.elaborate_expression(new_expr),
            Err(error) => make_error(self, error),
        }
    }

    fn try_get_comptime_function(
        &mut self,
        func: ExprId,
        location: Location,
    ) -> Result<FuncId, ResolverError> {
        match self.interner.expression(&func) {
            HirExpression::Ident(ident, _generics) => {
                let definition = self.interner.definition(ident.id);
                if let DefinitionKind::Function(function) = definition.kind {
                    let meta = self.interner.function_modifiers(&function);
                    if meta.is_comptime {
                        Ok(function)
                    } else {
                        Err(ResolverError::MacroIsNotComptime { span: location.span })
                    }
                } else {
                    Err(ResolverError::InvalidSyntaxInMacroCall { span: location.span })
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
            Ok(function) => function,
            Err(error) => {
                self.push_err(error);
                return None;
            }
        };

        let file = self.file;
        let mut interpreter_errors = vec![];
        let mut interpreter = self.setup_interpreter(&mut interpreter_errors);
        let mut comptime_args = Vec::new();
        let mut errors = Vec::new();

        for argument in arguments {
            match interpreter.evaluate(argument) {
                Ok(arg) => {
                    let location = interpreter.interner.expr_location(&argument);
                    comptime_args.push((arg, location));
                }
                Err(error) => errors.push((error.into(), file)),
            }
        }

        let bindings = interpreter.interner.get_instantiation_bindings(func).clone();
        let result = interpreter.call_function(function, comptime_args, bindings, location);
        self.include_interpreter_errors(interpreter_errors);

        if !errors.is_empty() {
            self.errors.append(&mut errors);
            return None;
        }

        let (expr_id, typ) = self.inline_comptime_value(result, location.span);
        Some((self.interner.expression(&expr_id), typ))
    }
}
