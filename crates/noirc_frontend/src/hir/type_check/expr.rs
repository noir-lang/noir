use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    hir::{resolution::resolver::verify_mutable_reference, type_check::errors::Source},
    hir_def::{
        expr::{
            self, HirArrayLiteral, HirBinaryOp, HirExpression, HirLiteral, HirMethodCallExpression,
            HirPrefixExpression,
        },
        types::Type,
    },
    node_interner::{DefinitionKind, ExprId, FuncId},
    token::Attribute::Deprecated,
    CompTime, Shared, TypeBinding, TypeVariableKind, UnaryOp,
};

use super::{errors::TypeCheckError, TypeChecker};

impl<'interner> TypeChecker<'interner> {
    fn check_if_deprecated(&mut self, expr: &ExprId) {
        if let HirExpression::Ident(expr::HirIdent { location, id }) =
            self.interner.expression(expr)
        {
            if let Some(DefinitionKind::Function(func_id)) =
                self.interner.try_definition(id).map(|def| &def.kind)
            {
                let meta = self.interner.function_meta(func_id);
                if let Some(Deprecated(note)) = meta.attributes {
                    self.errors.push(TypeCheckError::CallDeprecated {
                        name: self.interner.definition_name(id).to_string(),
                        note,
                        span: location.span,
                    });
                }
            }
        }
    }
    /// Infers a type for a given expression, and return this type.
    /// As a side-effect, this function will also remember this type in the NodeInterner
    /// for the given expr_id key.
    ///
    /// This function also converts any HirExpression::MethodCalls `a.foo(b, c)` into
    /// an equivalent HirExpression::Call in the form `foo(a, b, c)`. This cannot
    /// be done earlier since we need to know the type of the object `a` to resolve which
    /// function `foo` to refer to.
    pub(crate) fn check_expression(&mut self, expr_id: &ExprId) -> Type {
        let typ = match self.interner.expression(expr_id) {
            HirExpression::Ident(ident) => {
                // An identifiers type may be forall-quantified in the case of generic functions.
                // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
                // We must instantiate identifiers at every call site to replace this T with a new type
                // variable to handle generic functions.
                let t = self.interner.id_type(ident.id);
                let (typ, bindings) = t.instantiate(self.interner);
                self.interner.store_instantiation_bindings(*expr_id, bindings);
                typ
            }
            HirExpression::Literal(literal) => {
                match literal {
                    HirLiteral::Array(HirArrayLiteral::Standard(arr)) => {
                        let elem_types = vecmap(&arr, |arg| self.check_expression(arg));

                        let first_elem_type = elem_types
                            .get(0)
                            .cloned()
                            .unwrap_or_else(|| self.interner.next_type_variable());

                        let arr_type = Type::Array(
                            Box::new(Type::constant_variable(arr.len() as u64, self.interner)),
                            Box::new(first_elem_type.clone()),
                        );

                        // Check if the array is homogeneous
                        for (index, elem_type) in elem_types.iter().enumerate().skip(1) {
                            let location = self.interner.expr_location(&arr[index]);

                            elem_type.unify(
                                &first_elem_type,
                                location.span,
                                &mut self.errors,
                                || {
                                    TypeCheckError::NonHomogeneousArray {
                                        first_span: self.interner.expr_location(&arr[0]).span,
                                        first_type: first_elem_type.to_string(),
                                        first_index: index,
                                        second_span: location.span,
                                        second_type: elem_type.to_string(),
                                        second_index: index + 1,
                                    }
                                    .add_context("elements in an array must have the same type")
                                },
                            );
                        }

                        arr_type
                    }
                    HirLiteral::Array(HirArrayLiteral::Repeated { repeated_element, length }) => {
                        let elem_type = self.check_expression(&repeated_element);
                        let length = match length {
                            Type::Constant(length) => {
                                Type::constant_variable(length, self.interner)
                            }
                            other => other,
                        };
                        Type::Array(Box::new(length), Box::new(elem_type))
                    }
                    HirLiteral::Bool(_) => Type::Bool(CompTime::new(self.interner)),
                    HirLiteral::Integer(_) => Type::polymorphic_integer(self.interner),
                    HirLiteral::Str(string) => {
                        let len = Type::Constant(string.len() as u64);
                        Type::String(Box::new(len))
                    }
                    HirLiteral::Unit => Type::Unit,
                }
            }
            HirExpression::Infix(infix_expr) => {
                // The type of the infix expression must be looked up from a type table
                let lhs_type = self.check_expression(&infix_expr.lhs);
                let rhs_type = self.check_expression(&infix_expr.rhs);

                let lhs_span = self.interner.expr_span(&infix_expr.lhs);
                let rhs_span = self.interner.expr_span(&infix_expr.rhs);
                let span = lhs_span.merge(rhs_span);

                self.infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type, span)
                    .unwrap_or_else(|error| {
                        self.errors.push(error);
                        Type::Error
                    })
            }
            HirExpression::Index(index_expr) => self.check_index_expression(index_expr),
            HirExpression::Call(call_expr) => {
                self.check_if_deprecated(&call_expr.func);

                let function = self.check_expression(&call_expr.func);
                let args = vecmap(&call_expr.arguments, |arg| {
                    let typ = self.check_expression(arg);
                    (typ, *arg, self.interner.expr_span(arg))
                });
                let span = self.interner.expr_span(expr_id);
                self.bind_function_type(function, args, span)
            }
            HirExpression::MethodCall(mut method_call) => {
                let object_type = self.check_expression(&method_call.object).follow_bindings();
                let method_name = method_call.method.0.contents.as_str();
                match self.lookup_method(&object_type, method_name, expr_id) {
                    Some(method_id) => {
                        let mut args = vec![(
                            object_type,
                            method_call.object,
                            self.interner.expr_span(&method_call.object),
                        )];

                        for arg in &method_call.arguments {
                            let typ = self.check_expression(arg);
                            args.push((typ, *arg, self.interner.expr_span(arg)));
                        }

                        // Desugar the method call into a normal, resolved function call
                        // so that the backend doesn't need to worry about methods
                        let location = method_call.location;

                        // Automatically add `&mut` if the method expects a mutable reference and
                        // the object is not already one.
                        if method_id != FuncId::dummy_id() {
                            let func_meta = self.interner.function_meta(&method_id);
                            self.try_add_mutable_reference_to_object(
                                &mut method_call,
                                &func_meta.typ,
                                &mut args,
                            );
                        }

                        let (function_id, function_call) =
                            method_call.into_function_call(method_id, location, self.interner);

                        let span = self.interner.expr_span(expr_id);
                        let ret = self.check_method_call(&function_id, &method_id, args, span);

                        self.interner.replace_expr(expr_id, function_call);
                        ret
                    }
                    None => Type::Error,
                }
            }
            HirExpression::Cast(cast_expr) => {
                // Evaluate the LHS
                let lhs_type = self.check_expression(&cast_expr.lhs);
                let span = self.interner.expr_span(expr_id);
                self.check_cast(lhs_type, cast_expr.r#type, span)
            }
            HirExpression::For(for_expr) => {
                let start_range_type = self.check_expression(&for_expr.start_range);
                let end_range_type = self.check_expression(&for_expr.end_range);

                let start_span = self.interner.expr_span(&for_expr.start_range);
                let end_span = self.interner.expr_span(&for_expr.end_range);

                // Check that start range and end range have the same types
                let range_span = start_span.merge(end_span);
                self.unify(&start_range_type, &end_range_type, range_span, || {
                    TypeCheckError::TypeMismatch {
                        expected_typ: start_range_type.to_string(),
                        expr_typ: end_range_type.to_string(),
                        expr_span: range_span,
                    }
                });

                let expected_comptime = if self.is_unconstrained() {
                    CompTime::new(self.interner)
                } else {
                    CompTime::Yes(Some(range_span))
                };
                let fresh_id = self.interner.next_type_variable_id();
                let type_variable = Shared::new(TypeBinding::Unbound(fresh_id));
                let expected_type = Type::TypeVariable(
                    type_variable,
                    TypeVariableKind::IntegerOrField(expected_comptime),
                );

                self.unify(&start_range_type, &expected_type, range_span, || {
                    TypeCheckError::TypeCannotBeUsed {
                        typ: start_range_type.clone(),
                        place: "for loop",
                        span: range_span,
                    }
                    .add_context("The range of a loop must be known at compile-time")
                });

                self.interner.push_definition_type(for_expr.identifier.id, start_range_type);

                self.check_expression(&for_expr.block);
                Type::Unit
            }
            HirExpression::Block(block_expr) => {
                let mut block_type = Type::Unit;

                let statements = block_expr.statements();
                for (i, stmt) in statements.iter().enumerate() {
                    let expr_type = self.check_statement(stmt);

                    if i + 1 < statements.len() {
                        let id = match self.interner.statement(stmt) {
                            crate::hir_def::stmt::HirStatement::Expression(expr) => expr,
                            _ => *expr_id,
                        };

                        let span = self.interner.expr_span(&id);
                        self.unify(&expr_type, &Type::Unit, span, || {
                            TypeCheckError::TypeMismatch {
                                expected_typ: Type::Unit.to_string(),
                                expr_typ: expr_type.to_string(),
                                expr_span: span,
                            }
                        });
                    } else {
                        block_type = expr_type;
                    }
                }

                block_type
            }
            HirExpression::Prefix(prefix_expr) => {
                let rhs_type = self.check_expression(&prefix_expr.rhs);
                let span = self.interner.expr_span(&prefix_expr.rhs);
                self.type_check_prefix_operand(&prefix_expr.operator, &rhs_type, span)
            }
            HirExpression::If(if_expr) => self.check_if_expr(&if_expr, expr_id),
            HirExpression::Constructor(constructor) => self.check_constructor(constructor, expr_id),
            HirExpression::MemberAccess(access) => self.check_member_access(access, *expr_id),
            HirExpression::Error => Type::Error,
            HirExpression::Tuple(elements) => {
                Type::Tuple(vecmap(&elements, |elem| self.check_expression(elem)))
            }
            HirExpression::Lambda(lambda) => {
                let params = vecmap(lambda.parameters, |(pattern, typ)| {
                    self.bind_pattern(&pattern, typ.clone());
                    typ
                });

                let actual_return = self.check_expression(&lambda.body);

                let span = self.interner.expr_span(&lambda.body);
                actual_return.make_subtype_of(&lambda.return_type, span, &mut self.errors, || {
                    TypeCheckError::TypeMismatch {
                        expected_typ: lambda.return_type.to_string(),
                        expr_typ: actual_return.to_string(),
                        expr_span: span,
                    }
                });
                Type::Function(params, Box::new(lambda.return_type))
            }
        };

        self.interner.push_expr_type(expr_id, typ.clone());
        typ
    }

