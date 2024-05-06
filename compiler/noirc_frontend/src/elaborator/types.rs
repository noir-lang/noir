use std::rc::Rc;

use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use crate::{macros_api::{UnresolvedType, Path, SecondaryAttribute, PathKind, UnresolvedTypeData, HirExpression, HirLiteral, UnaryOp}, Type, hir::{resolution::{errors::ResolverError, resolver::SELF_TYPE_NAME, import::PathResolution}, def_map::ModuleDefId, type_check::TypeCheckError}, Generics, TypeVariable, ast::{UnresolvedTypeExpression, UnresolvedTraitConstraint, BinaryOpKind}, Shared, TypeAlias, StructType, hir_def::{traits::{Trait, TraitConstraint}, expr::HirPrefixExpression}, node_interner::{TraitMethodId, GlobalId, ExprId, DefinitionKind}, TypeVariableKind};

use super::Elaborator;



impl Elaborator {
    /// Translates an UnresolvedType to a Type
    pub fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        let span = typ.span;
        let resolved_type = self.resolve_type_inner(typ, &mut vec![]);
        if resolved_type.is_nested_slice() {
            self.push_err(ResolverError::NestedSlices { span: span.unwrap() });
        }

        resolved_type
    }

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    fn resolve_type_inner(&mut self, typ: UnresolvedType, new_variables: &mut Generics) -> Type {
        use crate::ast::UnresolvedTypeData::*;

        let resolved_type = match typ.typ {
            FieldElement => Type::FieldElement,
            Array(size, elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, new_variables));
                let size = self.resolve_array_size(Some(size), new_variables);
                Type::Array(Box::new(size), elem)
            }
            Slice(elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, new_variables));
                Type::Slice(elem)
            }
            Expression(expr) => self.convert_expression_type(expr),
            Integer(sign, bits) => Type::Integer(sign, bits),
            Bool => Type::Bool,
            String(size) => {
                let resolved_size = self.resolve_array_size(size, new_variables);
                Type::String(Box::new(resolved_size))
            }
            FormatString(size, fields) => {
                let resolved_size = self.convert_expression_type(size);
                let fields = self.resolve_type_inner(*fields, new_variables);
                Type::FmtString(Box::new(resolved_size), Box::new(fields))
            }
            Code => Type::Code,
            Unit => Type::Unit,
            Unspecified => Type::Error,
            Error => Type::Error,
            Named(path, args, _) => self.resolve_named_type(path, args, new_variables),
            TraitAsType(path, args) => self.resolve_trait_as_type(path, args, new_variables),

            Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field, new_variables)))
            }
            Function(args, ret, env) => {
                let args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));
                let ret = Box::new(self.resolve_type_inner(*ret, new_variables));

                // expect() here is valid, because the only places we don't have a span are omitted types
                // e.g. a function without return type implicitly has a spanless UnresolvedType::Unit return type
                // To get an invalid env type, the user must explicitly specify the type, which will have a span
                let env_span =
                    env.span.expect("Unexpected missing span for closure environment type");

                let env = Box::new(self.resolve_type_inner(*env, new_variables));

                match *env {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_, _) => {
                        Type::Function(args, ret, env)
                    }
                    _ => {
                        self.push_err(ResolverError::InvalidClosureEnvironment {
                            typ: *env,
                            span: env_span,
                        });
                        Type::Error
                    }
                }
            }
            MutableReference(element) => {
                Type::MutableReference(Box::new(self.resolve_type_inner(*element, new_variables)))
            }
            Parenthesized(typ) => self.resolve_type_inner(*typ, new_variables),
        };

        if let Type::Struct(_, _) = resolved_type {
            if let Some(unresolved_span) = typ.span {
                // Record the location of the type reference
                self.interner.push_type_ref_location(
                    resolved_type.clone(),
                    Location::new(unresolved_span, self.file),
                );
            }
        }
        resolved_type
    }

    fn find_generic(&self, target_name: &str) -> Option<&(Rc<String>, TypeVariable, Span)> {
        self.generics.iter().find(|(name, _, _)| name.as_ref() == target_name)
    }

    fn resolve_named_type(
        &mut self,
        path: Path,
        args: Vec<UnresolvedType>,
        new_variables: &mut Generics,
    ) -> Type {
        if args.is_empty() {
            if let Some(typ) = self.lookup_generic_or_global_type(&path) {
                return typ;
            }
        }

        // Check if the path is a type variable first. We currently disallow generics on type
        // variables since we do not support higher-kinded types.
        if path.segments.len() == 1 {
            let name = &path.last_segment().0.contents;

            if name == SELF_TYPE_NAME {
                if let Some(self_type) = self.self_type.clone() {
                    if !args.is_empty() {
                        self.push_err(ResolverError::GenericsOnSelfType { span: path.span() });
                    }
                    return self_type;
                }
            }
        }

        let span = path.span();
        let mut args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));

        if let Some(type_alias) = self.lookup_type_alias(path.clone()) {
            let type_alias = type_alias.borrow();
            let expected_generic_count = type_alias.generics.len();
            let type_alias_string = type_alias.to_string();
            let id = type_alias.id;

            self.verify_generics_count(expected_generic_count, &mut args, span, || {
                type_alias_string
            });

            if let Some(item) = self.current_item {
                self.interner.add_type_alias_dependency(item, id);
            }

            // Collecting Type Alias references [Location]s to be used by LSP in order
            // to resolve the definition of the type alias
            self.interner.add_type_alias_ref(id, Location::new(span, self.file));

            // Because there is no ordering to when type aliases (and other globals) are resolved,
            // it is possible for one to refer to an Error type and issue no error if it is set
            // equal to another type alias. Fixing this fully requires an analysis to create a DFG
            // of definition ordering, but for now we have an explicit check here so that we at
            // least issue an error that the type was not found instead of silently passing.
            let alias = self.interner.get_type_alias(id);
            return Type::Alias(alias, args);
        }

        match self.lookup_struct_or_error(path) {
            Some(struct_type) => {
                if self.resolving_ids.contains(&struct_type.borrow().id) {
                    self.push_err(ResolverError::SelfReferentialStruct {
                        span: struct_type.borrow().name.span(),
                    });

                    return Type::Error;
                }

                let expected_generic_count = struct_type.borrow().generics.len();
                if !self.in_contract
                    && self
                        .interner
                        .struct_attributes(&struct_type.borrow().id)
                        .iter()
                        .any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
                {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        span: struct_type.borrow().name.span(),
                    });
                }
                self.verify_generics_count(expected_generic_count, &mut args, span, || {
                    struct_type.borrow().to_string()
                });

                if let Some(current_item) = self.current_item {
                    let dependency_id = struct_type.borrow().id;
                    self.interner.add_type_dependency(current_item, dependency_id);
                }

                Type::Struct(struct_type, args)
            }
            None => Type::Error,
        }
    }

    fn resolve_trait_as_type(
        &mut self,
        path: Path,
        args: Vec<UnresolvedType>,
        new_variables: &mut Generics,
    ) -> Type {
        let args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));

        if let Some(t) = self.lookup_trait_or_error(path) {
            Type::TraitAsType(t.id, Rc::new(t.name.to_string()), args)
        } else {
            Type::Error
        }
    }

    fn verify_generics_count(
        &mut self,
        expected_count: usize,
        args: &mut Vec<Type>,
        span: Span,
        type_name: impl FnOnce() -> String,
    ) {
        if args.len() != expected_count {
            self.push_err(ResolverError::IncorrectGenericCount {
                span,
                item_name: type_name(),
                actual: args.len(),
                expected: expected_count,
            });

            // Fix the generic count so we can continue typechecking
            args.resize_with(expected_count, || Type::Error);
        }
    }

    fn lookup_generic_or_global_type(&mut self, path: &Path) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = &path.last_segment().0.contents;
            if let Some((name, var, _)) = self.find_generic(name) {
                return Some(Type::NamedGeneric(var.clone(), name.clone()));
            }
        }

        // If we cannot find a local generic of the same name, try to look up a global
        match self.path_resolver.resolve(&self.def_maps, path.clone()) {
            Ok(PathResolution { module_def_id: ModuleDefId::GlobalId(id), error }) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                if let Some(error) = error {
                    self.push_err(error);
                }
                Some(Type::Constant(self.eval_global_as_array_length(id, path)))
            }
            _ => None,
        }
    }

    fn resolve_array_size(
        &mut self,
        length: Option<UnresolvedTypeExpression>,
        new_variables: &mut Generics,
    ) -> Type {
        match length {
            None => {
                let id = self.interner.next_type_variable_id();
                let typevar = TypeVariable::unbound(id);
                new_variables.push(typevar.clone());

                // 'Named'Generic is a bit of a misnomer here, we want a type variable that
                // wont be bound over but this one has no name since we do not currently
                // require users to explicitly be generic over array lengths.
                Type::NamedGeneric(typevar, Rc::new("".into()))
            }
            Some(length) => self.convert_expression_type(length),
        }
    }

    pub fn convert_expression_type(&mut self, length: UnresolvedTypeExpression) -> Type {
        match length {
            UnresolvedTypeExpression::Variable(path) => {
                self.lookup_generic_or_global_type(&path).unwrap_or_else(|| {
                    self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                    Type::Constant(0)
                })
            }
            UnresolvedTypeExpression::Constant(int, _) => Type::Constant(int),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
                let (lhs_span, rhs_span) = (lhs.span(), rhs.span());
                let lhs = self.convert_expression_type(*lhs);
                let rhs = self.convert_expression_type(*rhs);

                match (lhs, rhs) {
                    (Type::Constant(lhs), Type::Constant(rhs)) => {
                        Type::Constant(op.function()(lhs, rhs))
                    }
                    (lhs, _) => {
                        let span =
                            if !matches!(lhs, Type::Constant(_)) { lhs_span } else { rhs_span };
                        self.push_err(ResolverError::InvalidArrayLengthExpr { span });
                        Type::Constant(0)
                    }
                }
            }
        }
    }

    /// Lookup a given struct type by name.
    fn lookup_struct_or_error(&mut self, path: Path) -> Option<Shared<StructType>> {
        match self.lookup(path) {
            Ok(struct_id) => Some(self.get_struct(struct_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Lookup a given trait by name/path.
    fn lookup_trait_or_error(&mut self, path: Path) -> Option<&mut Trait> {
        match self.lookup(path) {
            Ok(trait_id) => Some(self.get_trait_mut(trait_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Looks up a given type by name.
    /// This will also instantiate any struct types found.
    pub fn lookup_type_or_error(&mut self, path: Path) -> Option<Type> {
        let ident = path.as_ident();
        if ident.map_or(false, |i| i == SELF_TYPE_NAME) {
            if let Some(typ) = &self.self_type {
                return Some(typ.clone());
            }
        }

        match self.lookup(path) {
            Ok(struct_id) => {
                let struct_type = self.get_struct(struct_id);
                let generics = struct_type.borrow().instantiate(&mut self.interner);
                Some(Type::Struct(struct_type, generics))
            }
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    fn lookup_type_alias(&mut self, path: Path) -> Option<Shared<TypeAlias>> {
        self.lookup(path).ok().map(|id| self.interner.get_type_alias(id))
    }

    // this resolves Self::some_static_method, inside an impl block (where we don't have a concrete self_type)
    fn resolve_trait_static_method_by_self(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        let trait_id = self.trait_id?;

        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let name = &path.segments[0].0.contents;
            let method = &path.segments[1];

            if name == SELF_TYPE_NAME {
                let the_trait = self.interner.get_trait(trait_id);
                let method = the_trait.find_method(method.0.contents.as_str())?;

                let constraint = TraitConstraint {
                    typ: self.self_type.clone()?,
                    trait_generics: Type::from_generics(&the_trait.generics),
                    trait_id,
                };
                return Some((method, constraint, false));
            }
        }
        None
    }

    // this resolves TraitName::some_static_method
    fn resolve_trait_static_method(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let method = &path.segments[1];

            let mut trait_path = path.clone();
            trait_path.pop();
            let trait_id = self.lookup(trait_path).ok()?;
            let the_trait = self.interner.get_trait(trait_id);

            let method = the_trait.find_method(method.0.contents.as_str())?;
            let constraint = TraitConstraint {
                typ: Type::TypeVariable(
                    the_trait.self_type_typevar.clone(),
                    TypeVariableKind::Normal,
                ),
                trait_generics: Type::from_generics(&the_trait.generics),
                trait_id,
            };
            return Some((method, constraint, false));
        }
        None
    }

    // This resolves a static trait method T::trait_method by iterating over the where clause
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed from a where
    // clause. This is always true since this helper searches where clauses for a generic constraint.
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_method_by_named_generic(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        if path.segments.len() != 2 {
            return None;
        }

        for UnresolvedTraitConstraint { typ, trait_bound } in self.trait_bounds.clone() {
            if let UnresolvedTypeData::Named(constraint_path, _, _) = &typ.typ {
                // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
                if constraint_path.segments.len() == 1
                    && path.segments[0] != constraint_path.last_segment()
                {
                    continue;
                }

                if let Ok(ModuleDefId::TraitId(trait_id)) =
                    self.resolve_path(trait_bound.trait_path.clone())
                {
                    let the_trait = self.interner.get_trait(trait_id);
                    if let Some(method) =
                        the_trait.find_method(path.segments.last().unwrap().0.contents.as_str())
                    {
                        let constraint = TraitConstraint {
                            trait_id,
                            typ: self.resolve_type(typ.clone()),
                            trait_generics: vecmap(trait_bound.trait_generics, |typ| {
                                self.resolve_type(typ)
                            }),
                        };
                        return Some((method, constraint, true));
                    }
                }
            }
        }
        None
    }

    // Try to resolve the given trait method path.
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    pub fn resolve_trait_generic_path(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        self.resolve_trait_static_method_by_self(path)
            .or_else(|| self.resolve_trait_static_method(path))
            .or_else(|| self.resolve_trait_method_by_named_generic(path))
    }

    fn eval_global_as_array_length(&mut self, global: GlobalId, path: &Path) -> u64 {
        let Some(stmt) = self.interner.get_global_let_statement(global) else {
            let path = path.clone();
            self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
            return 0;
        };

        let length = stmt.expression;
        let span = self.interner.expr_span(&length);
        let result = self.try_eval_array_length_id(length, span);

        match result.map(|length| length.try_into()) {
            Ok(Ok(length_value)) => return length_value,
            Ok(Err(_cast_err)) => self.push_err(ResolverError::IntegerTooLarge { span }),
            Err(Some(error)) => self.push_err(error),
            Err(None) => (),
        }
        0
    }

    fn try_eval_array_length_id(
        &self,
        rhs: ExprId,
        span: Span,
    ) -> Result<u128, Option<ResolverError>> {
        // Arbitrary amount of recursive calls to try before giving up
        let fuel = 100;
        self.try_eval_array_length_id_with_fuel(rhs, span, fuel)
    }

    fn try_eval_array_length_id_with_fuel(
        &self,
        rhs: ExprId,
        span: Span,
        fuel: u32,
    ) -> Result<u128, Option<ResolverError>> {
        if fuel == 0 {
            // If we reach here, it is likely from evaluating cyclic globals. We expect an error to
            // be issued for them after name resolution so issue no error now.
            return Err(None);
        }

        match self.interner.expression(&rhs) {
            HirExpression::Literal(HirLiteral::Integer(int, false)) => {
                int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
            }
            HirExpression::Ident(ident) => {
                let definition = self.interner.definition(ident.id);
                match definition.kind {
                    DefinitionKind::Global(global_id) => {
                        let let_statement = self.interner.get_global_let_statement(global_id);
                        if let Some(let_statement) = let_statement {
                            let expression = let_statement.expression;
                            self.try_eval_array_length_id_with_fuel(expression, span, fuel - 1)
                        } else {
                            Err(Some(ResolverError::InvalidArrayLengthExpr { span }))
                        }
                    }
                    _ => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
                }
            }
            HirExpression::Infix(infix) => {
                let lhs = self.try_eval_array_length_id_with_fuel(infix.lhs, span, fuel - 1)?;
                let rhs = self.try_eval_array_length_id_with_fuel(infix.rhs, span, fuel - 1)?;

                match infix.operator.kind {
                    BinaryOpKind::Add => Ok(lhs + rhs),
                    BinaryOpKind::Subtract => Ok(lhs - rhs),
                    BinaryOpKind::Multiply => Ok(lhs * rhs),
                    BinaryOpKind::Divide => Ok(lhs / rhs),
                    BinaryOpKind::Equal => Ok((lhs == rhs) as u128),
                    BinaryOpKind::NotEqual => Ok((lhs != rhs) as u128),
                    BinaryOpKind::Less => Ok((lhs < rhs) as u128),
                    BinaryOpKind::LessEqual => Ok((lhs <= rhs) as u128),
                    BinaryOpKind::Greater => Ok((lhs > rhs) as u128),
                    BinaryOpKind::GreaterEqual => Ok((lhs >= rhs) as u128),
                    BinaryOpKind::And => Ok(lhs & rhs),
                    BinaryOpKind::Or => Ok(lhs | rhs),
                    BinaryOpKind::Xor => Ok(lhs ^ rhs),
                    BinaryOpKind::ShiftRight => Ok(lhs >> rhs),
                    BinaryOpKind::ShiftLeft => Ok(lhs << rhs),
                    BinaryOpKind::Modulo => Ok(lhs % rhs),
                }
            }
            _other => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }

    /// Check if an assignment is overflowing with respect to `annotated_type`
    /// in a declaration statement where `annotated_type` is an unsigned integer
    pub fn lint_overflowing_uint(&mut self, rhs_expr: &ExprId, annotated_type: &Type) {
        let expr = self.interner.expression(rhs_expr);
        let span = self.interner.expr_span(rhs_expr);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(value, false)) => {
                let v = value.to_u128();
                if let Type::Integer(_, bit_count) = annotated_type {
                    let bit_count: u32 = (*bit_count).into();
                    let max = 1 << bit_count;
                    if v >= max {
                        self.push_err(TypeCheckError::OverflowingAssignment {
                            expr: value,
                            ty: annotated_type.clone(),
                            range: format!("0..={}", max - 1),
                            span,
                        });
                    };
                };
            }
            HirExpression::Prefix(expr) => {
                self.lint_overflowing_uint(&expr.rhs, annotated_type);
                if matches!(expr.operator, UnaryOp::Minus) {
                    self.push_err(TypeCheckError::InvalidUnaryOp {
                        kind: "annotated_type".to_string(),
                        span,
                    });
                }
            }
            HirExpression::Infix(expr) => {
                self.lint_overflowing_uint(&expr.lhs, annotated_type);
                self.lint_overflowing_uint(&expr.rhs, annotated_type);
            }
            _ => {}
        }
    }

    pub fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut errors = Vec::new();
        actual.unify(expected, &mut errors, make_error);
        self.errors.extend(errors.into_iter().map(Into::into));
    }

    /// Wrapper of Type::unify_with_coercions using self.errors
    pub fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut errors = Vec::new();
        actual.unify_with_coercions(
            expected,
            expression,
            &mut self.interner,
            &mut errors,
            make_error,
        );
        self.errors.extend(errors.into_iter().map(Into::into));
    }

    /// Return a fresh integer or field type variable and log it
    /// in self.type_variables to default it later.
    pub fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(&mut self.interner);
        self.type_variables.push(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(&mut self.interner);
        self.type_variables.push(typ.clone());
        typ
    }

    /// Translates a (possibly Unspecified) UnresolvedType to a Type.
    /// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
    pub fn resolve_inferred_type(&mut self, typ: UnresolvedType) -> Type {
        match &typ.typ {
            UnresolvedTypeData::Unspecified => self.interner.next_type_variable(),
            _ => self.resolve_type_inner(typ, &mut vec![]),
        }
    }

    pub fn type_check_prefix_operand(
        &mut self,
        op: &crate::ast::UnaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Type {
        let mut unify = |expected| {
            self.unify(rhs_type, &expected, || TypeCheckError::TypeMismatch {
                expr_typ: rhs_type.to_string(),
                expected_typ: expected.to_string(),
                expr_span: span,
            });
            expected
        };

        match op {
            crate::ast::UnaryOp::Minus => {
                if rhs_type.is_unsigned() {
                    self.push_err(TypeCheckError::InvalidUnaryOp { kind: rhs_type.to_string(), span });
                }
                let expected = self.polymorphic_integer_or_field();
                self.unify(rhs_type, &expected, || TypeCheckError::InvalidUnaryOp {
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

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    pub fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
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
}
