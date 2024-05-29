use iter_extended::vecmap;
use noirc_errors::Span;

use crate::ast::{BinaryOpKind, IntegerBitSize, UnaryOp};
use crate::hir_def::expr::HirCallExpression;
use crate::macros_api::Signedness;
use crate::{
    hir::{resolution::resolver::verify_mutable_reference, type_check::errors::Source},
    hir_def::{
        expr::{
            self, HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirExpression, HirIdent,
            HirLiteral, HirMethodCallExpression, HirMethodReference, HirPrefixExpression, ImplKind,
        },
        types::Type,
    },
    node_interner::{DefinitionKind, ExprId, FuncId, TraitId, TraitImplKind, TraitMethodId},
    TypeBinding, TypeBindings, TypeVariableKind,
};

use super::{errors::TypeCheckError, TypeChecker};

impl<'interner> TypeChecker<'interner> {
    fn check_if_deprecated(&mut self, expr: &ExprId) {
        if let HirExpression::Ident(expr::HirIdent { location, id, impl_kind: _ }, _) =
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

    fn is_unconstrained_call(&self, expr: &ExprId) -> bool {
        if let HirExpression::Ident(expr::HirIdent { id, .. }, _) = self.interner.expression(expr) {
            if let Some(DefinitionKind::Function(func_id)) =
                self.interner.try_definition(id).map(|def| &def.kind)
            {
                let modifiers = self.interner.function_modifiers(func_id);
                return modifiers.is_unconstrained;
            }
        }
        false
    }

    fn check_hir_array_literal(
        &mut self,
        hir_array_literal: HirArrayLiteral,
    ) -> (Result<u64, Box<Type>>, Box<Type>) {
        match hir_array_literal {
            HirArrayLiteral::Standard(arr) => {
                let elem_types = vecmap(&arr, |arg| self.check_expression(arg));

                let first_elem_type = elem_types
                    .first()
                    .cloned()
                    .unwrap_or_else(|| self.interner.next_type_variable());

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

                (Ok(arr.len() as u64), Box::new(first_elem_type.clone()))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let elem_type = self.check_expression(&repeated_element);
                let length = match length {
                    Type::Constant(length) => Ok(length),
                    other => Err(Box::new(other)),
                };
                (length, Box::new(elem_type))
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
            HirExpression::Ident(ident, generics) => self.check_ident(ident, expr_id, generics),
            HirExpression::Literal(literal) => match literal {
                HirLiteral::Array(hir_array_literal) => {
                    let (length, elem_type) = self.check_hir_array_literal(hir_array_literal);
                    Type::Array(
                        length.map_or_else(
                            |typ| typ,
                            |constant| Box::new(Type::constant_variable(constant, self.interner)),
                        ),
                        elem_type,
                    )
                }
                HirLiteral::Slice(hir_array_literal) => {
                    let (length_type, elem_type) = self.check_hir_array_literal(hir_array_literal);
                    match length_type {
                        Ok(_length) => Type::Slice(elem_type),
                        Err(_non_constant) => {
                            self.errors.push(TypeCheckError::NonConstantSliceLength {
                                span: self.interner.expr_span(expr_id),
                            });
                            Type::Error
                        }
                    }
                }
                HirLiteral::Bool(_) => Type::Bool,
                HirLiteral::Integer(_, _) => self.polymorphic_integer_or_field(),
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
            },
            HirExpression::Infix(infix_expr) => {
                // The type of the infix expression must be looked up from a type table
                let lhs_type = self.check_expression(&infix_expr.lhs);
                let rhs_type = self.check_expression(&infix_expr.rhs);

                let lhs_span = self.interner.expr_span(&infix_expr.lhs);
                let rhs_span = self.interner.expr_span(&infix_expr.rhs);
                let span = lhs_span.merge(rhs_span);

                let operator = &infix_expr.operator;
                match self.infix_operand_type_rules(&lhs_type, operator, &rhs_type, span) {
                    Ok((typ, use_impl)) => {
                        if use_impl {
                            let id = infix_expr.trait_method_id;

                            // Delay checking the trait constraint until the end of the function.
                            // Checking it now could bind an unbound type variable to any type
                            // that implements the trait.
                            let constraint = crate::hir_def::traits::TraitConstraint {
                                typ: lhs_type.clone(),
                                trait_id: id.trait_id,
                                trait_generics: Vec::new(),
                            };
                            self.trait_constraints.push((constraint, *expr_id));
                            self.typecheck_operator_method(*expr_id, id, &lhs_type, span);
                        }
                        typ
                    }
                    Err(error) => {
                        self.errors.push(error);
                        Type::Error
                    }
                }
            }
            HirExpression::Index(index_expr) => self.check_index_expression(expr_id, index_expr),
            HirExpression::Call(call_expr) => {
                let function = self.check_expression(&call_expr.func);

                let args = vecmap(&call_expr.arguments, |arg| {
                    let typ = self.check_expression(arg);
                    (typ, *arg, self.interner.expr_span(arg))
                });

                let span = self.interner.expr_span(expr_id);
                self.check_call(&call_expr, function, args, span)
            }
            HirExpression::MethodCall(mut method_call) => {
                let method_call_span = self.interner.expr_span(expr_id);
                let object = method_call.object;
                let object_span = self.interner.expr_span(&method_call.object);
                let mut object_type = self.check_expression(&method_call.object).follow_bindings();
                let method_name = method_call.method.0.contents.as_str();
                match self.lookup_method(&object_type, method_name, expr_id) {
                    Some(method_ref) => {
                        // Desugar the method call into a normal, resolved function call
                        // so that the backend doesn't need to worry about methods
                        let location = method_call.location;

                        // Automatically add `&mut` if the method expects a mutable reference and
                        // the object is not already one.
                        let func_id = match &method_ref {
                            HirMethodReference::FuncId(func_id) => *func_id,
                            HirMethodReference::TraitMethodId(method_id, _) => {
                                let id = self.interner.trait_method_id(*method_id);
                                let definition = self.interner.definition(id);
                                let DefinitionKind::Function(func_id) = definition.kind else {
                                    unreachable!(
                                        "Expected trait function to be a DefinitionKind::Function"
                                    )
                                };
                                func_id
                            }
                        };

                        if func_id != FuncId::dummy_id() {
                            let function_type = self.interner.function_meta(&func_id).typ.clone();
                            self.try_add_mutable_reference_to_object(
                                &mut method_call,
                                &function_type,
                                &mut object_type,
                            );
                        }

                        // These arguments will be given to the desugared function call.
                        // Compared to the method arguments, they also contain the object.
                        let mut function_args = Vec::with_capacity(method_call.arguments.len() + 1);

                        function_args.push((object_type.clone(), object, object_span));

                        for arg in method_call.arguments.iter() {
                            let span = self.interner.expr_span(arg);
                            let typ = self.check_expression(arg);
                            function_args.push((typ, *arg, span));
                        }

                        // TODO: update object_type here?
                        let ((function_id, _), function_call) = method_call.into_function_call(
                            &method_ref,
                            object_type,
                            location,
                            self.interner,
                        );

                        let func_type = self.check_expression(&function_id);

                        // Type check the new call now that it has been changed from a method call
                        // to a function call. This way we avoid duplicating code.
                        // We call `check_call` rather than `check_expression` directly as we want to avoid
                        // resolving the object type again once it is part of the arguments.
                        let typ = self.check_call(
                            &function_call,
                            func_type,
                            function_args,
                            method_call_span,
                        );

                        self.interner.replace_expr(expr_id, HirExpression::Call(function_call));

                        typ
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
            HirExpression::Block(block_expr) => self.check_block(block_expr),
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
                let captured_vars = vecmap(lambda.captures, |capture| {
                    self.interner.definition_type(capture.ident.id)
                });

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
            HirExpression::Quote(_) => Type::Code,
            HirExpression::Comptime(block) => self.check_block(block),

            // Unquote should be inserted & removed by the comptime interpreter.
            // Even if we allowed it here, we wouldn't know what type to give to the result.
            HirExpression::Unquote(block) => {
                unreachable!("Unquote remaining during type checking {block}")
            }
        };

        self.interner.push_expr_type(*expr_id, typ.clone());
        typ
    }

    fn check_call(
        &mut self,
        call: &HirCallExpression,
        func_type: Type,
        args: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        // Need to setup these flags here as `self` is borrowed mutably to type check the rest of the call expression
        // These flags are later used to type check calls to unconstrained functions from constrained functions
        let func_mod = self.current_function.map(|func| self.interner.function_modifiers(&func));
        let is_current_func_constrained =
            func_mod.map_or(true, |func_mod| !func_mod.is_unconstrained);

        let is_unconstrained_call = self.is_unconstrained_call(&call.func);
        self.check_if_deprecated(&call.func);

        // Check that we are not passing a mutable reference from a constrained runtime to an unconstrained runtime
        if is_current_func_constrained && is_unconstrained_call {
            for (typ, _, _) in args.iter() {
                if matches!(&typ.follow_bindings(), Type::MutableReference(_)) {
                    self.errors.push(TypeCheckError::ConstrainedReferenceToUnconstrained { span });
                }
            }
        }

        let return_type = self.bind_function_type(func_type, args, span);

        // Check that we are not passing a slice from an unconstrained runtime to a constrained runtime
        if is_current_func_constrained && is_unconstrained_call {
            if return_type.contains_slice() {
                self.errors.push(TypeCheckError::UnconstrainedSliceReturnToConstrained { span });
            } else if matches!(&return_type.follow_bindings(), Type::MutableReference(_)) {
                self.errors.push(TypeCheckError::UnconstrainedReferenceToConstrained { span });
            }
        };

        return_type
    }

    fn check_block(&mut self, block: HirBlockExpression) -> Type {
        let mut block_type = Type::Unit;

        let statements = block.statements();
        for (i, stmt) in statements.iter().enumerate() {
            let expr_type = self.check_statement(stmt);

            if let crate::hir_def::stmt::HirStatement::Semi(expr) = self.interner.statement(stmt) {
                let inner_expr_type = self.interner.id_type(expr);
                let span = self.interner.expr_span(&expr);

                self.unify(&inner_expr_type, &Type::Unit, || TypeCheckError::UnusedResultError {
                    expr_type: inner_expr_type.clone(),
                    expr_span: span,
                });
            }

            if i + 1 == statements.len() {
                block_type = expr_type;
            }
        }

        block_type
    }

    /// Returns the type of the given identifier
    fn check_ident(
        &mut self,
        ident: HirIdent,
        expr_id: &ExprId,
        generics: Option<Vec<Type>>,
    ) -> Type {
        let mut bindings = TypeBindings::new();

        // Add type bindings from any constraints that were used.
        // We need to do this first since otherwise instantiating the type below
        // will replace each trait generic with a fresh type variable, rather than
        // the type used in the trait constraint (if it exists). See #4088.
        if let ImplKind::TraitMethod(_, constraint, assumed) = &ident.impl_kind {
            let the_trait = self.interner.get_trait(constraint.trait_id);
            assert_eq!(the_trait.generics.len(), constraint.trait_generics.len());

            for (param, arg) in the_trait.generics.iter().zip(&constraint.trait_generics) {
                // Avoid binding t = t
                if !arg.occurs(param.id()) {
                    bindings.insert(param.id(), (param.clone(), arg.clone()));
                }
            }

            // If the trait impl is already assumed to exist we should add any type bindings for `Self`.
            // Otherwise `self` will be replaced with a fresh type variable, which will require the user
            // to specify a redundant type annotation.
            if *assumed {
                bindings.insert(
                    the_trait.self_type_typevar_id,
                    (the_trait.self_type_typevar.clone(), constraint.typ.clone()),
                );
            }
        }

        // An identifiers type may be forall-quantified in the case of generic functions.
        // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
        // We must instantiate identifiers at every call site to replace this T with a new type
        // variable to handle generic functions.
        let t = self.interner.id_type_substitute_trait_as_type(ident.id);

        let definition = self.interner.try_definition(ident.id);
        let function_generic_count = definition.map_or(0, |definition| match &definition.kind {
            DefinitionKind::Function(function) => {
                self.interner.function_modifiers(function).generic_count
            }
            _ => 0,
        });

        let span = self.interner.expr_span(expr_id);
        // This instantiates a trait's generics as well which need to be set
        // when the constraint below is later solved for when the function is
        // finished. How to link the two?
        let (typ, bindings) = self.instantiate(t, bindings, generics, function_generic_count, span);

        // Push any trait constraints required by this definition to the context
        // to be checked later when the type of this variable is further constrained.
        if let Some(definition) = self.interner.try_definition(ident.id) {
            if let DefinitionKind::Function(func_id) = definition.kind {
                let function = self.interner.function_meta(&func_id);
                for mut constraint in function.trait_constraints.clone() {
                    constraint.apply_bindings(&bindings);
                    self.trait_constraints.push((constraint, *expr_id));
                }
            }
        }

        if let ImplKind::TraitMethod(_, mut constraint, assumed) = ident.impl_kind {
            constraint.apply_bindings(&bindings);
            if assumed {
                let trait_impl = TraitImplKind::Assumed {
                    object_type: constraint.typ,
                    trait_generics: constraint.trait_generics,
                };
                self.interner.select_impl_for_expression(*expr_id, trait_impl);
            } else {
                // Currently only one impl can be selected per expr_id, so this
                // constraint needs to be pushed after any other constraints so
                // that monomorphization can resolve this trait method to the correct impl.
                self.trait_constraints.push((constraint, *expr_id));
            }
        }

        self.interner.store_instantiation_bindings(*expr_id, bindings);
        typ
    }

    fn instantiate(
        &mut self,
        typ: Type,
        bindings: TypeBindings,
        turbofish_generics: Option<Vec<Type>>,
        function_generic_count: usize,
        span: Span,
    ) -> (Type, TypeBindings) {
        match turbofish_generics {
            Some(turbofish_generics) => {
                if turbofish_generics.len() != function_generic_count {
                    self.errors.push(TypeCheckError::IncorrectTurbofishGenericCount {
                        expected_count: function_generic_count,
                        actual_count: turbofish_generics.len(),
                        span,
                    });
                    typ.instantiate_with_bindings(bindings, self.interner)
                } else {
                    // Fetch the count of any implicit generics on the function, such as
                    // for a method within a generic impl.
                    let implicit_generic_count = match &typ {
                        Type::Forall(generics, _) => generics.len() - function_generic_count,
                        _ => 0,
                    };
                    typ.instantiate_with(turbofish_generics, self.interner, implicit_generic_count)
                }
            }
            None => typ.instantiate_with_bindings(bindings, self.interner),
        }
    }

    pub fn verify_trait_constraint(
        &mut self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        function_ident_id: ExprId,
        span: Span,
    ) {
        match self.interner.lookup_trait_implementation(object_type, trait_id, trait_generics) {
            Ok(impl_kind) => {
                self.interner.select_impl_for_expression(function_ident_id, impl_kind);
            }
            Err(erroring_constraints) => {
                if erroring_constraints.is_empty() {
                    self.errors.push(TypeCheckError::TypeAnnotationsNeeded { span });
                } else {
                    // Don't show any errors where try_get_trait returns None.
                    // This can happen if a trait is used that was never declared.
                    let constraints = erroring_constraints
                        .into_iter()
                        .map(|constraint| {
                            let r#trait = self.interner.try_get_trait(constraint.trait_id)?;
                            let mut name = r#trait.name.to_string();
                            if !constraint.trait_generics.is_empty() {
                                let generics =
                                    vecmap(&constraint.trait_generics, ToString::to_string);
                                name += &format!("<{}>", generics.join(", "));
                            }
                            Some((constraint.typ, name))
                        })
                        .collect::<Option<Vec<_>>>();

                    if let Some(constraints) = constraints {
                        self.errors.push(TypeCheckError::NoMatchingImplFound { constraints, span });
                    }
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
        object_type: &mut Type,
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
            let actual_type = object_type.follow_bindings();

            if matches!(expected_object_type.follow_bindings(), Type::MutableReference(_)) {
                if !matches!(actual_type, Type::MutableReference(_)) {
                    if let Err(error) = verify_mutable_reference(self.interner, method_call.object)
                    {
                        self.errors.push(TypeCheckError::ResolverError(error));
                    }

                    let new_type = Type::MutableReference(Box::new(actual_type));
                    *object_type = new_type.clone();

                    // First try to remove a dereference operator that may have been implicitly
                    // inserted by a field access expression `foo.bar` on a mutable reference `foo`.
                    let new_object = self.try_remove_implicit_dereference(method_call.object);

                    // If that didn't work, then wrap the whole expression in an `&mut`
                    method_call.object = new_object.unwrap_or_else(|| {
                        let location = self.interner.id_location(method_call.object);

                        let new_object =
                            self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                                operator: UnaryOp::MutableReference,
                                rhs: method_call.object,
                            }));
                        self.interner.push_expr_type(new_object, new_type);
                        self.interner.push_expr_location(new_object, location.span, location.file);
                        new_object
                    });
                }
            // Otherwise if the object type is a mutable reference and the method is not, insert as
            // many dereferences as needed.
            } else if matches!(actual_type, Type::MutableReference(_)) {
                let (object, new_type) =
                    self.insert_auto_dereferences(method_call.object, actual_type);
                *object_type = new_type;
                method_call.object = object;
            }
        }
    }

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    pub(crate) fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if let Type::MutableReference(element) = typ {
            let location = self.interner.id_location(object);

            let object = self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: UnaryOp::Dereference { implicitly_added: true },
                rhs: object,
            }));
            self.interner.push_expr_type(object, element.as_ref().clone());
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
    /// Returns Some(new_expr_id) if a dereference was removed and None otherwise.
    fn try_remove_implicit_dereference(&mut self, object: ExprId) -> Option<ExprId> {
        match self.interner.expression(&object) {
            HirExpression::MemberAccess(mut access) => {
                let new_lhs = self.try_remove_implicit_dereference(access.lhs)?;
                access.lhs = new_lhs;
                access.is_offset = true;

                // `object` will have a different type now, which will be filled in
                // later when type checking the method call as a function call.
                self.interner.replace_expr(&object, HirExpression::MemberAccess(access));
                Some(object)
            }
            HirExpression::Prefix(prefix) => match prefix.operator {
                // Found a dereference we can remove. Now just replace it with its rhs to remove it.
                UnaryOp::Dereference { implicitly_added: true } => Some(prefix.rhs),
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

        index_type.unify(&self.polymorphic_integer_or_field(), &mut self.errors, || {
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
            Type::Slice(base_type) => *base_type,
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
            | Type::TypeVariable(_, TypeVariableKind::Integer)
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
                operator: crate::ast::UnaryOp::Dereference { implicitly_added: true },
                rhs: old_lhs,
            }));
            this.interner.push_expr_type(old_lhs, lhs_type);
            this.interner.push_expr_type(*access_lhs, element);

            let old_location = this.interner.id_location(old_lhs);
            this.interner.push_expr_location(*access_lhs, span, old_location.file);
        };

        // If this access is just a field offset, we want to avoid dereferencing
        let dereference_lhs = (!access.is_offset).then_some(dereference_lhs);

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
    ///
    /// If `dereference_lhs` is None, this will assume we're taking the offset of a struct field
    /// rather than dereferencing it. So the result of `foo.bar` with a `foo : &mut Foo` will
    /// be a `&mut Bar` rather than just a `Bar`.
    pub(super) fn check_field_access(
        &mut self,
        lhs_type: &Type,
        field_name: &str,
        span: Span,
        dereference_lhs: Option<impl FnMut(&mut Self, Type, Type)>,
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
                if let Some(mut dereference_lhs) = dereference_lhs {
                    dereference_lhs(self, lhs_type.clone(), element.as_ref().clone());
                    return self.check_field_access(
                        element,
                        field_name,
                        span,
                        Some(dereference_lhs),
                    );
                } else {
                    let (element, index) =
                        self.check_field_access(element, field_name, span, dereference_lhs)?;
                    return Some((Type::MutableReference(Box::new(element)), index));
                }
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

    // Given a binary comparison operator and another type. This method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    fn comparator_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        rhs_type: &Type,
        op: &HirBinaryOp,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match (lhs_type, rhs_type) {
            // Avoid reporting errors multiple times
            (Error, _) | (_, Error) => Ok((Bool, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.comparator_operand_type_rules(&alias, other, op, span)
            }

            // Matches on TypeVariable must be first to follow any type
            // bindings.
            (TypeVariable(var, _), other) | (other, TypeVariable(var, _)) => {
                if let TypeBinding::Bound(binding) = &*var.borrow() {
                    return self.comparator_operand_type_rules(other, binding, op, span);
                }

                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, span);
                Ok((Bool, use_impl))
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
                Ok((Bool, false))
            }
            (FieldElement, FieldElement) => {
                if op.kind.is_valid_for_field_type() {
                    Ok((Bool, false))
                } else {
                    Err(TypeCheckError::FieldComparison { span })
                }
            }

            // <= and friends are technically valid for booleans, just not very useful
            (Bool, Bool) => Ok((Bool, false)),

            (lhs, rhs) => {
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    span: op.location.span,
                    source: Source::Binary,
                });
                Ok((Bool, true))
            }
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

                for constraint in &func_meta.trait_constraints {
                    if *object_type == constraint.typ {
                        if let Some(the_trait) = self.interner.try_get_trait(constraint.trait_id) {
                            for (method_index, method) in the_trait.methods.iter().enumerate() {
                                if method.name.0.contents == method_name {
                                    let trait_method = TraitMethodId {
                                        trait_id: constraint.trait_id,
                                        method_index,
                                    };
                                    return Some(HirMethodReference::TraitMethodId(
                                        trait_method,
                                        constraint.trait_generics.clone(),
                                    ));
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
        fn_params: &[Type],
        fn_ret: &Type,
        callsite_args: &[(Type, ExprId, Span)],
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
            // ignoring env for subtype on purpose
            Type::Function(parameters, ret, _env) => {
                self.bind_function_type_impl(&parameters, &ret, &args, span)
            }
            Type::Error => Type::Error,
            found => {
                self.errors.push(TypeCheckError::ExpectedFunction { found, span });
                Type::Error
            }
        }
    }

    /// Handles the TypeVariable case for checking binary operators.
    /// Returns true if we should use the impl for the operator instead of the primitive
    /// version of it.
    fn bind_type_variables_for_infix(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> bool {
        self.unify(lhs_type, rhs_type, || TypeCheckError::TypeMismatchWithSource {
            expected: lhs_type.clone(),
            actual: rhs_type.clone(),
            source: Source::Binary,
            span,
        });

        let use_impl = !lhs_type.is_numeric();

        // If this operator isn't valid for fields we have to possibly narrow
        // TypeVariableKind::IntegerOrField to TypeVariableKind::Integer.
        // Doing so also ensures a type error if Field is used.
        // The is_numeric check is to allow impls for custom types to bypass this.
        if !op.kind.is_valid_for_field_type() && lhs_type.is_numeric() {
            let target = Type::polymorphic_integer(self.interner);

            use crate::ast::BinaryOpKind::*;
            use TypeCheckError::*;
            self.unify(lhs_type, &target, || match op.kind {
                Less | LessEqual | Greater | GreaterEqual => FieldComparison { span },
                And | Or | Xor | ShiftRight | ShiftLeft => FieldBitwiseOp { span },
                Modulo => FieldModulo { span },
                other => unreachable!("Operator {other:?} should be valid for Field"),
            });
        }

        use_impl
    }

    // Given a binary operator and another type. This method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    fn infix_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        if op.kind.is_comparator() {
            return self.comparator_operand_type_rules(lhs_type, rhs_type, op, span);
        }

        use Type::*;
        match (lhs_type, rhs_type) {
            // An error type on either side will always return an error
            (Error, _) | (_, Error) => Ok((Error, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.infix_operand_type_rules(&alias, op, other, span)
            }

            // Matches on TypeVariable must be first so that we follow any type
            // bindings.
            (TypeVariable(int, _), other) | (other, TypeVariable(int, _)) => {
                if let TypeBinding::Bound(binding) = &*int.borrow() {
                    return self.infix_operand_type_rules(binding, op, other, span);
                }
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    self.unify(
                        rhs_type,
                        &Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
                        || TypeCheckError::InvalidShiftSize { span },
                    );
                    let use_impl = if lhs_type.is_numeric() {
                        let integer_type = Type::polymorphic_integer(self.interner);
                        self.bind_type_variables_for_infix(lhs_type, op, &integer_type, span)
                    } else {
                        true
                    };
                    return Ok((lhs_type.clone(), use_impl));
                }
                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, span);
                Ok((other.clone(), use_impl))
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    if *sign_y != Signedness::Unsigned || *bit_width_y != IntegerBitSize::Eight {
                        return Err(TypeCheckError::InvalidShiftSize { span });
                    }
                    return Ok((Integer(*sign_x, *bit_width_x), false));
                }
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
                Ok((Integer(*sign_x, *bit_width_x), false))
            }
            // The result of two Fields is always a witness
            (FieldElement, FieldElement) => {
                if !op.kind.is_valid_for_field_type() {
                    if op.kind == BinaryOpKind::Modulo {
                        return Err(TypeCheckError::FieldModulo { span });
                    } else {
                        return Err(TypeCheckError::FieldBitwiseOp { span });
                    }
                }
                Ok((FieldElement, false))
            }

            (Bool, Bool) => Ok((Bool, false)),

            (lhs, rhs) => {
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    if rhs == &Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight) {
                        return Ok((lhs.clone(), true));
                    }
                    return Err(TypeCheckError::InvalidShiftSize { span });
                }
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    span: op.location.span,
                    source: Source::Binary,
                });
                Ok((lhs.clone(), true))
            }
        }
    }

    fn type_check_prefix_operand(
        &mut self,
        op: &crate::ast::UnaryOp,
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
            crate::ast::UnaryOp::Minus => {
                if rhs_type.is_unsigned() {
                    self.errors
                        .push(TypeCheckError::InvalidUnaryOp { kind: rhs_type.to_string(), span });
                }
                let expected = self.polymorphic_integer_or_field();
                rhs_type.unify(&expected, &mut self.errors, || TypeCheckError::InvalidUnaryOp {
                    kind: rhs_type.to_string(),
                    span,
                });
                expected
            }
            crate::ast::UnaryOp::Not => {
                let rhs_type = rhs_type.follow_bindings();

                // `!` can work on booleans or integers
                if matches!(rhs_type, Type::Integer(..)) {
                    return rhs_type;
                }

                unify(Type::Bool)
            }
            crate::ast::UnaryOp::MutableReference => {
                Type::MutableReference(Box::new(rhs_type.follow_bindings()))
            }
            crate::ast::UnaryOp::Dereference { implicitly_added: _ } => {
                let element_type = self.interner.next_type_variable();
                unify(Type::MutableReference(Box::new(element_type.clone())));
                element_type
            }
        }
    }

    /// Prerequisite: verify_trait_constraint of the operator's trait constraint.
    ///
    /// Although by this point the operator is expected to already have a trait impl,
    /// we still need to match the operator's type against the method's instantiated type
    /// to ensure the instantiation bindings are correct and the monomorphizer can
    /// re-apply the needed bindings.
    fn typecheck_operator_method(
        &mut self,
        expr_id: ExprId,
        trait_method_id: TraitMethodId,
        object_type: &Type,
        span: Span,
    ) {
        let the_trait = self.interner.get_trait(trait_method_id.trait_id);

        let method = &the_trait.methods[trait_method_id.method_index];
        let (method_type, mut bindings) = method.typ.instantiate(self.interner);

        match method_type {
            Type::Function(args, _, _) => {
                // We can cheat a bit and match against only the object type here since no operator
                // overload uses other generic parameters or return types aside from the object type.
                let expected_object_type = &args[0];
                self.unify(object_type, expected_object_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_object_type.to_string(),
                    expr_typ: object_type.to_string(),
                    expr_span: span,
                });
            }
            other => {
                unreachable!("Expected operator method to have a function type, but found {other}")
            }
        }

        // We must also remember to apply these substitutions to the object_type
        // referenced by the selected trait impl, if one has yet to be selected.
        let impl_kind = self.interner.get_selected_impl_for_expression(expr_id);
        if let Some(TraitImplKind::Assumed { object_type, trait_generics }) = impl_kind {
            let the_trait = self.interner.get_trait(trait_method_id.trait_id);
            let object_type = object_type.substitute(&bindings);
            bindings.insert(
                the_trait.self_type_typevar_id,
                (the_trait.self_type_typevar.clone(), object_type.clone()),
            );
            self.interner.select_impl_for_expression(
                expr_id,
                TraitImplKind::Assumed { object_type, trait_generics },
            );
        }

        self.interner.store_instantiation_bindings(expr_id, bindings);
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