    /// Check if the given method type requires a mutable reference to the object type, and check
    /// if the given object type is already a mutable reference. If not, add one.
    /// This is used to automatically transform a method call: `foo.bar()` into a function
    /// call: `bar(&mut foo)`.
    ///
    /// A notable corner case of this function is where it interacts with auto-deref of `.`.
    /// If a field is being mutated e.g. `foo.bar.mutate_bar()` where `foo: &mut Foo`, the compiler
    /// will insert a dereference before bar `(*foo).bar.mutate_bar()` which would cause us to
    /// mutate a copy of bar rather than a reference to it. We must check for this corner case here
    /// and remove the implicitly added dereference operator if we find one.
    fn try_add_mutable_reference_to_object(
        &mut self,
        method_call: &mut HirMethodCallExpression,
        function_type: &Type,
        argument_types: &mut [(Type, ExprId, noirc_errors::Span)],
    ) {
        let expected_object_type = match function_type {
            Type::Function(args, _) => args.get(0),
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _) => args.get(0),
                typ => unreachable!("Unexpected type for function: {typ}"),
            },
            typ => unreachable!("Unexpected type for function: {typ}"),
        };

        if let Some(expected_object_type) = expected_object_type {
            if matches!(expected_object_type.follow_bindings(), Type::MutableReference(_)) {
                let actual_type = argument_types[0].0.follow_bindings();

                if !matches!(actual_type, Type::MutableReference(_)) {
                    if let Err(error) = verify_mutable_reference(self.interner, method_call.object)
                    {
                        self.errors.push(TypeCheckError::ResolverError(error));
                    }

                    let new_type = Type::MutableReference(Box::new(actual_type));
                    argument_types[0].0 = new_type.clone();

                    // First try to remove a dereference operator that may have been implicitly
                    // inserted by a field access expression `foo.bar` on a mutable reference `foo`.
                    if self.try_remove_implicit_dereference(method_call.object).is_none() {
                        // If that didn't work, then wrap the whole expression in an `&mut`
                        method_call.object =
                            self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                                operator: UnaryOp::MutableReference,
                                rhs: method_call.object,
                            }));
                        self.interner.push_expr_type(&method_call.object, new_type);
                    }
                }
            }
        }
    }

    /// Given a method object: `(*foo).bar` of a method call `(*foo).bar.baz()`, remove the
    /// implicitly added dereference operator if one is found.
    ///
    /// Returns Some(()) if a dereference was removed and None otherwise.
    fn try_remove_implicit_dereference(&mut self, object: ExprId) -> Option<()> {
        match self.interner.expression(&object) {
            HirExpression::MemberAccess(access) => {
                self.try_remove_implicit_dereference(access.lhs)?;

                // Since we removed a dereference, instead of returning the field directly,
                // we expect to be returning a reference to the field, so update the type accordingly.
                let current_type = self.interner.id_type(object);
                let reference_type = Type::MutableReference(Box::new(current_type));
                self.interner.push_expr_type(&object, reference_type);
                Some(())
            }
            HirExpression::Prefix(prefix) => match prefix.operator {
                UnaryOp::Dereference { implicitly_added: true } => {
                    // Found a dereference we can remove. Now just replace it with its rhs to remove it.
                    let rhs = self.interner.expression(&prefix.rhs);
                    self.interner.replace_expr(&object, rhs);

                    let rhs_type = self.interner.id_type(prefix.rhs);
                    self.interner.push_expr_type(&object, rhs_type);
                    Some(())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn check_index_expression(&mut self, index_expr: expr::HirIndexExpression) -> Type {
        let index_type = self.check_expression(&index_expr.index);
        let span = self.interner.expr_span(&index_expr.index);

        index_type.unify(&Type::polymorphic_integer(self.interner), span, &mut self.errors, || {
            TypeCheckError::TypeMismatch {
                expected_typ: "an integer".to_owned(),
                expr_typ: index_type.to_string(),
                expr_span: span,
            }
        });

        let lhs_type = self.check_expression(&index_expr.collection);
        match lhs_type {
            // XXX: We can check the array bounds here also, but it may be better to constant fold first
            // and have ConstId instead of ExprId for constants
            Type::Array(_, base_type) => *base_type,
            Type::Error => Type::Error,
            typ => {
                let span = self.interner.expr_span(&index_expr.collection);
                self.errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: "Array".to_owned(),
                    expr_typ: typ.to_string(),
                    expr_span: span,
                });
                Type::Error
            }
        }
    }

    fn check_cast(&mut self, from: Type, to: Type, span: Span) -> Type {
        let is_comp_time = match from.follow_bindings() {
            Type::Integer(is_comp_time, ..) => is_comp_time,
            Type::FieldElement(is_comp_time) => is_comp_time,
            Type::TypeVariable(_, TypeVariableKind::IntegerOrField(is_comp_time)) => is_comp_time,
            Type::TypeVariable(_, _) => {
                self.errors.push(TypeCheckError::TypeAnnotationsNeeded { span });
                return Type::Error;
            }
            Type::Bool(is_comp_time) => is_comp_time,
            Type::Error => return Type::Error,
            from => {
                self.errors.push(TypeCheckError::InvalidCast { from, span });
                return Type::Error;
            }
        };

        match to {
            Type::Integer(dest_comp_time, sign, bits) => {
                if dest_comp_time.is_comp_time()
                    && is_comp_time.unify(&dest_comp_time, span).is_err()
                {
                    self.errors.push(TypeCheckError::CannotCastToComptimeType { span });
                }

                Type::Integer(is_comp_time, sign, bits)
            }
            Type::FieldElement(dest_comp_time) => {
                if dest_comp_time.is_comp_time()
                    && is_comp_time.unify(&dest_comp_time, span).is_err()
                {
                    self.errors.push(TypeCheckError::CannotCastToComptimeType { span });
                }

                Type::FieldElement(is_comp_time)
            }
            Type::Bool(dest_comp_time) => {
                if dest_comp_time.is_comp_time()
                    && is_comp_time.unify(&dest_comp_time, span).is_err()
                {
                    self.errors.push(TypeCheckError::CannotCastToComptimeType { span });
                }
                Type::Bool(dest_comp_time)
            }
            Type::Error => Type::Error,
            _ => {
                self.errors.push(TypeCheckError::UnsupportedCast { span });
                Type::Error
            }
        }
    }

    // We need a special function to type check method calls since the method
    // is not a Expression::Ident it must be manually instantiated here
    fn check_method_call(
        &mut self,
        function_ident_id: &ExprId,
        func_id: &FuncId,
        arguments: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        if func_id == &FuncId::dummy_id() {
            Type::Error
        } else {
            let func_meta = self.interner.function_meta(func_id);

            // Check function call arity is correct
            let param_len = func_meta.parameters.len();
            let arg_len = arguments.len();

            if param_len != arg_len {
                self.errors.push(TypeCheckError::ArityMisMatch {
                    expected: param_len as u16,
                    found: arg_len as u16,
                    span,
                });
            }

            let (function_type, instantiation_bindings) = func_meta.typ.instantiate(self.interner);

            self.interner.store_instantiation_bindings(*function_ident_id, instantiation_bindings);
            self.interner.push_expr_type(function_ident_id, function_type.clone());

            self.bind_function_type(function_type, arguments, span)
        }
    }

    fn check_if_expr(&mut self, if_expr: &expr::HirIfExpression, expr_id: &ExprId) -> Type {
        let cond_type = self.check_expression(&if_expr.condition);
        let then_type = self.check_expression(&if_expr.consequence);

        let expr_span = self.interner.expr_span(&if_expr.condition);

        let bool_type = Type::Bool(CompTime::new(self.interner));
        self.unify(&cond_type, &bool_type, expr_span, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool(CompTime::No(None)).to_string(),
            expr_typ: cond_type.to_string(),
            expr_span,
        });

        match if_expr.alternative {
            None => Type::Unit,
            Some(alternative) => {
                let else_type = self.check_expression(&alternative);

                let expr_span = self.interner.expr_span(expr_id);
                self.unify(&then_type, &else_type, expr_span, || {
                    let err = TypeCheckError::TypeMismatch {
                        expected_typ: then_type.to_string(),
                        expr_typ: else_type.to_string(),
                        expr_span,
                    };

                    let context = if then_type == Type::Unit {
                        "Are you missing a semicolon at the end of your 'else' branch?"
                    } else if else_type == Type::Unit {
                        "Are you missing a semicolon at the end of the first block of this 'if'?"
                    } else {
                        "Expected the types of both if branches to be equal"
                    };

                    err.add_context(context)
                });

                then_type
            }
        }
    }

    fn check_constructor(
        &mut self,
        constructor: expr::HirConstructorExpression,
        expr_id: &ExprId,
    ) -> Type {
        let typ = constructor.r#type;
        let generics = constructor.struct_generics;

        // Sort argument types by name so we can zip with the struct type in the same ordering.
        // Note that we use a Vec to store the original arguments (rather than a BTreeMap) to
        // preserve the evaluation order of the source code.
        let mut args = constructor.fields;
        sort_by_key_ref(&mut args, |(name, _)| name);

        let mut fields = typ.borrow().get_fields(&generics);
        sort_by_key_ref(&mut fields, |(name, _)| name);

        for ((param_name, param_type), (arg_ident, arg)) in fields.into_iter().zip(args) {
            // This can be false if the user provided an incorrect field count. That error should
            // be caught during name resolution so it is fine to skip typechecking if there is a
            // mismatch here as long as we continue typechecking the rest of the program to the best
            // of our ability.
            if param_name == arg_ident.0.contents {
                let arg_type = self.check_expression(&arg);

                let span = self.interner.expr_span(expr_id);
                self.make_subtype_of(&arg_type, &param_type, arg, || {
                    TypeCheckError::TypeMismatch {
                        expected_typ: param_type.to_string(),
                        expr_typ: arg_type.to_string(),
                        expr_span: span,
                    }
                });
            }
        }

        Type::Struct(typ, generics)
    }

    fn check_member_access(&mut self, mut access: expr::HirMemberAccess, expr_id: ExprId) -> Type {
        let lhs_type = self.check_expression(&access.lhs).follow_bindings();
        let span = self.interner.expr_span(&expr_id);
        let access_lhs = &mut access.lhs;

        let dereference_lhs = |this: &mut Self, lhs_type, element| {
            let old_lhs = *access_lhs;
            *access_lhs = this.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: crate::UnaryOp::Dereference { implicitly_added: true },
                rhs: old_lhs,
            }));
            this.interner.push_expr_type(&old_lhs, lhs_type);
            this.interner.push_expr_type(access_lhs, element);
        };

        match self.check_field_access(&lhs_type, &access.rhs.0.contents, span, dereference_lhs) {
            Some((element_type, index)) => {
                self.interner.set_field_index(expr_id, index);
                // We must update `access` in case we added any dereferences to it
                self.interner.replace_expr(&expr_id, HirExpression::MemberAccess(access));
                element_type
            }
            None => Type::Error,
        }
    }

    /// This will verify that an expression in the form `lhs.rhs_name` has the given field and will push
    /// a type error if it does not. If there is no error, the type of the struct/tuple field is returned
    /// along with the index of the field in question.
    ///
    /// This function is abstracted from check_member_access so that it can be shared between
    /// there and the HirLValue::MemberAccess case of check_lvalue.
    ///
    /// `dereference_lhs` is called when the lhs type is a Type::MutableReference that should be
    /// automatically dereferenced so its field can be extracted. This function is expected to
    /// perform any mutations necessary to wrap the lhs in a UnaryOp::Dereference prefix
    /// expression. The second parameter of this function represents the lhs_type (which should
    /// always be a Type::MutableReference if `dereference_lhs` is called) and the third
    /// represents the element type.
    pub(super) fn check_field_access(
        &mut self,
        lhs_type: &Type,
        field_name: &str,
        span: Span,
        mut dereference_lhs: impl FnMut(&mut Self, Type, Type),
    ) -> Option<(Type, usize)> {
        let lhs_type = lhs_type.follow_bindings();

        match &lhs_type {
            Type::Struct(s, args) => {
                let s = s.borrow();
                if let Some((field, index)) = s.get_field(field_name, args) {
                    return Some((field, index));
                }
            }
            Type::Tuple(elements) => {
                if let Ok(index) = field_name.parse::<usize>() {
                    let length = elements.len();
                    if index < length {
                        return Some((elements[index].clone(), index));
                    } else {
                        self.errors.push(TypeCheckError::TupleIndexOutOfBounds {
                            index,
                            lhs_type,
                            length,
                            span,
                        });
                        return None;
                    }
                }
            }
            // If the lhs is a mutable reference we automatically transform
            // lhs.field into (*lhs).field
            Type::MutableReference(element) => {
                dereference_lhs(self, lhs_type.clone(), element.as_ref().clone());
                return self.check_field_access(element, field_name, span, dereference_lhs);
            }
            _ => (),
        }

        // If we get here the type has no field named 'access.rhs'.
        // Now we specialize the error message based on whether we know the object type in question yet.
        if let Type::TypeVariable(..) = &lhs_type {
            self.errors.push(TypeCheckError::TypeAnnotationsNeeded { span });
        } else if lhs_type != Type::Error {
            self.errors.push(TypeCheckError::AccessUnknownMember {
                lhs_type,
                field_name: field_name.to_string(),
                span,
            });
        }

        None
    }

    fn comparator_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        rhs_type: &Type,
        op: &HirBinaryOp,
        span: Span,
    ) -> Result<Type, TypeCheckError> {
        use crate::BinaryOpKind::{Equal, NotEqual};
        use Type::*;

        match (lhs_type, rhs_type) {
            // Avoid reporting errors multiple times
            (Error, _) | (_, Error) => Ok(Bool(CompTime::Yes(None))),

            // Matches on TypeVariable must be first to follow any type
            // bindings.
            (var @ TypeVariable(int, _), other) | (other, var @ TypeVariable(int, _)) => {
                if let TypeBinding::Bound(binding) = &*int.borrow() {
                    return self.comparator_operand_type_rules(other, binding, op, span);
                }

                if !op.kind.is_valid_for_field_type() && (other.is_bindable() || other.is_field()) {
                    let other = other.follow_bindings();

                    self.push_delayed_type_check(Box::new(move || {
                        if other.is_field() || other.is_bindable() {
                            Err(TypeCheckError::InvalidComparisonOnField { span })
                        } else {
                            Ok(())
                        }
                    }));
                }

                let comptime = var.try_get_comptime();
                if other.try_bind_to_polymorphic_int(int, &comptime, true, op.location.span).is_ok()
                    || other == &Type::Error
                {
                    Ok(Bool(comptime.into_owned()))
                } else {
                    Err(TypeCheckError::TypeMismatchWithSource {
                        rhs: lhs_type.clone(),
                        lhs: rhs_type.clone(),
                        span,
                        source: Source::Binary,
                    })
                }
            }
            (
                Integer(comptime_x, sign_x, bit_width_x),
                Integer(comptime_y, sign_y, bit_width_y),
            ) => {
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        span,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        span,
                    });
                }
                let comptime = comptime_x.and(comptime_y, op.location.span);
                Ok(Bool(comptime))
            }
            (Integer(..), FieldElement(..)) | (FieldElement(..), Integer(..)) => {
                Err(TypeCheckError::IntegerAndFieldBinaryOperation { span })
            }
            (Integer(..), typ) | (typ, Integer(..)) => {
                Err(TypeCheckError::IntegerTypeMismatch { typ: typ.clone(), span })
            }
            (FieldElement(comptime_x), FieldElement(comptime_y)) => {
                if op.kind.is_valid_for_field_type() {
                    let comptime = comptime_x.and(comptime_y, op.location.span);
                    Ok(Bool(comptime))
                } else {
                    Err(TypeCheckError::FieldComparison { span })
                }
            }

            // <= and friends are technically valid for booleans, just not very useful
            (Bool(comptime_x), Bool(comptime_y)) => {
                let comptime = comptime_x.and(comptime_y, op.location.span);
                Ok(Bool(comptime))
            }

            // Special-case == and != for arrays
            (Array(x_size, x_type), Array(y_size, y_type))
                if matches!(op.kind, Equal | NotEqual) =>
            {
                x_type.unify(y_type, op.location.span, &mut self.errors, || {
                    TypeCheckError::TypeMismatchWithSource {
                        rhs: lhs_type.clone(),
                        lhs: rhs_type.clone(),
                        source: Source::ArrayElements,
                        span: op.location.span,
                    }
                });

                self.unify(x_size, y_size, op.location.span, || {
                    TypeCheckError::TypeMismatchWithSource {
                        rhs: lhs_type.clone(),
                        lhs: rhs_type.clone(),
                        source: Source::ArrayLen,
                        span: op.location.span,
                    }
                });

                // We could check if all elements of all arrays are comptime but I am lazy
                Ok(Bool(CompTime::No(Some(op.location.span))))
            }
            (lhs @ NamedGeneric(binding_a, _), rhs @ NamedGeneric(binding_b, _)) => {
                if binding_a == binding_b {
                    return Ok(Bool(CompTime::No(Some(op.location.span))));
                }
                Err(TypeCheckError::TypeMismatchWithSource {
                    rhs: lhs.clone(),
                    lhs: rhs.clone(),
                    source: Source::Comparison,
                    span,
                })
            }
            (String(x_size), String(y_size)) => {
                x_size.unify(y_size, op.location.span, &mut self.errors, || {
                    TypeCheckError::TypeMismatchWithSource {
                        rhs: *x_size.clone(),
                        lhs: *y_size.clone(),
                        span: op.location.span,
                        source: Source::StringLen,
                    }
                });

                Ok(Bool(CompTime::No(Some(op.location.span))))
            }
            (lhs, rhs) => Err(TypeCheckError::TypeMismatchWithSource {
                rhs: lhs.clone(),
                lhs: rhs.clone(),
                source: Source::Comparison,
                span,
            }),
        }
    }

    fn lookup_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        expr_id: &ExprId,
    ) -> Option<FuncId> {
        match object_type {
            Type::Struct(typ, _args) => {
                match self.interner.lookup_method(typ.borrow().id, method_name) {
                    Some(method_id) => Some(method_id),
                    None => {
                        self.errors.push(TypeCheckError::UnresolvedMethodCall {
                            method_name: method_name.to_string(),
                            object_type: object_type.clone(),
                            span: self.interner.expr_span(expr_id),
                        });
                        None
                    }
                }
            }
            // Mutable references to another type should resolve to methods of their element type.
            // This may be a struct or a primitive type.
            Type::MutableReference(element) => self.lookup_method(element, method_name, expr_id),
            // If we fail to resolve the object to a struct type, we have no way of type
            // checking its arguments as we can't even resolve the name of the function
            Type::Error => None,

            // In the future we could support methods for non-struct types if we have a context
            // (in the interner?) essentially resembling HashMap<Type, Methods>
            other => match self.interner.lookup_primitive_method(other, method_name) {
                Some(method_id) => Some(method_id),
                None => {
                    self.errors.push(TypeCheckError::UnresolvedMethodCall {
                        method_name: method_name.to_string(),
                        object_type: object_type.clone(),
                        span: self.interner.expr_span(expr_id),
                    });
                    None
                }
            },
        }
    }

    fn bind_function_type(
        &mut self,
        function: Type,
        args: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        // Could do a single unification for the entire function type, but matching beforehand
        // lets us issue a more precise error on the individual argument that fails to type check.
        match function {
            Type::TypeVariable(binding, TypeVariableKind::Normal) => {
                if let TypeBinding::Bound(typ) = &*binding.borrow() {
                    return self.bind_function_type(typ.clone(), args, span);
                }

                let ret = self.interner.next_type_variable();
                let args = vecmap(args, |(arg, _, _)| arg);
                let expected = Type::Function(args, Box::new(ret.clone()));

                if let Err(error) = binding.borrow_mut().bind_to(expected, span) {
                    self.errors.push(error);
                }
                ret
            }
            Type::Function(parameters, ret) => {
                if parameters.len() != args.len() {
                    self.errors.push(TypeCheckError::ParameterCountMismatch {
                        expected: parameters.len(),
                        found: args.len(),
                        span,
                    });
                    return Type::Error;
                }

                for (param, (arg, arg_id, arg_span)) in parameters.iter().zip(args) {
                    arg.make_subtype_with_coercions(
                        param,
                        arg_id,
                        self.interner,
                        &mut self.errors,
                        || TypeCheckError::TypeMismatch {
                            expected_typ: param.to_string(),
                            expr_typ: arg.to_string(),
                            expr_span: arg_span,
                        },
                    );
                }

                *ret
            }
            Type::Error => Type::Error,
            found => {
                self.errors.push(TypeCheckError::ExpectedFunction { found, span });
                Type::Error
            }
        }
    }

    // Given a binary operator and another type. This method will produce the output type
    // XXX: Review these rules. In particular, the interaction between integers, comptime and private/public variables
    fn infix_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Result<Type, TypeCheckError> {
        if op.kind.is_comparator() {
            return self.comparator_operand_type_rules(lhs_type, rhs_type, op, span);
        }

        use Type::*;
        match (lhs_type, rhs_type) {
            // An error type on either side will always return an error
            (Error, _) | (_, Error) => Ok(Error),

            // Matches on TypeVariable must be first so that we follow any type
            // bindings.
            (var @ TypeVariable(int, _), other) | (other, var @ TypeVariable(int, _)) => {
                if let TypeBinding::Bound(binding) = &*int.borrow() {
                    return self.infix_operand_type_rules(binding, op, other, span);
                }

                if op.is_bitwise() && (other.is_bindable() || other.is_field()) {
                    let other = other.follow_bindings();

                    // This will be an error if these types later resolve to a Field, or stay
                    // polymorphic as the bit size will be unknown. Delay this error until the function
                    // finishes resolving so we can still allow cases like `let x: u8 = 1 << 2;`.
                    self.push_delayed_type_check(Box::new(move || {
                        if other.is_field() {
                            Err(TypeCheckError::InvalidBitwiseOperationOnField { span })
                        } else if other.is_bindable() {
                            Err(TypeCheckError::AmbiguousBitWidth { span })
                        } else {
                            Ok(())
                        }
                    }));
                }

                let comptime = var.try_get_comptime();
                if other.try_bind_to_polymorphic_int(int, &comptime, true, op.location.span).is_ok()
                    || other == &Type::Error
                {
                    Ok(other.clone())
                } else {
                    Err(TypeCheckError::TypeMismatchWithSource {
                        rhs: lhs_type.clone(),
                        lhs: rhs_type.clone(),
                        source: Source::Binary,
                        span,
                    })
                }
            }
            (
                Integer(comptime_x, sign_x, bit_width_x),
                Integer(comptime_y, sign_y, bit_width_y),
            ) => {
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        span,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        span,
                    });
                }
                let comptime = comptime_x.and(comptime_y, op.location.span);
                Ok(Integer(comptime, *sign_x, *bit_width_x))
            }
            (Integer(..), FieldElement(..)) | (FieldElement(..), Integer(..)) => {
                Err(TypeCheckError::IntegerAndFieldBinaryOperation { span })
            }
            (Integer(..), typ) | (typ, Integer(..)) => {
                Err(TypeCheckError::IntegerTypeMismatch { typ: typ.clone(), span })
            }
            // These types are not supported in binary operations
            (Array(..), _) | (_, Array(..)) => {
                Err(TypeCheckError::InvalidInfixOp { kind: "Arrays", span })
            }
            (Struct(..), _) | (_, Struct(..)) => {
                Err(TypeCheckError::InvalidInfixOp { kind: "Structs", span })
            }
            (Tuple(_), _) | (_, Tuple(_)) => {
                Err(TypeCheckError::InvalidInfixOp { kind: "Tuples", span })
            }

            (Unit, _) | (_, Unit) => Ok(Unit),

            // The result of two Fields is always a witness
            (FieldElement(comptime_x), FieldElement(comptime_y)) => {
                if op.is_bitwise() {
                    return Err(TypeCheckError::InvalidBitwiseOperationOnField { span });
                }
                let comptime = comptime_x.and(comptime_y, op.location.span);
                Ok(FieldElement(comptime))
            }

            (Bool(comptime_x), Bool(comptime_y)) => {
                Ok(Bool(comptime_x.and(comptime_y, op.location.span)))
            }

            (lhs, rhs) => Err(TypeCheckError::TypeMismatchWithSource {
                rhs: lhs.clone(),
                lhs: rhs.clone(),
                source: Source::BinOp,
                span,
            }),
        }
    }

    fn type_check_prefix_operand(
        &mut self,
        op: &crate::UnaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Type {
        let mut unify = |expected| {
            rhs_type.unify(&expected, span, &mut self.errors, || TypeCheckError::TypeMismatch {
                expr_typ: rhs_type.to_string(),
                expected_typ: expected.to_string(),
                expr_span: span,
            });
            expected
        };

        match op {
            crate::UnaryOp::Minus => unify(Type::polymorphic_integer(self.interner)),
            crate::UnaryOp::Not => {
                let rhs_type = rhs_type.follow_bindings();

                // `!` can work on booleans or integers
                if matches!(rhs_type, Type::Integer(..)) {
                    return rhs_type;
                }

                unify(Type::Bool(CompTime::new(self.interner)))
            }
            crate::UnaryOp::MutableReference => {
                Type::MutableReference(Box::new(rhs_type.follow_bindings()))
            }
            crate::UnaryOp::Dereference { implicitly_added: _ } => {
                let element_type = self.interner.next_type_variable();
                unify(Type::MutableReference(Box::new(element_type.clone())));
                element_type
            }
        }
    }
}

/// Taken from: https://stackoverflow.com/a/47127500
fn sort_by_key_ref<T, F, K>(xs: &mut [T], key: F)
where
    F: Fn(&T) -> &K,
    K: ?Sized + Ord,
{
    xs.sort_by(|x, y| key(x).cmp(key(y)));
}
