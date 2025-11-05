use std::borrow::Cow;

use noirc_errors::Location;

use crate::{
    BinaryTypeOperator, Kind, QuotedType, Type, TypeBinding, TypeBindings, TypeVariable,
    hir::{def_collector::dc_crate::CompilationError, type_check::TypeCheckError},
    hir_def::{
        expr::{HirCallExpression, HirExpression, HirIdent},
        types,
    },
    node_interner::{ExprId, NodeInterner},
};

pub struct UnificationError;

enum FunctionCoercionResult {
    NoCoercion,
    Coerced(Type),
    UnconstrainedMismatch(Type),
}

/// When unifying types we sometimes need to adjust the algorithm a bit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum UnificationFlags {
    /// Nothing special to do.
    None,
    /// If the right-hand side is `expr op constant`, don't try to move the constant to the left-hand side.
    DoNotMoveConstantsOnTheRight,
}

impl Kind {
    /// Unifies this kind with the other. Returns true on success
    pub(crate) fn unifies(&self, other: &Kind) -> bool {
        match (self, other) {
            // Kind::Any unifies with everything
            (Kind::Any, _) | (_, Kind::Any) => true,

            // Kind::Normal unifies with Kind::Integer and Kind::IntegerOrField
            (Kind::Normal, Kind::Integer | Kind::IntegerOrField)
            | (Kind::Integer | Kind::IntegerOrField, Kind::Normal) => true,

            // Kind::Integer unifies with Kind::IntegerOrField
            (Kind::Integer | Kind::IntegerOrField, Kind::Integer | Kind::IntegerOrField) => true,

            // Kind::IntegerOrField unifies with Kind::Numeric(_)
            (Kind::IntegerOrField, Kind::Numeric(_typ))
            | (Kind::Numeric(_typ), Kind::IntegerOrField) => true,

            // Kind::Numeric unifies along its Type argument
            (Kind::Numeric(lhs), Kind::Numeric(rhs)) => {
                let mut bindings = TypeBindings::default();
                let unifies = lhs.try_unify(rhs, &mut bindings).is_ok();
                if unifies {
                    Type::apply_type_bindings(bindings);
                }
                unifies
            }

            // everything unifies with itself
            (lhs, rhs) => lhs == rhs,
        }
    }

    pub(crate) fn unify(&self, other: &Kind) -> Result<(), UnificationError> {
        if self.unifies(other) { Ok(()) } else { Err(UnificationError) }
    }
}

