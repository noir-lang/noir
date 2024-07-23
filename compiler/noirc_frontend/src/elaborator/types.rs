use std::{collections::BTreeMap, rc::Rc};

use acvm::acir::AcirField;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use crate::{
    ast::{
        BinaryOpKind, IntegerBitSize, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedTypeExpression,
    },
    hir::{
        comptime::{Interpreter, Value},
        def_map::ModuleDefId,
        resolution::errors::ResolverError,
        type_check::{NoMatchingImplFoundError, Source, TypeCheckError},
    },
    hir_def::{
        expr::{
            HirBinaryOp, HirCallExpression, HirMemberAccess, HirMethodReference,
            HirPrefixExpression,
        },
        function::{FuncMeta, Parameters},
        traits::TraitConstraint,
    },
    macros_api::{
        HirExpression, HirLiteral, HirStatement, NodeInterner, Path, PathKind, SecondaryAttribute,
        Signedness, UnaryOp, UnresolvedType, UnresolvedTypeData,
    },
    node_interner::{
        DefinitionKind, DependencyId, ExprId, FuncId, GlobalId, TraitId, TraitImplKind,
        TraitMethodId,
    },
    Generics, Kind, ResolvedGeneric, Type, TypeBinding, TypeVariable, TypeVariableKind,
};

use super::{lints, Elaborator};

pub const SELF_TYPE_NAME: &str = "Self";
pub const WILDCARD_TYPE: &str = "_";

