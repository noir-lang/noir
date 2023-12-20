use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    hir::{resolution::resolver::verify_mutable_reference, type_check::errors::Source},
    hir_def::{
        expr::{
            self, HirArrayLiteral, HirBinaryOp, HirExpression, HirLiteral, HirMethodCallExpression,
            HirMethodReference, HirPrefixExpression,
        },
        types::Type,
    },
    node_interner::{DefinitionKind, ExprId, FuncId, TraitId, TraitImplKind, TraitMethodId},
    BinaryOpKind, TypeBinding, TypeBindings, TypeVariableKind, UnaryOp,
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
                let attributes = self.interner.function_attributes(func_id);
                if let Some(note) = attributes.get_deprecated_note() {
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
                let t = self.interner.id_type_substitute_trait_as_type(ident.id);
                let (typ, bindings) = t.instantiate(self.interner);

                // Push any trait constraints required by this definition to the context
                // to be checked later when the type of this variable is further constrained.
                if let Some(definition) = self.interner.try_definition(ident.id) {
                    if let DefinitionKind::Function(function) = definition.kind {
                        let function = self.interner.function_meta(&function);
                        for mut constraint in function.trait_constraints.clone() {
                            constraint.typ = constraint.typ.substitute(&bindings);
                            self.trait_constraints.push((constraint, *expr_id));
                        }
                    }
                }

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

                            elem_type.unify(&first_elem_type, &mut self.errors, || {
                                TypeCheckError::NonHomogeneousArray {
                                    first_span: self.interner.expr_location(&arr[0]).span,
                                    first_type: first_elem_type.to_string(),
                                    first_index: index,
                                    second_span: location.span,
                                    second_type: elem_type.to_string(),
                                    second_index: index + 1,
                                }
                                .add_context("elements in an array must have the same type")
                            });
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
                    HirLiteral::Bool(_) => Type::Bool,
                    HirLiteral::Integer(_, _) => Type::polymorphic_integer(self.interner),
                    HirLiteral::Str(string) => {
                        let len = Type::Constant(string.len() as u64);
                        Type::String(Box::new(len))
                    }
                    HirLiteral::FmtStr(string, idents) => {
                        let len = Type::Constant(string.len() as u64);
                        let types = vecmap(&idents, |elem| self.check_expression(elem));
                        Type::FmtString(Box::new(len), Box::new(Type::Tuple(types)))
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
            HirExpression::Index(index_expr) => self.check_index_expression(expr_id, index_expr),
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
                    Some(method_ref) => {
                        let mut args = vec![(
                            object_type.clone(),
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

                        let trait_id = match &method_ref {
                            HirMethodReference::FuncId(func_id) => {
                                // Automatically add `&mut` if the method expects a mutable reference and
                                // the object is not already one.
                                if *func_id != FuncId::dummy_id() {
                                    let func_meta = self.interner.function_meta(func_id);
                                    self.try_add_mutable_reference_to_object(
                                        &mut method_call,
                                        &func_meta.typ,
                                        &mut args,
                                    );
                                }

                                let meta = self.interner.function_meta(func_id);
                                meta.trait_impl.map(|impl_id| {
                                    self.interner
                                        .get_trait_implementation(impl_id)
                                        .borrow()
                                        .trait_id
                                })
                            }
                            HirMethodReference::TraitMethodId(method) => Some(method.trait_id),
                        };

                        let (function_id, function_call) = method_call.into_function_call(
                            method_ref.clone(),
                            location,
                            self.interner,
                        );

                        let span = self.interner.expr_span(expr_id);
                        let ret = self.check_method_call(&function_id, method_ref, args, span);

                        if let Some(trait_id) = trait_id {
                            self.verify_trait_constraint(&object_type, trait_id, function_id, span);
                        }

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
            HirExpression::Block(block_expr) => {
                let mut block_type = Type::Unit;

                let statements = block_expr.statements();
                for (i, stmt) in statements.iter().enumerate() {
                    let expr_type = self.check_statement(stmt);

                    if let crate::hir_def::stmt::HirStatement::Semi(expr) =
                        self.interner.statement(stmt)
                    {
                        let inner_expr_type = self.interner.id_type(expr);
                        let span = self.interner.expr_span(&expr);

                        self.unify(&inner_expr_type, &Type::Unit, || {
                            TypeCheckError::UnusedResultError {
                                expr_type: inner_expr_type.clone(),
                                expr_span: span,
                            }
                        });
                    }

                    if i + 1 == statements.len() {
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
                let captured_vars =
                    vecmap(lambda.captures, |capture| self.interner.id_type(capture.ident.id));

                let env_type: Type =
                    if captured_vars.is_empty() { Type::Unit } else { Type::Tuple(captured_vars) };

                let params = vecmap(lambda.parameters, |(pattern, typ)| {
                    self.bind_pattern(&pattern, typ.clone());
                    typ
                });

                let actual_return = self.check_expression(&lambda.body);

                let span = self.interner.expr_span(&lambda.body);
                self.unify(&actual_return, &lambda.return_type, || TypeCheckError::TypeMismatch {
                    expected_typ: lambda.return_type.to_string(),
                    expr_typ: actual_return.to_string(),
                    expr_span: span,
                });

                Type::Function(params, Box::new(lambda.return_type), Box::new(env_type))
            }
            HirExpression::TraitMethodReference(method) => {
                let the_trait = self.interner.get_trait(method.trait_id);
                let typ2 = &the_trait.methods[method.method_index].typ;
                let (typ, mut bindings) = typ2.instantiate(self.interner);

                // We must also remember to apply these substitutions to the object_type
                // referenced by the selected trait impl, if one has yet to be selected.
                let impl_kind = self.interner.get_selected_impl_for_ident(*expr_id);
                if let Some(TraitImplKind::Assumed { object_type }) = impl_kind {
                    let the_trait = self.interner.get_trait(method.trait_id);
                    let object_type = object_type.substitute(&bindings);
                    bindings.insert(
                        the_trait.self_type_typevar_id,
                        (the_trait.self_type_typevar.clone(), object_type.clone()),
                    );
                    self.interner
                        .select_impl_for_ident(*expr_id, TraitImplKind::Assumed { object_type });
                }

                self.interner.store_instantiation_bindings(*expr_id, bindings);
                typ
            }
        };

        self.interner.push_expr_type(expr_id, typ.clone());
        typ
    }

    pub fn verify_trait_constraint(
        &mut self,
        object_type: &Type,
        trait_id: TraitId,
        function_ident_id: ExprId,
        span: Span,
    ) {
        match self.interner.lookup_trait_implementation(object_type, trait_id) {
            Ok(impl_kind) => self.interner.select_impl_for_ident(function_ident_id, impl_kind),
            Err(erroring_constraints) => {
                // Don't show any errors where try_get_trait returns None.
                // This can happen if a trait is used that was never declared.
                let constraints = erroring_constraints
                    .into_iter()
                    .map(|constraint| {
                        let r#trait = self.interner.try_get_trait(constraint.trait_id)?;
                        Some((constraint.typ, r#trait.name.to_string()))
                    })
                    .collect::<Option<Vec<_>>>();

                if let Some(constraints) = constraints {
                    self.errors.push(TypeCheckError::NoMatchingImplFound { constraints, span });
                }
            }
        }
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
            Type::Function(args, _, _) => args.first(),
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _, _) => args.first(),
                typ => unreachable!("Unexpected type for function: {typ}"),
            },
            typ => unreachable!("Unexpected type for function: {typ}"),
        };

        if let Some(expected_object_type) = expected_object_type {
            let actual_type = argument_types[0].0.follow_bindings();

            if matches!(expected_object_type.follow_bindings(), Type::MutableReference(_)) {
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
                        let location = self.interner.id_location(method_call.object);

                        method_call.object =
                            self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                                operator: UnaryOp::MutableReference,
                                rhs: method_call.object,
                            }));
                        self.interner.push_expr_type(&method_call.object, new_type);
                        self.interner.push_expr_location(
                            method_call.object,
                            location.span,
                            location.file,
                        );
                    }
                }
            // Otherwise if the object type is a mutable reference and the method is not, insert as
            // many dereferences as needed.
            } else if matches!(actual_type, Type::MutableReference(_)) {
                let (object, new_type) =
                    self.insert_auto_dereferences(method_call.object, actual_type);
                method_call.object = object;
                argument_types[0].0 = new_type;
            }
        }
    }

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if let Type::MutableReference(element) = typ {
            let location = self.interner.id_location(object);

            let object = self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: UnaryOp::Dereference { implicitly_added: true },
                rhs: object,
            }));
            self.interner.push_expr_type(&object, element.as_ref().clone());
            self.interner.push_expr_location(object, location.span, location.file);

            // Recursively dereference to allow for converting &mut &mut T to T
            self.insert_auto_dereferences(object, *element)
        } else {
            (object, typ)
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

    fn check_index_expression(
        &mut self,
        id: &ExprId,
        mut index_expr: expr::HirIndexExpression,
    ) -> Type {
        let index_type = self.check_expression(&index_expr.index);
        let span = self.interner.expr_span(&index_expr.index);

        index_type.unify(&Type::polymorphic_integer(self.interner), &mut self.errors, || {
            TypeCheckError::TypeMismatch {
                expected_typ: "an integer".to_owned(),
                expr_typ: index_type.to_string(),
                expr_span: span,
            }
        });

        // When writing `a[i]`, if `a : &mut ...` then automatically dereference `a` as many
        // times as needed to get the underlying array.
        let lhs_type = self.check_expression(&index_expr.collection);
        let (new_lhs, lhs_type) = self.insert_auto_dereferences(index_expr.collection, lhs_type);
        index_expr.collection = new_lhs;
        self.interner.replace_expr(id, HirExpression::Index(index_expr));

        match lhs_type.follow_bindings() {
            // XXX: We can check the array bounds here also, but it may be better to constant fold first
            // and have ConstId instead of ExprId for constants
            Type::Array(_, base_type) => *base_type,
            Type::Error => Type::Error,
            typ => {
                let span = self.interner.expr_span(&new_lhs);
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
        match from.follow_bindings() {
            Type::Integer(..)
            | Type::FieldElement
            | Type::TypeVariable(_, TypeVariableKind::IntegerOrField)
            | Type::Bool => (),

            Type::TypeVariable(_, _) => {
                self.errors.push(TypeCheckError::TypeAnnotationsNeeded { span });
                return Type::Error;
            }
            Type::Error => return Type::Error,
            from => {
                self.errors.push(TypeCheckError::InvalidCast { from, span });
                return Type::Error;
            }
        }

        match to {
            Type::Integer(sign, bits) => Type::Integer(sign, bits),
            Type::FieldElement => Type::FieldElement,
            Type::Bool => Type::Bool,
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
        method_ref: HirMethodReference,
        arguments: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        let (fn_typ, param_len) = match method_ref {
            HirMethodReference::FuncId(func_id) => {
                if func_id == FuncId::dummy_id() {
                    return Type::Error;
                }

                let func_meta = self.interner.function_meta(&func_id);
                let param_len = func_meta.parameters.len();
                (func_meta.typ, param_len)
            }
            HirMethodReference::TraitMethodId(method) => {
                let the_trait = self.interner.get_trait(method.trait_id);
                let method = &the_trait.methods[method.method_index];
                (method.typ.clone(), method.arguments().len())
            }
        };

        let arg_len = arguments.len();

        if param_len != arg_len {
            self.errors.push(TypeCheckError::ArityMisMatch {
                expected: param_len as u16,
                found: arg_len as u16,
                span,
            });
        }

        let (function_type, instantiation_bindings) = fn_typ.instantiate(self.interner);

        self.interner.store_instantiation_bindings(*function_ident_id, instantiation_bindings);
        self.interner.push_expr_type(function_ident_id, function_type.clone());
        self.bind_function_type(function_type, arguments, span)
    }

    fn check_if_expr(&mut self, if_expr: &expr::HirIfExpression, expr_id: &ExprId) -> Type {
        let cond_type = self.check_expression(&if_expr.condition);
        let then_type = self.check_expression(&if_expr.consequence);

        let expr_span = self.interner.expr_span(&if_expr.condition);

        self.unify(&cond_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool.to_string(),
            expr_typ: cond_type.to_string(),
            expr_span,
        });

        match if_expr.alternative {
            None => Type::Unit,
            Some(alternative) => {
                let else_type = self.check_expression(&alternative);

                let expr_span = self.interner.expr_span(expr_id);
                self.unify(&then_type, &else_type, || {
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
                self.unify_with_coercions(&arg_type, &param_type, arg, || {
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

            let old_location = this.interner.id_location(old_lhs);
            this.interner.push_expr_location(*access_lhs, span, old_location.file);
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
            (Error, _) | (_, Error) => Ok(Bool),

            // Matches on TypeVariable must be first to follow any type
            // bindings.
            (TypeVariable(int, _), other) | (other, TypeVariable(int, _)) => {
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

                let mut bindings = TypeBindings::new();
                if other.try_bind_to_polymorphic_int(int, &mut bindings).is_ok()
                    || other == &Type::Error
                {
                    Type::apply_type_bindings(bindings);
                    Ok(Bool)
                } else {
                    Err(TypeCheckError::TypeMismatchWithSource {
                        expected: lhs_type.clone(),
                        actual: rhs_type.clone(),
                        span,
                        source: Source::Binary,
                    })
                }
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
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
                Ok(Bool)
            }
            (Integer(..), FieldElement) | (FieldElement, Integer(..)) => {
                Err(TypeCheckError::IntegerAndFieldBinaryOperation { span })
            }
            (Integer(..), typ) | (typ, Integer(..)) => {
                Err(TypeCheckError::IntegerTypeMismatch { typ: typ.clone(), span })
            }
            (FieldElement, FieldElement) => {
                if op.kind.is_valid_for_field_type() {
                    Ok(Bool)
                } else {
                    Err(TypeCheckError::FieldComparison { span })
                }
            }

            // <= and friends are technically valid for booleans, just not very useful
            (Bool, Bool) => Ok(Bool),

            // Special-case == and != for arrays
            (Array(x_size, x_type), Array(y_size, y_type))
                if matches!(op.kind, Equal | NotEqual) =>
            {
                self.unify(x_type, y_type, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs_type.clone(),
                    actual: rhs_type.clone(),
                    source: Source::ArrayElements,
                    span: op.location.span,
                });

                self.unify(x_size, y_size, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs_type.clone(),
                    actual: rhs_type.clone(),
                    source: Source::ArrayLen,
                    span: op.location.span,
                });

                Ok(Bool)
            }
            (lhs @ NamedGeneric(binding_a, _), rhs @ NamedGeneric(binding_b, _)) => {
                if binding_a == binding_b {
                    return Ok(Bool);
                }
                Err(TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    source: Source::Comparison,
                    span,
                })
            }
            (String(x_size), String(y_size)) => {
                self.unify(x_size, y_size, || TypeCheckError::TypeMismatchWithSource {
                    expected: *x_size.clone(),
                    actual: *y_size.clone(),
                    span: op.location.span,
                    source: Source::StringLen,
                });

                Ok(Bool)
            }
            (lhs, rhs) => Err(TypeCheckError::TypeMismatchWithSource {
                expected: lhs.clone(),
                actual: rhs.clone(),
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
    ) -> Option<HirMethodReference> {
        match object_type.follow_bindings() {
            Type::Struct(typ, _args) => {
                let id = typ.borrow().id;
                match self.interner.lookup_method(object_type, id, method_name, false) {
                    Some(method_id) => Some(HirMethodReference::FuncId(method_id)),
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
            // TODO: We should allow method calls on `impl Trait`s eventually.
            //       For now it is fine since they are only allowed on return types.
            Type::TraitAsType(..) => {
                self.errors.push(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span: self.interner.expr_span(expr_id),
                });
                None
            }
            Type::NamedGeneric(_, _) => {
                let func_meta = self.interner.function_meta(
                    &self.current_function.expect("unexpected method outside a function"),
                );

                for constraint in func_meta.trait_constraints {
                    if *object_type == constraint.typ {
                        if let Some(the_trait) = self.interner.try_get_trait(constraint.trait_id) {
                            for (method_index, method) in the_trait.methods.iter().enumerate() {
                                if method.name.0.contents == method_name {
                                    let trait_method = TraitMethodId {
                                        trait_id: constraint.trait_id,
                                        method_index,
                                    };
                                    return Some(HirMethodReference::TraitMethodId(trait_method));
                                }
                            }
                        }
                    }
                }

                self.errors.push(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span: self.interner.expr_span(expr_id),
                });
                None
            }
            // Mutable references to another type should resolve to methods of their element type.
            // This may be a struct or a primitive type.
            Type::MutableReference(element) => self
                .interner
                .lookup_primitive_trait_method_mut(element.as_ref(), method_name)
                .map(HirMethodReference::FuncId)
                .or_else(|| self.lookup_method(&element, method_name, expr_id)),

            // If we fail to resolve the object to a struct type, we have no way of type
            // checking its arguments as we can't even resolve the name of the function
            Type::Error => None,

            // The type variable must be unbound at this point since follow_bindings was called
            Type::TypeVariable(_, TypeVariableKind::Normal) => {
                let span = self.interner.expr_span(expr_id);
                self.errors.push(TypeCheckError::TypeAnnotationsNeeded { span });
                None
            }

            other => match self.interner.lookup_primitive_method(&other, method_name) {
                Some(method_id) => Some(HirMethodReference::FuncId(method_id)),
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

    fn bind_function_type_impl(
        &mut self,
        fn_params: &Vec<Type>,
        fn_ret: &Type,
        callsite_args: &Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        if fn_params.len() != callsite_args.len() {
            self.errors.push(TypeCheckError::ParameterCountMismatch {
                expected: fn_params.len(),
                found: callsite_args.len(),
                span,
            });
            return Type::Error;
        }

        for (param, (arg, _, arg_span)) in fn_params.iter().zip(callsite_args) {
            self.unify(arg, param, || TypeCheckError::TypeMismatch {
                expected_typ: param.to_string(),
                expr_typ: arg.to_string(),
                expr_span: *arg_span,
            });
        }

        fn_ret.clone()
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
                let env_type = self.interner.next_type_variable();
                let expected = Type::Function(args, Box::new(ret.clone()), Box::new(env_type));

                if let Err(error) = binding.try_bind(expected, span) {
                    self.errors.push(error);
                }
                ret
            }
            Type::Function(parameters, ret, _env) => {
                // ignoring env for subtype on purpose
                self.bind_function_type_impl(parameters.as_ref(), ret.as_ref(), args.as_ref(), span)
            }
            Type::Error => Type::Error,
            found => {
                self.errors.push(TypeCheckError::ExpectedFunction { found, span });
                Type::Error
            }
        }
    }

    // Given a binary operator and another type. This method will produce the output type
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
            (TypeVariable(int, _), other) | (other, TypeVariable(int, _)) => {
                if let TypeBinding::Bound(binding) = &*int.borrow() {
                    return self.infix_operand_type_rules(binding, op, other, span);
                }
                if (op.is_modulo() || op.is_bitwise()) && (other.is_bindable() || other.is_field())
                {
                    let other = other.follow_bindings();
                    let kind = op.kind;
                    // This will be an error if these types later resolve to a Field, or stay
                    // polymorphic as the bit size will be unknown. Delay this error until the function
                    // finishes resolving so we can still allow cases like `let x: u8 = 1 << 2;`.
                    self.push_delayed_type_check(Box::new(move || {
                        if other.is_field() {
                            if kind == BinaryOpKind::Modulo {
                                Err(TypeCheckError::FieldModulo { span })
                            } else {
                                Err(TypeCheckError::InvalidBitwiseOperationOnField { span })
                            }
                        } else if other.is_bindable() {
                            Err(TypeCheckError::AmbiguousBitWidth { span })
                        } else if kind.is_bit_shift() && other.is_signed() {
                            Err(TypeCheckError::TypeCannotBeUsed {
                                typ: other,
                                place: "bit shift",
                                span,
                            })
                        } else {
                            Ok(())
                        }
                    }));
                }

                let mut bindings = TypeBindings::new();
                if other.try_bind_to_polymorphic_int(int, &mut bindings).is_ok()
                    || other == &Type::Error
                {
                    Type::apply_type_bindings(bindings);
                    Ok(other.clone())
                } else {
                    Err(TypeCheckError::TypeMismatchWithSource {
                        expected: lhs_type.clone(),
                        actual: rhs_type.clone(),
                        source: Source::Binary,
                        span,
                    })
                }
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
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
                Ok(Integer(*sign_x, *bit_width_x))
            }
            (Integer(..), FieldElement) | (FieldElement, Integer(..)) => {
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

            // The result of two Fields is always a witness
            (FieldElement, FieldElement) => {
                if op.is_bitwise() {
                    return Err(TypeCheckError::InvalidBitwiseOperationOnField { span });
                }
                if op.is_modulo() {
                    return Err(TypeCheckError::FieldModulo { span });
                }
                Ok(FieldElement)
            }

            (Bool, Bool) => Ok(Bool),

            (lhs, rhs) => Err(TypeCheckError::TypeMismatchWithSource {
                expected: lhs.clone(),
                actual: rhs.clone(),
                source: Source::BinOp(op.kind),
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
            rhs_type.unify(&expected, &mut self.errors, || TypeCheckError::TypeMismatch {
                expr_typ: rhs_type.to_string(),
                expected_typ: expected.to_string(),
                expr_span: span,
            });
            expected
        };

        match op {
            crate::UnaryOp::Minus => {
                if rhs_type.is_unsigned() {
                    self.errors
                        .push(TypeCheckError::InvalidUnaryOp { kind: rhs_type.to_string(), span });
                }
                let expected = Type::polymorphic_integer(self.interner);
                rhs_type.unify(&expected, &mut self.errors, || TypeCheckError::InvalidUnaryOp {
                    kind: rhs_type.to_string(),
                    span,
                });
                expected
            }
            crate::UnaryOp::Not => {
                let rhs_type = rhs_type.follow_bindings();

                // `!` can work on booleans or integers
                if matches!(rhs_type, Type::Integer(..)) {
                    return rhs_type;
                }

                unify(Type::Bool)
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