impl Type {
    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. When comparing types, unification
    /// (including try_unify) are almost always preferred over Type::eq as unification
    /// will correctly handle generic types.
    pub fn unify(&self, expected: &Type) -> Result<(), UnificationError> {
        let mut bindings = TypeBindings::default();

        self.try_unify(expected, &mut bindings).map(|()| {
            // Commit any type bindings on success
            Self::apply_type_bindings(bindings);
        })
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    pub fn try_unify(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        self.try_unify_with_flags(other, UnificationFlags::None, bindings)
    }

    fn try_unify_with_flags(
        &self,
        other: &Type,
        flags: UnificationFlags,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        use Type::*;

        // If the two types are exactly the same then they trivially unify.
        // This check avoids potentially unifying very complex types (usually infix
        // expressions) when they are the same.
        if self == other {
            return Ok(());
        }

        let lhs = self.follow_bindings_shallow();
        let rhs = other.follow_bindings_shallow();

        let lhs = match lhs.as_ref() {
            InfixExpr(..) => Cow::Owned(self.substitute(bindings).canonicalize()),
            other => Cow::Borrowed(other),
        };

        let rhs = match rhs.as_ref() {
            InfixExpr(..) => Cow::Owned(other.substitute(bindings).canonicalize()),
            other => Cow::Borrowed(other),
        };

        match (lhs.as_ref(), rhs.as_ref()) {
            (Error, _) | (_, Error) => Ok(()),

            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                alias.try_unify_with_flags(other, flags, bindings)
            }

            (TypeVariable(var), other) | (other, TypeVariable(var)) => match &*var.borrow() {
                TypeBinding::Bound(typ) => {
                    if typ.is_numeric_value() {
                        other.try_unify_to_type_variable(var, flags, bindings, |bindings| {
                            let only_integer = matches!(typ, Integer(..));
                            other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                        })
                    } else {
                        other.try_unify_to_type_variable(var, flags, bindings, |bindings| {
                            other.try_bind_to(var, bindings, typ.kind())
                        })
                    }
                }
                TypeBinding::Unbound(_id, Kind::IntegerOrField) => other
                    .try_unify_to_type_variable(var, flags, bindings, |bindings| {
                        let only_integer = false;
                        other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }),
                TypeBinding::Unbound(_id, Kind::Integer) => {
                    other.try_unify_to_type_variable(var, flags, bindings, |bindings| {
                        let only_integer = true;
                        other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    })
                }
                TypeBinding::Unbound(_id, type_var_kind) => {
                    other.try_unify_to_type_variable(var, flags, bindings, |bindings| {
                        other.try_bind_to(var, bindings, type_var_kind.clone())
                    })
                }
            },

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.try_unify(len_b, bindings)?;
                elem_a.try_unify(elem_b, bindings)
            }

            (Slice(elem_a), Slice(elem_b)) => elem_a.try_unify(elem_b, bindings),

            (String(len_a), String(len_b)) => len_a.try_unify(len_b, bindings),

            (FmtString(len_a, elements_a), FmtString(len_b, elements_b)) => {
                len_a.try_unify(len_b, bindings)?;
                elements_a.try_unify(elements_b, bindings)
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(UnificationError)
                } else {
                    for (a, b) in elements_a.iter().zip(elements_b) {
                        a.try_unify(b, bindings)?;
                    }
                    Ok(())
                }
            }

            // No recursive try_unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (DataType(id_a, args_a), DataType(id_b, args_b)) => {
                if id_a == id_b && args_a.len() == args_b.len() {
                    for (a, b) in args_a.iter().zip(args_b) {
                        a.try_unify(b, bindings)?;
                    }
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }

            (CheckedCast { to, .. }, other) | (other, CheckedCast { to, .. }) => {
                to.try_unify_with_flags(other, flags, bindings)
            }

            (NamedGeneric(types::NamedGeneric { type_var, .. }), other)
            | (other, NamedGeneric(types::NamedGeneric { type_var, .. }))
                if !type_var.borrow().is_unbound() =>
            {
                if let TypeBinding::Bound(link) = &*type_var.borrow() {
                    link.try_unify_with_flags(other, flags, bindings)
                } else {
                    unreachable!("If guard ensures binding is bound")
                }
            }