impl<'context> Elaborator<'context> {
    /// Translates an UnresolvedType to a Type with a `TypeKind::Normal`
    pub(super) fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        let span = typ.span;
        let resolved_type = self.resolve_type_inner(typ, &Kind::Normal);
        if resolved_type.is_nested_slice() {
            self.push_err(ResolverError::NestedSlices {
                span: span.expect("Type should have span"),
            });
        }
        resolved_type
    }

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    pub fn resolve_type_inner(&mut self, typ: UnresolvedType, kind: &Kind) -> Type {
        use crate::ast::UnresolvedTypeData::*;

        let span = typ.span;
        let (named_path_span, is_self_type_name, is_synthetic) =
            if let Named(ref named_path, _, synthetic) = typ.typ {
                (
                    Some(named_path.last_segment().span()),
                    named_path.last_segment().is_self_type_name(),
                    synthetic,
                )
            } else {
                (None, false, false)
            };

        let resolved_type = match typ.typ {
            FieldElement => Type::FieldElement,
            Array(size, elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, kind));
                let mut size = self.convert_expression_type(size);
                // TODO(https://github.com/noir-lang/noir/issues/5156): Remove this once we only have explicit numeric generics
                if let Type::NamedGeneric(type_var, name, _) = size {
                    size = Type::NamedGeneric(
                        type_var,
                        name,
                        Kind::Numeric(Box::new(Type::default_int_type())),
                    );
                }
                Type::Array(Box::new(size), elem)
            }
            Slice(elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, kind));
                Type::Slice(elem)
            }
            Expression(expr) => self.convert_expression_type(expr),
            Integer(sign, bits) => Type::Integer(sign, bits),
            Bool => Type::Bool,
            String(size) => {
                let mut resolved_size = self.convert_expression_type(size);
                // TODO(https://github.com/noir-lang/noir/issues/5156): Remove this once we only have explicit numeric generics
                if let Type::NamedGeneric(type_var, name, _) = resolved_size {
                    resolved_size = Type::NamedGeneric(
                        type_var,
                        name,
                        Kind::Numeric(Box::new(Type::default_int_type())),
                    );
                }
                Type::String(Box::new(resolved_size))
            }
            FormatString(size, fields) => {
                let mut resolved_size = self.convert_expression_type(size);
                if let Type::NamedGeneric(type_var, name, _) = resolved_size {
                    resolved_size = Type::NamedGeneric(
                        type_var,
                        name,
                        Kind::Numeric(Box::new(Type::default_int_type())),
                    );
                }
                let fields = self.resolve_type_inner(*fields, kind);
                Type::FmtString(Box::new(resolved_size), Box::new(fields))
            }
            Quoted(quoted) => Type::Quoted(quoted),
            Unit => Type::Unit,
            Unspecified => Type::Error,
            Error => Type::Error,
            Named(path, args, _) => self.resolve_named_type(path, args),
            TraitAsType(path, args) => self.resolve_trait_as_type(path, args),

            Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field, kind)))
            }
            Function(args, ret, env) => {
                let args = vecmap(args, |arg| self.resolve_type_inner(arg, kind));
                let ret = Box::new(self.resolve_type_inner(*ret, kind));

                // expect() here is valid, because the only places we don't have a span are omitted types
                // e.g. a function without return type implicitly has a spanless UnresolvedType::Unit return type
                // To get an invalid env type, the user must explicitly specify the type, which will have a span
                let env_span =
                    env.span.expect("Unexpected missing span for closure environment type");

                let env = Box::new(self.resolve_type_inner(*env, kind));

                match *env {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_, _, _) => {
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
                Type::MutableReference(Box::new(self.resolve_type_inner(*element, kind)))
            }
            Parenthesized(typ) => self.resolve_type_inner(*typ, kind),
            Resolved(id) => self.interner.get_quoted_type(id).clone(),
        };

        if let Some(unresolved_span) = typ.span {
            let location = Location::new(named_path_span.unwrap_or(unresolved_span), self.file);

            match resolved_type {
                Type::Struct(ref struct_type, _) => {
                    // Record the location of the type reference
                    self.interner.push_type_ref_location(resolved_type.clone(), location);

                    if !is_synthetic {
                        self.interner.add_struct_reference(
                            struct_type.borrow().id,
                            location,
                            is_self_type_name,
                        );
                    }
                }
                Type::Alias(ref alias_type, _) => {
                    self.interner.add_alias_reference(alias_type.borrow().id, location);
                }
                _ => (),
            }
        }

        // Check that any types with a type kind match the expected type kind supplied to this function
        // TODO(https://github.com/noir-lang/noir/issues/5156): make this named generic check more general with `*resolved_kind != kind`
        // as implicit numeric generics still existing makes this check more challenging to enforce
        // An example of a more general check that we should switch to:
        // if resolved_type.kind() != kind.clone() {
        //     let expected_typ_err = CompilationError::TypeError(TypeCheckError::TypeKindMismatch {
        //         expected_kind: kind.to_string(),
        //         expr_kind: resolved_type.kind().to_string(),
        //         expr_span: span.expect("Type should have span"),
        //     });
        //     self.errors.push((expected_typ_err, self.file));
        //     return Type::Error;
        // }
        if let Type::NamedGeneric(_, name, resolved_kind) = &resolved_type {
            if matches!(resolved_kind, Kind::Numeric { .. }) && matches!(kind, Kind::Normal) {
                let expected_typ_err = ResolverError::NumericGenericUsedForType {
                    name: name.to_string(),
                    span: span.expect("Type should have span"),
                };
                self.push_err(expected_typ_err);
                return Type::Error;
            }
        }

        resolved_type
    }

    pub fn find_generic(&self, target_name: &str) -> Option<&ResolvedGeneric> {
        self.generics.iter().find(|generic| generic.name.as_ref() == target_name)
    }

    fn resolve_named_type(&mut self, path: Path, args: Vec<UnresolvedType>) -> Type {
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
            } else if name == WILDCARD_TYPE {
                return self.interner.next_type_variable();
            }
        }

        let span = path.span();

        if let Some(type_alias) = self.lookup_type_alias(path.clone()) {
            let type_alias = type_alias.borrow();
            let expected_generic_count = type_alias.generics.len();
            let type_alias_string = type_alias.to_string();
            let id = type_alias.id;

            let mut args = vecmap(type_alias.generics.iter().zip(args), |(generic, arg)| {
                self.resolve_type_inner(arg, &generic.kind)
            });

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
                if !self.in_contract()
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

                let mut args =
                    vecmap(struct_type.borrow().generics.iter().zip(args), |(generic, arg)| {
                        self.resolve_type_inner(arg, &generic.kind)
                    });

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

    fn resolve_trait_as_type(&mut self, path: Path, args: Vec<UnresolvedType>) -> Type {
        // Fetch information needed from the trait as the closure for resolving all the `args`
        // requires exclusive access to `self`
        let trait_as_type_info = self
            .lookup_trait_or_error(path)
            .map(|t| (t.id, Rc::new(t.name.to_string()), t.generics.clone()));

        if let Some((id, name, resolved_generics)) = trait_as_type_info {
            assert_eq!(resolved_generics.len(), args.len());
            let generics_with_types = resolved_generics.iter().zip(args);
            let args = vecmap(generics_with_types, |(generic, typ)| {
                self.resolve_type_inner(typ, &generic.kind)
            });
            Type::TraitAsType(id, Rc::new(name.to_string()), args)
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

    pub fn lookup_generic_or_global_type(&mut self, path: &Path) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = &path.last_segment().0.contents;
            if let Some(generic) = self.find_generic(name) {
                let generic = generic.clone();
                return Some(Type::NamedGeneric(generic.type_var, generic.name, generic.kind));
            }
        }

        // If we cannot find a local generic of the same name, try to look up a global
        match self.resolve_path(path.clone()) {
            Ok(ModuleDefId::GlobalId(id)) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                let reference_location = Location::new(path.span(), self.file);
                self.interner.add_global_reference(id, reference_location);

                Some(Type::Constant(self.eval_global_as_array_length(id, path)))
            }
            _ => None,
        }
    }

    pub(super) fn convert_expression_type(&mut self, length: UnresolvedTypeExpression) -> Type {
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

    // this resolves Self::some_static_method, inside an impl block (where we don't have a concrete self_type)
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
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
                    trait_generics: Type::from_generics(&vecmap(&the_trait.generics, |generic| {
                        generic.type_var.clone()
                    })),
                    trait_id,
                    span: path.span(),
                };

                return Some((method, constraint, false));
            }
        }
        None
    }

    // this resolves TraitName::some_static_method
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        let func_id: FuncId = self.lookup(path.clone()).ok()?;
        let meta = self.interner.function_meta(&func_id);
        let trait_id = meta.trait_id?;
        let the_trait = self.interner.get_trait(trait_id);
        let method = the_trait.find_method(&path.last_segment().0.contents)?;
        let constraint = TraitConstraint {
            typ: Type::TypeVariable(the_trait.self_type_typevar.clone(), TypeVariableKind::Normal),
            trait_generics: Type::from_generics(&vecmap(&the_trait.generics, |generic| {
                generic.type_var.clone()
            })),
            trait_id,
            span: path.span(),
        };
        Some((method, constraint, false))
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

        for constraint in self.trait_bounds.clone() {
            if let Type::NamedGeneric(_, name, _) = &constraint.typ {
                // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
                if path.segments[0].0.contents != name.as_str() {
                    continue;
                }

                let the_trait = self.interner.get_trait(constraint.trait_id);
                if let Some(method) =
                    the_trait.find_method(path.segments.last().unwrap().0.contents.as_str())
                {
                    return Some((method, constraint, true));
                }
            }
        }
        None
    }

    // Try to resolve the given trait method path.
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    pub(super) fn resolve_trait_generic_path(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        self.resolve_trait_static_method_by_self(path)
            .or_else(|| self.resolve_trait_static_method(path))
            .or_else(|| self.resolve_trait_method_by_named_generic(path))
    }

    fn eval_global_as_array_length(&mut self, global_id: GlobalId, path: &Path) -> u32 {
        let Some(stmt) = self.interner.get_global_let_statement(global_id) else {
            if let Some(global) = self.unresolved_globals.remove(&global_id) {
                self.elaborate_global(global);
                return self.eval_global_as_array_length(global_id, path);
            } else {
                let path = path.clone();
                self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                return 0;
            }
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
            HirExpression::Ident(ident, _) => {
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
            HirExpression::Cast(cast) => {
                let lhs = self.try_eval_array_length_id_with_fuel(cast.lhs, span, fuel - 1)?;
                let lhs_value = Value::Field(lhs.into());
                let evaluated_value =
                    Interpreter::evaluate_cast_one_step(&cast, rhs, lhs_value, self.interner)
                        .map_err(|error| Some(ResolverError::ArrayLengthInterpreter { error }))?;

                evaluated_value
                    .to_u128()
                    .ok_or_else(|| Some(ResolverError::InvalidArrayLengthExpr { span }))
            }
            _other => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }

    pub(super) fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut errors = Vec::new();
        actual.unify(expected, &mut errors, make_error);
        self.errors.extend(errors.into_iter().map(|error| (error.into(), self.file)));
    }

    /// Wrapper of Type::unify_with_coercions using self.errors
    pub(super) fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut errors = Vec::new();
        actual.unify_with_coercions(expected, expression, self.interner, &mut errors, make_error);
        self.errors.extend(errors.into_iter().map(|error| (error.into(), self.file)));
    }

    /// Return a fresh integer or field type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(self.interner);
        self.push_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(self.interner);
        self.push_type_variable(typ.clone());
        typ
    }

    /// Translates a (possibly Unspecified) UnresolvedType to a Type.
    /// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
    pub(super) fn resolve_inferred_type(&mut self, typ: UnresolvedType) -> Type {
        match &typ.typ {
            UnresolvedTypeData::Unspecified => self.interner.next_type_variable(),
            _ => self.resolve_type(typ),
        }
    }

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    pub(super) fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if let Type::MutableReference(element) = typ {
            let location = self.interner.id_location(object);

            let object = self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: UnaryOp::Dereference { implicitly_added: true },
                rhs: object,
                trait_method_id: None,
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

    fn bind_function_type_impl(
        &mut self,
        fn_params: &[Type],
        fn_ret: &Type,
        callsite_args: &[(Type, ExprId, Span)],
        span: Span,
    ) -> Type {
        if fn_params.len() != callsite_args.len() {
            self.push_err(TypeCheckError::ParameterCountMismatch {
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

    pub(super) fn bind_function_type(
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
                    self.push_err(error);
                }
                ret
            }
            // The closure env is ignored on purpose: call arguments never place
            // constraints on closure environments.
            Type::Function(parameters, ret, _env) => {
                self.bind_function_type_impl(&parameters, &ret, &args, span)
            }
            Type::Error => Type::Error,
            found => {
                self.push_err(TypeCheckError::ExpectedFunction { found, span });
                Type::Error
            }
        }
    }

    pub(super) fn check_cast(&mut self, from: Type, to: &Type, span: Span) -> Type {
        match from.follow_bindings() {
            Type::Integer(..)
            | Type::FieldElement
            | Type::TypeVariable(_, TypeVariableKind::IntegerOrField)
            | Type::TypeVariable(_, TypeVariableKind::Integer)
            | Type::Bool => (),

            Type::TypeVariable(_, _) => {
                self.push_err(TypeCheckError::TypeAnnotationsNeeded { span });
                return Type::Error;
            }
            Type::Error => return Type::Error,
            from => {
                self.push_err(TypeCheckError::InvalidCast { from, span });
                return Type::Error;
            }
        }

        match to {
            Type::Integer(sign, bits) => Type::Integer(*sign, *bits),
            Type::FieldElement => Type::FieldElement,
            Type::Bool => Type::Bool,
            Type::Error => Type::Error,
            _ => {
                self.push_err(TypeCheckError::UnsupportedCast { span });
                Type::Error
            }
        }
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
    pub(super) fn infix_operand_type_rules(
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

    // Given a unary operator and a type, this method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    pub(super) fn prefix_operand_type_rules(
        &mut self,
        op: &UnaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match op {
            crate::ast::UnaryOp::Minus | crate::ast::UnaryOp::Not => {
                match rhs_type {
                    // An error type will always return an error
                    Error => Ok((Error, false)),
                    Alias(alias, args) => {
                        let alias = alias.borrow().get_type(args);
                        self.prefix_operand_type_rules(op, &alias, span)
                    }

                    // Matches on TypeVariable must be first so that we follow any type
                    // bindings.
                    TypeVariable(int, _) => {
                        if let TypeBinding::Bound(binding) = &*int.borrow() {
                            return self.prefix_operand_type_rules(op, binding, span);
                        }

                        // The `!` prefix operator is not valid for Field, so if this is a numeric
                        // type we constrain it to just (non-Field) integer types.
                        if matches!(op, crate::ast::UnaryOp::Not) && rhs_type.is_numeric() {
                            let integer_type = Type::polymorphic_integer(self.interner);
                            self.unify(rhs_type, &integer_type, || {
                                TypeCheckError::InvalidUnaryOp { kind: rhs_type.to_string(), span }
                            });
                        }

                        Ok((rhs_type.clone(), !rhs_type.is_numeric()))
                    }
                    Integer(sign_x, bit_width_x) => {
                        if *op == UnaryOp::Minus && *sign_x == Signedness::Unsigned {
                            return Err(TypeCheckError::InvalidUnaryOp {
                                kind: rhs_type.to_string(),
                                span,
                            });
                        }
                        Ok((Integer(*sign_x, *bit_width_x), false))
                    }
                    // The result of a Field is always a witness
                    FieldElement => {
                        if *op == UnaryOp::Not {
                            return Err(TypeCheckError::FieldNot { span });
                        }
                        Ok((FieldElement, false))
                    }

                    Bool => Ok((Bool, false)),

                    _ => Ok((rhs_type.clone(), true)),
                }
            }
            crate::ast::UnaryOp::MutableReference => {
                Ok((Type::MutableReference(Box::new(rhs_type.follow_bindings())), false))
            }
            crate::ast::UnaryOp::Dereference { implicitly_added: _ } => {
                let element_type = self.interner.next_type_variable();
                let expected = Type::MutableReference(Box::new(element_type.clone()));
                self.unify(rhs_type, &expected, || TypeCheckError::TypeMismatch {
                    expr_typ: rhs_type.to_string(),
                    expected_typ: expected.to_string(),
                    expr_span: span,
                });
                Ok((element_type, false))
            }
        }
    }

    /// Prerequisite: verify_trait_constraint of the operator's trait constraint.
    ///
    /// Although by this point the operator is expected to already have a trait impl,
    /// we still need to match the operator's type against the method's instantiated type
    /// to ensure the instantiation bindings are correct and the monomorphizer can
    /// re-apply the needed bindings.
    pub(super) fn type_check_operator_method(
        &mut self,
        expr_id: ExprId,
        trait_method_id: TraitMethodId,
        object_type: &Type,
        span: Span,
    ) {
        let the_trait = self.interner.get_trait(trait_method_id.trait_id);

        let method = &the_trait.methods[trait_method_id.method_index];
        let (method_type, mut bindings) = method.typ.clone().instantiate(self.interner);

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

    pub(super) fn type_check_member_access(
        &mut self,
        mut access: HirMemberAccess,
        expr_id: ExprId,
        lhs_type: Type,
        span: Span,
    ) -> Type {
        let access_lhs = &mut access.lhs;

        let dereference_lhs = |this: &mut Self, lhs_type, element| {
            let old_lhs = *access_lhs;
            *access_lhs = this.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: crate::ast::UnaryOp::Dereference { implicitly_added: true },
                rhs: old_lhs,
                trait_method_id: None,
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

    pub(super) fn lookup_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        span: Span,
    ) -> Option<HirMethodReference> {
        match object_type.follow_bindings() {
            Type::Struct(typ, _args) => {
                let id = typ.borrow().id;
                match self.interner.lookup_method(object_type, id, method_name, false) {
                    Some(method_id) => Some(HirMethodReference::FuncId(method_id)),
                    None => {
                        self.push_err(TypeCheckError::UnresolvedMethodCall {
                            method_name: method_name.to_string(),
                            object_type: object_type.clone(),
                            span,
                        });
                        None
                    }
                }
            }
            // TODO: We should allow method calls on `impl Trait`s eventually.
            //       For now it is fine since they are only allowed on return types.
            Type::TraitAsType(..) => {
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span,
                });
                None
            }
            Type::NamedGeneric(_, _, _) => {
                let func_id = match self.current_item {
                    Some(DependencyId::Function(id)) => id,
                    _ => panic!("unexpected method outside a function"),
                };
                let func_meta = self.interner.function_meta(&func_id);

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

                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span,
                });
                None
            }
            // Mutable references to another type should resolve to methods of their element type.
            // This may be a struct or a primitive type.
            Type::MutableReference(element) => self
                .interner
                .lookup_primitive_trait_method_mut(element.as_ref(), method_name)
                .map(HirMethodReference::FuncId)
                .or_else(|| self.lookup_method(&element, method_name, span)),

            // If we fail to resolve the object to a struct type, we have no way of type
            // checking its arguments as we can't even resolve the name of the function
            Type::Error => None,

            // The type variable must be unbound at this point since follow_bindings was called
            Type::TypeVariable(_, TypeVariableKind::Normal) => {
                self.push_err(TypeCheckError::TypeAnnotationsNeeded { span });
                None
            }

            other => match self.interner.lookup_primitive_method(&other, method_name) {
                Some(method_id) => Some(HirMethodReference::FuncId(method_id)),
                None => {
                    self.push_err(TypeCheckError::UnresolvedMethodCall {
                        method_name: method_name.to_string(),
                        object_type: object_type.clone(),
                        span,
                    });
                    None
                }
            },
        }
    }

    pub(super) fn type_check_call(
        &mut self,
        call: &HirCallExpression,
        func_type: Type,
        args: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        self.run_lint(|elaborator| {
            lints::deprecated_function(elaborator.interner, call.func).map(Into::into)
        });

        let is_current_func_constrained = self.in_constrained_function();

        let is_unconstrained_call = self.is_unconstrained_call(call.func);
        let crossing_runtime_boundary = is_current_func_constrained && is_unconstrained_call;
        if crossing_runtime_boundary {
            let called_func_id = self
                .interner
                .lookup_function_from_expr(&call.func)
                .expect("Called function should exist");
            self.run_lint(|elaborator| {
                lints::oracle_called_from_constrained_function(
                    elaborator.interner,
                    &called_func_id,
                    is_current_func_constrained,
                    span,
                )
                .map(Into::into)
            });
            let errors = lints::unconstrained_function_args(&args);
            for error in errors {
                self.push_err(error);
            }
        }

        let return_type = self.bind_function_type(func_type, args, span);

        if crossing_runtime_boundary {
            self.run_lint(|_| {
                lints::unconstrained_function_return(&return_type, span).map(Into::into)
            });
        }

        return_type
    }

    fn is_unconstrained_call(&self, expr: ExprId) -> bool {
        if let Some(func_id) = self.interner.lookup_function_from_expr(&expr) {
            let modifiers = self.interner.function_modifiers(&func_id);
            modifiers.is_unconstrained
        } else {
            false
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
    pub(super) fn try_add_mutable_reference_to_object(
        &mut self,
        function_type: &Type,
        object_type: &mut Type,
        object: &mut ExprId,
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
                    if let Err(error) = verify_mutable_reference(self.interner, *object) {
                        self.push_err(TypeCheckError::ResolverError(error));
                    }

                    let new_type = Type::MutableReference(Box::new(actual_type));
                    *object_type = new_type.clone();

                    // First try to remove a dereference operator that may have been implicitly
                    // inserted by a field access expression `foo.bar` on a mutable reference `foo`.
                    let new_object = self.try_remove_implicit_dereference(*object);

                    // If that didn't work, then wrap the whole expression in an `&mut`
                    *object = new_object.unwrap_or_else(|| {
                        let location = self.interner.id_location(*object);

                        let new_object =
                            self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                                operator: UnaryOp::MutableReference,
                                rhs: *object,
                                trait_method_id: None,
                            }));
                        self.interner.push_expr_type(new_object, new_type);
                        self.interner.push_expr_location(new_object, location.span, location.file);
                        new_object
                    });
                }
            // Otherwise if the object type is a mutable reference and the method is not, insert as
            // many dereferences as needed.
            } else if matches!(actual_type, Type::MutableReference(_)) {
                let (new_object, new_type) = self.insert_auto_dereferences(*object, actual_type);
                *object_type = new_type;
                *object = new_object;
            }
        }
    }

    pub fn type_check_function_body(&mut self, body_type: Type, meta: &FuncMeta, body_id: ExprId) {
        let (expr_span, empty_function) = self.function_info(body_id);
        let declared_return_type = meta.return_type();

        let func_span = self.interner.expr_span(&body_id); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        if let Type::TraitAsType(trait_id, _, generics) = declared_return_type {
            if self.interner.lookup_trait_implementation(&body_type, *trait_id, generics).is_err() {
                self.push_err(TypeCheckError::TypeMismatchWithSource {
                    expected: declared_return_type.clone(),
                    actual: body_type,
                    span: func_span,
                    source: Source::Return(meta.return_type.clone(), expr_span),
                });
            }
        } else {
            self.unify_with_coercions(&body_type, declared_return_type, body_id, || {
                let mut error = TypeCheckError::TypeMismatchWithSource {
                    expected: declared_return_type.clone(),
                    actual: body_type.clone(),
                    span: func_span,
                    source: Source::Return(meta.return_type.clone(), expr_span),
                };

                if empty_function {
                    error = error.add_context(
                        "implicitly returns `()` as its body has no tail or `return` expression",
                    );
                }
                error
            });
        }
    }

    fn function_info(&self, function_body_id: ExprId) -> (noirc_errors::Span, bool) {
        let (expr_span, empty_function) =
            if let HirExpression::Block(block) = self.interner.expression(&function_body_id) {
                let last_stmt = block.statements().last();
                let mut span = self.interner.expr_span(&function_body_id);

                if let Some(last_stmt) = last_stmt {
                    if let HirStatement::Expression(expr) = self.interner.statement(last_stmt) {
                        span = self.interner.expr_span(&expr);
                    }
                }

                (span, last_stmt.is_none())
            } else {
                (self.interner.expr_span(&function_body_id), false)
            };
        (expr_span, empty_function)
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
                    self.push_err(TypeCheckError::TypeAnnotationsNeeded { span });
                } else if let Some(error) =
                    NoMatchingImplFoundError::new(self.interner, erroring_constraints, span)
                {
                    self.push_err(TypeCheckError::NoMatchingImplFound(error));
                }
            }
        }
    }

    pub fn add_existing_generics(
        &mut self,
        unresolved_generics: &UnresolvedGenerics,
        generics: &Generics,
    ) {
        assert_eq!(unresolved_generics.len(), generics.len());

        for (unresolved_generic, generic) in unresolved_generics.iter().zip(generics) {
            self.add_existing_generic(unresolved_generic, unresolved_generic.span(), generic);
        }
    }

    pub fn add_existing_generic(
        &mut self,
        unresolved_generic: &UnresolvedGeneric,
        span: Span,
        resolved_generic: &ResolvedGeneric,
    ) {
        let name = &unresolved_generic.ident().0.contents;

        if let Some(generic) = self.find_generic(name) {
            self.push_err(ResolverError::DuplicateDefinition {
                name: name.clone(),
                first_span: generic.span,
                second_span: span,
            });
        } else {
            self.generics.push(resolved_generic.clone());
        }
    }

    pub fn find_numeric_generics(
        parameters: &Parameters,
        return_type: &Type,
    ) -> Vec<(String, TypeVariable)> {
        let mut found = BTreeMap::new();
        for (_, parameter, _) in &parameters.0 {
            Self::find_numeric_generics_in_type(parameter, &mut found);
        }
        Self::find_numeric_generics_in_type(return_type, &mut found);
        found.into_iter().collect()
    }

    fn find_numeric_generics_in_type(typ: &Type, found: &mut BTreeMap<String, TypeVariable>) {
        match typ {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::TypeVariable(_, _)
            | Type::Constant(_)
            | Type::NamedGeneric(_, _, _)
            | Type::Quoted(_)
            | Type::Forall(_, _) => (),

            Type::TraitAsType(_, _, args) => {
                for arg in args {
                    Self::find_numeric_generics_in_type(arg, found);
                }
            }

            Type::Array(length, element_type) => {
                if let Type::NamedGeneric(type_variable, name, _) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
                Self::find_numeric_generics_in_type(element_type, found);
            }

            Type::Slice(element_type) => {
                Self::find_numeric_generics_in_type(element_type, found);
            }

            Type::Tuple(fields) => {
                for field in fields {
                    Self::find_numeric_generics_in_type(field, found);
                }
            }

            Type::Function(parameters, return_type, _env) => {
                for parameter in parameters {
                    Self::find_numeric_generics_in_type(parameter, found);
                }
                Self::find_numeric_generics_in_type(return_type, found);
            }

            Type::Struct(struct_type, generics) => {
                for (i, generic) in generics.iter().enumerate() {
                    if let Type::NamedGeneric(type_variable, name, _) = generic {
                        if struct_type.borrow().generic_is_numeric(i) {
                            found.insert(name.to_string(), type_variable.clone());
                        }
                    } else {
                        Self::find_numeric_generics_in_type(generic, found);
                    }
                }
            }
            Type::Alias(alias, generics) => {
                for (i, generic) in generics.iter().enumerate() {
                    if let Type::NamedGeneric(type_variable, name, _) = generic {
                        if alias.borrow().generic_is_numeric(i) {
                            found.insert(name.to_string(), type_variable.clone());
                        }
                    } else {
                        Self::find_numeric_generics_in_type(generic, found);
                    }
                }
            }
            Type::MutableReference(element) => Self::find_numeric_generics_in_type(element, found),
            Type::String(length) => {
                if let Type::NamedGeneric(type_variable, name, _) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
            }
            Type::FmtString(length, fields) => {
                if let Type::NamedGeneric(type_variable, name, _) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
                Self::find_numeric_generics_in_type(fields, found);
            }
        }
    }

    /// Push a type variable into the current FunctionContext to be defaulted if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    fn push_type_variable(&mut self, typ: Type) {
        let context = self.function_context.last_mut();
        let context = context.expect("The function_context stack should always be non-empty");
        context.type_variables.push(typ);
    }

    /// Push a trait constraint into the current FunctionContext to be solved if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    pub fn push_trait_constraint(&mut self, constraint: TraitConstraint, expr_id: ExprId) {
        let context = self.function_context.last_mut();
        let context = context.expect("The function_context stack should always be non-empty");
        context.trait_constraints.push((constraint, expr_id));
    }
}

/// Gives an error if a user tries to create a mutable reference
/// to an immutable variable.
fn verify_mutable_reference(interner: &NodeInterner, rhs: ExprId) -> Result<(), ResolverError> {
    match interner.expression(&rhs) {
        HirExpression::MemberAccess(member_access) => {
            verify_mutable_reference(interner, member_access.lhs)
        }
        HirExpression::Index(_) => {
            let span = interner.expr_span(&rhs);
            Err(ResolverError::MutableReferenceToArrayElement { span })
        }
        HirExpression::Ident(ident, _) => {
            if let Some(definition) = interner.try_definition(ident.id) {
                if !definition.mutable {
                    return Err(ResolverError::MutableReferenceToImmutableVariable {
                        span: interner.expr_span(&rhs),
                        variable: definition.name.clone(),
                    });
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