            (
                NamedGeneric(types::NamedGeneric { type_var: binding_a, name: name_a, .. }),
                NamedGeneric(types::NamedGeneric { type_var: binding_b, name: name_b, .. }),
            ) => {
                // Bound NamedGenerics are caught by the check above
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    binding_a.kind().unify(&binding_b.kind())
                } else {
                    Err(UnificationError)
                }
            }

            (
                Function(params_a, ret_a, env_a, unconstrained_a),
                Function(params_b, ret_b, env_b, unconstrained_b),
            ) => {
                if unconstrained_a == unconstrained_b && params_a.len() == params_b.len() {
                    for (a, b) in params_a.iter().zip(params_b.iter()) {
                        a.try_unify(b, bindings)?;
                    }

                    env_a.try_unify(env_b, bindings)?;
                    ret_b.try_unify(ret_a, bindings)
                } else {
                    Err(UnificationError)
                }
            }

            (Reference(elem_a, mutable_a), Reference(elem_b, mutable_b)) => {
                if mutable_a == mutable_b {
                    elem_a.try_unify(elem_b, bindings)
                } else {
                    Err(UnificationError)
                }
            }

            (InfixExpr(lhs_a, op_a, rhs_a, _), InfixExpr(lhs_b, op_b, rhs_b, _)) => {
                if op_a == op_b {
                    // We need to preserve the original bindings since if syntactic equality
                    // fails we fall back to other equality strategies.
                    let mut new_bindings = bindings.clone();
                    let lhs_result = lhs_a.try_unify(lhs_b, &mut new_bindings);
                    let rhs_result = rhs_a.try_unify(rhs_b, &mut new_bindings);

                    if lhs_result.is_ok() && rhs_result.is_ok() {
                        *bindings = new_bindings;
                        return Ok(());
                    }
                }

                lhs.try_unify_by_isolating_an_unbound_type_variable(&rhs, bindings).or_else(|_| {
                    lhs.try_unify_by_moving_single_constant_term(&rhs, flags, bindings)
                })
            }

            (Constant(value, kind), other) | (other, Constant(value, kind)) => {
                let dummy_location = Location::dummy();
                let other = other.substitute(bindings);
                if let Ok(other_value) = other.evaluate_to_signed_field(kind, dummy_location) {
                    if *value == other_value && kind.unifies(&other.kind()) {
                        Ok(())
                    } else {
                        Err(UnificationError)
                    }
                } else if let InfixExpr(lhs, op, rhs, _) = other {
                    if let Some(inverse) = op.approx_inverse() {
                        // Handle cases like `4 = a + b` by trying to solve to `a = 4 - b`
                        let new_type = Type::inverted_infix_expr(
                            Box::new(Constant(*value, kind.clone())),
                            inverse,
                            rhs.clone(),
                        );

                        new_type.try_unify(&lhs, bindings)?;
                        Ok(())
                    } else {
                        Err(UnificationError)
                    }
                } else {
                    Err(UnificationError)
                }
            }

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }
        }
    }

    /// Try to unify a type variable to `self`.
    /// This is a helper function factored out from try_unify.
    fn try_unify_to_type_variable(
        &self,
        type_variable: &TypeVariable,
        flags: UnificationFlags,
        bindings: &mut TypeBindings,

        // Bind the type variable to a type. This is factored out since depending on the
        // Kind, there are different methods to check whether the variable can
        // bind to the given type or not.
        bind_variable: impl FnOnce(&mut TypeBindings) -> Result<(), UnificationError>,
    ) -> Result<(), UnificationError> {
        match &*type_variable.borrow() {
            // If it is already bound, unify against what it is bound to
            TypeBinding::Bound(link) => link.try_unify_with_flags(self, flags, bindings),
            TypeBinding::Unbound(id, _) => {
                // We may have already "bound" this type variable in this call to
                // try_unify, so check those bindings as well.
                match bindings.get(id) {
                    Some((_, kind, binding)) => {
                        if !kind.unifies(&binding.kind()) {
                            return Err(UnificationError);
                        }
                        binding.clone().try_unify_with_flags(self, flags, bindings)
                    }

                    // Otherwise, bind it
                    None => bind_variable(bindings),
                }
            }
        }
    }

    /// Try to unify the following equations:
    /// - `A + rhs = other` -> `A = other - rhs`
    /// - `A - rhs = other` -> `A = other + rhs`
    /// - `lhs + B = other` -> `B = other - lhs`
    /// - `lhs - B = other` -> `B = lhs - other`
    /// - `other = A + rhs` -> `A = other - rhs`
    /// - `other = A - rhs` -> `A = other + rhs`
    /// - `other = lhs + B` -> `B = other - lhs`
    /// - `other = lhs - B` -> `B = lhs - other`
    pub(super) fn try_unify_by_isolating_an_unbound_type_variable(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        self.try_unify_by_isolating_an_unbound_type_variable_in_self(other, bindings).or_else(
            |_| other.try_unify_by_isolating_an_unbound_type_variable_in_self(self, bindings),
        )
    }

    /// Try to unify the following equations:
    /// - `A + rhs = other` -> `A = other - rhs`
    /// - `A - rhs = other` -> `A = other + rhs`
    /// - `lhs + B = other` -> `B = other - lhs`
    /// - `lhs - B = other` -> `B = lhs - other`
    fn try_unify_by_isolating_an_unbound_type_variable_in_self(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        if let Type::InfixExpr(lhs_lhs, lhs_op, lhs_rhs, _) = self {
            let lhs_op_inverse = lhs_op.inverse();

            // Check if it's `A + rhs = other` or `A - rhs = other`
            if let (Some(op_a_inverse), Type::TypeVariable(lhs_lhs_var)) =
                (lhs_op_inverse, lhs_lhs.as_ref())
            {
                if lhs_lhs_var.1.borrow().is_unbound() {
                    // We can say that `A = other - rhs` or `A = other + rhs` respectively
                    let new_rhs =
                        Type::infix_expr(Box::new(other.clone()), op_a_inverse, lhs_rhs.clone());

                    let mut tmp_bindings = bindings.clone();
                    if lhs_lhs.try_unify(&new_rhs, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }

            // Check if it's `lhs + B = other`
            if let (BinaryTypeOperator::Addition, Type::TypeVariable(lhs_rhs_var)) =
                (lhs_op, lhs_rhs.as_ref())
            {
                if lhs_rhs_var.1.borrow().is_unbound() {
                    // We can say that `B = other - lhs`
                    let new_rhs = Type::inverted_infix_expr(
                        Box::new(other.clone()),
                        BinaryTypeOperator::Subtraction,
                        lhs_lhs.clone(),
                    );

                    let mut tmp_bindings = bindings.clone();
                    if lhs_rhs.try_unify(&new_rhs, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }

            // Check if it's `lhs - B = other`
            if let (BinaryTypeOperator::Subtraction, Type::TypeVariable(lhs_rhs_var)) =
                (lhs_op, lhs_rhs.as_ref())
            {
                if lhs_rhs_var.1.borrow().is_unbound() {
                    // We can say that `B = lhs - other`
                    let new_rhs = Type::inverted_infix_expr(
                        lhs_lhs.clone(),
                        BinaryTypeOperator::Subtraction,
                        Box::new(other.clone()),
                    );

                    let mut tmp_bindings = bindings.clone();
                    if lhs_rhs.try_unify(&new_rhs, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }
        }

        Err(UnificationError)
    }

    /// Try to unify the following equations:
    /// - `(..a..) + 1 = (..b..)` -> `(..a..) = (..b..) - 1`
    /// - `(..a..) - 1 = (..b..)` -> `(..a..) = (..b..) + 1`
    /// - `(..a..) = (..b..) + 1` -> `(..b..) = (..a..) - 1`
    /// - `(..a..) = (..b..) - 1` -> `(..b..) = (..a..) + 1`
    fn try_unify_by_moving_single_constant_term(
        &self,
        other: &Type,
        flags: UnificationFlags,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        let result = self.try_unify_by_moving_single_constant_term_in_self(other, bindings);
        if result.is_ok() {
            return Ok(());
        }

        if flags != UnificationFlags::DoNotMoveConstantsOnTheRight {
            let result = other.try_unify_by_moving_single_constant_term_in_self(self, bindings);
            if result.is_ok() {
                return Ok(());
            }
        }

        Err(UnificationError)
    }

    /// Try to unify the following equations:
    /// - `(..a..) + 1 = (..b..)` -> `(..a..) = (..b..) - 1`
    /// - `(..a..) - 1 = (..b..)` -> `(..a..) = (..b..) + 1`
    fn try_unify_by_moving_single_constant_term_in_self(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        if let Type::InfixExpr(lhs_lhs, lhs_op, lhs_rhs, _) = self {
            if let Some(lhs_op_inverse) = lhs_op.approx_inverse() {
                let kind = lhs_lhs.infix_kind(lhs_rhs);
                let dummy_location = Location::dummy();
                let lhs_rhs = lhs_rhs.substitute(bindings);
                if let Ok(value) = lhs_rhs.evaluate_to_signed_field(&kind, dummy_location) {
                    let lhs_rhs = Box::new(Type::Constant(value, kind));
                    let new_rhs =
                        Type::inverted_infix_expr(Box::new(other.clone()), lhs_op_inverse, lhs_rhs);

                    let mut tmp_bindings = bindings.clone();

                    // Since we are going to move a constant from one side to the other, we don't want
                    // to try moving the constant back to where it was because it would lead to infinite recursion.
                    let flags = UnificationFlags::DoNotMoveConstantsOnTheRight;
                    if lhs_lhs.try_unify_with_flags(&new_rhs, flags, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }
        }

        Err(UnificationError)
    }

    /// Similar to `unify` but if the check fails this will attempt to coerce the
    /// argument to the target type. When this happens, the given expression is wrapped in
    /// a new expression to convert its type. E.g. `array` -> `array.as_slice()`
    ///
    /// Currently the only type coercion in Noir is `[T; N]` into `[T]` via `.as_slice()`.
    pub fn unify_with_coercions(
        &self,
        expected: &Type,
        expression: ExprId,
        location: Location,
        interner: &mut NodeInterner,
        errors: &mut Vec<CompilationError>,
        make_error: impl FnOnce() -> CompilationError,
    ) {
        let mut bindings = TypeBindings::default();

        if let Ok(()) = self.try_unify(expected, &mut bindings) {
            Type::apply_type_bindings(bindings);
            return;
        }

        if self.try_array_to_slice_coercion(expected, expression, interner) {
            return;
        }

        if self.try_string_to_ctstring_coercion(expected, expression, interner) {
            return;
        }

        if self.try_reference_coercion(expected) {
            return;
        }

        // Try to coerce `fn (..) -> T` to `unconstrained fn (..) -> T`
        match self.try_fn_to_unconstrained_fn_coercion(expected) {
            FunctionCoercionResult::NoCoercion => errors.push(make_error()),
            FunctionCoercionResult::Coerced(coerced_self) => {
                coerced_self.unify_with_coercions(
                    expected, expression, location, interner, errors, make_error,
                );
            }
            FunctionCoercionResult::UnconstrainedMismatch(coerced_self) => {
                errors.push(CompilationError::TypeError(TypeCheckError::UnsafeFn { location }));

                coerced_self.unify_with_coercions(
                    expected, expression, location, interner, errors, make_error,
                );
            }
        }
    }

    // If `self` and `expected` are function types, tries to coerce `self` to `expected`.
    // Returns None if no coercion can be applied, otherwise returns `self` coerced to `expected`.
    fn try_fn_to_unconstrained_fn_coercion(&self, expected: &Type) -> FunctionCoercionResult {
        // If `self` and `expected` are function types, `self` can be coerced to `expected`
        // if `self` is unconstrained and `expected` is not. The other way around is an error, though.
        if let (
            Type::Function(params, ret, env, unconstrained_self),
            Type::Function(_, _, _, unconstrained_expected),
        ) = (self.follow_bindings(), expected.follow_bindings())
        {
            let coerced_type = Type::Function(params, ret, env, unconstrained_expected);

            match (unconstrained_self, unconstrained_expected) {
                (true, true) | (false, false) => FunctionCoercionResult::NoCoercion,
                (false, true) => FunctionCoercionResult::Coerced(coerced_type),
                (true, false) => FunctionCoercionResult::UnconstrainedMismatch(coerced_type),
            }
        } else {
            FunctionCoercionResult::NoCoercion
        }
    }

    /// Try to apply the array to slice coercion to this given type pair and expression.
    /// If self can be converted to target this way, do so and return true to indicate success.
    fn try_array_to_slice_coercion(
        &self,
        target: &Type,
        expression: ExprId,
        interner: &mut NodeInterner,
    ) -> bool {
        let this = self.follow_bindings();
        let target = target.follow_bindings();

        if let (Type::Array(_size, element1), Type::Slice(element2)) = (&this, &target) {
            // We can only do the coercion if the `as_slice` method exists.
            // This is usually true, but some tests don't have access to the standard library.
            if let Some(as_slice) = interner.lookup_direct_method(&this, "as_slice", true) {
                // Still have to ensure the element types match.
                // Don't need to issue an error here if not, it will be done in unify_with_coercions
                let mut bindings = TypeBindings::default();
                if element1.try_unify(element2, &mut bindings).is_ok() {
                    invoke_function_on_expression(expression, this, target, as_slice, interner);
                    Self::apply_type_bindings(bindings);
                    return true;
                }
            }
        }
        false
    }

    fn try_string_to_ctstring_coercion(
        &self,
        target: &Type,
        expression: ExprId,
        interner: &mut NodeInterner,
    ) -> bool {
        let this = self.follow_bindings();
        let target = target.follow_bindings();

        let Type::Quoted(QuotedType::CtString) = &target else {
            return false;
        };

        match &this {
            Type::String(..) | Type::FmtString(..) => {
                // as_ctstring is defined as a trait method
                for (func_id, trait_id) in interner.lookup_trait_methods(&this, "as_ctstring", true)
                {
                    // Look up the one that's in the standard library.
                    let trait_ = interner.get_trait(trait_id);
                    if trait_.crate_id.is_stdlib() && trait_.name.as_str() == "AsCtString" {
                        invoke_function_on_expression(expression, this, target, func_id, interner);
                        return true;
                    }
                }
            }
            _ => (),
        }

        false
    }

    /// Attempt to coerce `&mut T` to `&T`, returning true if this is possible.
    pub(crate) fn try_reference_coercion(&self, target: &Type) -> bool {
        let this = self.follow_bindings();
        let target = target.follow_bindings();

        if let (Type::Reference(this_elem, true), Type::Reference(target_elem, false)) =
            (&this, &target)
        {
            // Still have to ensure the element types match.
            // Don't need to issue an error here if not, it will be done in unify_with_coercions
            let mut bindings = TypeBindings::default();
            if this_elem.try_unify(target_elem, &mut bindings).is_ok() {
                Self::apply_type_bindings(bindings);
                return true;
            }
        }
        false
    }
}

/// Wraps a given `expression` in `expression.method()`
fn invoke_function_on_expression(
    expression: ExprId,
    expression_type: Type,
    target_type: Type,
    method: crate::node_interner::FuncId,
    interner: &mut NodeInterner,
) {
    let method_id = interner.function_definition_id(method);
    let location = interner.expr_location(&expression);
    let as_slice = HirExpression::Ident(HirIdent::non_trait_method(method_id, location), None);
    let func_type = Type::Function(
        vec![expression_type.clone()],
        Box::new(target_type.clone()),
        Box::new(Type::Unit),
        false,
    );
    let func = interner.push_expr_full(as_slice, location, func_type);

    // Copy the expression and give it a new ExprId. The old one
    // will be mutated in place into a Call expression.
    let argument = interner.expression(&expression);
    let argument = interner.push_expr_full(argument, location, expression_type);

    let arguments = vec![argument];
    let is_macro_call = false;
    let call = HirExpression::Call(HirCallExpression { func, arguments, location, is_macro_call });
    interner.replace_expr(&expression, call);
    interner.push_expr_type(expression, target_type);
}

#[cfg(test)]
mod tests {
    use crate::{BinaryTypeOperator, Kind, Type, TypeBindings, TypeVariable, TypeVariableId};

    struct Types {
        next_type_variable_id: usize,
    }

    impl Types {
        fn new() -> Self {
            Self { next_type_variable_id: 0 }
        }

        fn type_variable(&mut self) -> (Type, TypeVariableId) {
            self.type_variable_with_kind(Kind::Any)
        }

        fn type_variable_with_kind(&mut self, kind: Kind) -> (Type, TypeVariableId) {
            let id = TypeVariableId(self.next_type_variable_id);
            self.next_type_variable_id += 1;
            (Type::TypeVariable(TypeVariable::unbound(id, kind)), id)
        }
    }

    fn constant(value: u128) -> Type {
        Type::Constant(value.into(), Kind::Any)
    }

    fn add(a: &Type, b: &Type) -> Type {
        binary(a, BinaryTypeOperator::Addition, b)
    }

    fn subtract(a: &Type, b: &Type) -> Type {
        binary(a, BinaryTypeOperator::Subtraction, b)
    }

    fn multiply(a: &Type, b: &Type) -> Type {
        binary(a, BinaryTypeOperator::Multiplication, b)
    }

    fn divide(a: &Type, b: &Type) -> Type {
        binary(a, BinaryTypeOperator::Division, b)
    }

    fn binary(a: &Type, op: BinaryTypeOperator, b: &Type) -> Type {
        Type::infix_expr(Box::new(a.clone()), op, Box::new(b.clone()))
    }

    #[test]
    fn unifies_two_type_variables() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // A = B
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        assert!(a.try_unify(&b, &mut bindings).is_ok());

        // A = B
        assert_eq!(bindings[&id_a].2, b);
    }

    #[test]
    fn unifies_addition_1() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // A + B = 1
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let one = constant(1);

        let addition = add(&a, &b);
        assert!(addition.try_unify(&one, &mut bindings).is_ok());

        // A = 1 - B
        assert_eq!(bindings[&id_a].2, subtract(&one, &b));
    }

    #[test]
    fn unifies_addition_2() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // A + B = C * D
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let (c, _) = types.type_variable();
        let (d, _) = types.type_variable();

        let left = add(&a, &b);
        let right = multiply(&c, &d);
        assert!(left.try_unify(&right, &mut bindings).is_ok());

        // A = (C * D) - B
        assert_eq!(bindings[&id_a].2, subtract(&right, &b));
    }

    #[test]
    fn unifies_subtraction_1() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // A - B = 1
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let subtraction = subtract(&a, &b);
        let one = constant(1);
        assert!(subtraction.try_unify(&one, &mut bindings).is_ok());

        // A = B + 1
        assert_eq!(bindings[&id_a].2, add(&b, &one));
    }

    #[test]
    fn unifies_subtraction_2() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // 1 - A = B * C
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let (c, _) = types.type_variable();
        let one = constant(1);

        let left = subtract(&one, &a);
        let right = multiply(&b, &c);
        assert!(left.try_unify(&right, &mut bindings).is_ok());

        // A = 1 - (B * C)
        assert_eq!(bindings[&id_a].2, subtract(&one, &right));
    }

    #[test]
    fn unifies_constant_added_in_both_sides() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // A + 1 = B + 3
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let one = constant(1);
        let two = constant(2);
        let three = constant(3);

        let left = add(&a, &one);
        let right = add(&b, &three);
        assert!(left.try_unify(&right, &mut bindings).is_ok());

        // A = B + 2
        assert_eq!(bindings[&id_a].2, add(&b, &two));
    }

    #[test]
    fn unifies_infix_subtraction_against_multiplication() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // (3 - A) - 1 = B * C
        let (a, id_a) = types.type_variable();
        let (b, _) = types.type_variable();
        let (c, _) = types.type_variable();
        let one = constant(1);
        let three = constant(3);

        let left = subtract(&subtract(&three, &a), &one);
        let right = multiply(&b, &c);
        assert!(left.try_unify(&right, &mut bindings).is_ok());

        // A = 3 - ((B * C) + 1)
        assert_eq!(bindings[&id_a].2, subtract(&three, &add(&right, &one)));
    }

    #[test]
    fn does_not_recurse_forever_when_moving_single_constant_terms() {
        let mut types = Types::new();
        let mut bindings = TypeBindings::default();

        // (A / B) - 1 = C * D
        let (a, _) = types.type_variable();
        let (b, _) = types.type_variable();
        let (c, _) = types.type_variable();
        let (d, _) = types.type_variable();
        let one = constant(1);

        let left = subtract(&divide(&a, &b), &one);
        let right = multiply(&c, &d);

        // This shouldn't unify. The idea is the compiler will try to do this:
        //
        // 1. (A / B) - 1 = C * D
        // 2. (A / B) = (C * D) + 1
        //
        // It can't solve either A or B, so it will try to move the `1` back to
        // the right side... except that we prevent that recursion.
        assert!(left.try_unify(&right, &mut bindings).is_err());
    }
}
