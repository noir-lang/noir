use std::borrow::Cow;

use noirc_errors::Span;

use crate::{Kind, Type, TypeBinding, TypeBindings, TypeVariable, UnificationError};

pub(crate) struct Unifier {
    // Temporary storage in order to get references to the types to be processed during unification
    types: Vec<Type>,
}

impl Unifier {
    pub(crate) fn new() -> Unifier {
        Unifier { types: Vec::new() }
    }

    fn add(&mut self, typ: &Type) -> usize {
        let len = self.types.len();
        self.types.push(typ.clone());
        len
    }

    // Adds types to the temporary storage and returns their index
    fn for_unite(&mut self, lhs: &Type, rhs: &Type) -> (usize, usize) {
        let lhs_id = self.add(lhs);
        let rhs_id = self.add(rhs);
        (lhs_id, rhs_id)
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    pub(crate) fn try_unify(
        lhs: &Type,
        rhs: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        let mut unifier = Unifier::new();
        unifier.unify(lhs, rhs, bindings)
    }

    /// Iterative version of type unification
    /// Unifying types may requires to unify other types which are
    /// put in a queue and processed sequentially.
    pub(crate) fn unify(
        &mut self,
        lhs: &Type,
        rhs: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        let mut to_process = vec![self.for_unite(lhs, rhs)];
        while let Some((a, b)) = to_process.pop() {
            let (a, b) = (self.types[a].clone(), self.types[b].clone());
            let to_unit = self.try_unify_single(&a, &b, bindings)?;
            to_process.extend(to_unit);
        }
        Ok(())
    }

    /// Try to unify a type variable to `self`.
    /// This is a helper function factored out from try_unify.
    fn try_unify_to_type_variable_iter(
        &mut self,
        lhs: usize,
        type_variable: &TypeVariable,
        bindings: &mut TypeBindings,

        // Bind the type variable to a type. This is factored out since depending on the
        // Kind, there are different methods to check whether the variable can
        // bind to the given type or not.
        bind_variable: impl FnOnce(&mut TypeBindings) -> Result<(), UnificationError>,
    ) -> Result<Vec<(usize, usize)>, UnificationError> {
        match &*type_variable.borrow() {
            // If it is already bound, unify against what it is bound to
            TypeBinding::Bound(link) => {
                let link_id = self.add(link);
                return Ok(vec![(link_id, lhs)]);
            }
            TypeBinding::Unbound(id, _) => {
                // We may have already "bound" this type variable in this call to
                // try_unify, so check those bindings as well.
                if let Some((_, kind, binding)) = bindings.clone().get(id) {
                    if !self.kind_unifies_iter(kind, &binding.kind()) {
                        return Err(UnificationError);
                    }
                    let bind_id = self.add(binding);
                    return Ok(vec![(bind_id, lhs)]);
                }
                // Otherwise, bind it
                bind_variable(bindings)?;
            }
        }
        Ok(Vec::new())
    }

    fn try_unify_single(
        &mut self,
        lhs: &Type,
        rhs: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<Vec<(usize, usize)>, UnificationError> {
        use Type::*;

        let lhs: Cow<Type> = match lhs {
            Type::InfixExpr(..) => Cow::Owned(lhs.canonicalize()),
            other => Cow::Borrowed(other),
        };

        let rhs: Cow<Type> = match rhs {
            Type::InfixExpr(..) => Cow::Owned(rhs.canonicalize()),
            other => Cow::Borrowed(other),
        };

        match (lhs.as_ref(), rhs.as_ref()) {
            (Error, _) | (_, Error) => Ok(Vec::new()),

            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                Ok(vec![self.for_unite(other, &alias)])
            }

            (TypeVariable(var), other) | (other, TypeVariable(var)) => {
                let other_id = self.add(other);

                match &*var.borrow() {
                    TypeBinding::Bound(typ) => {
                        if typ.is_numeric_value() {
                            self.try_unify_to_type_variable_iter(
                                other_id,
                                var,
                                bindings,
                                |bindings| {
                                    let only_integer = matches!(typ, Type::Integer(..));
                                    other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                                },
                            )
                        } else {
                            self.try_unify_to_type_variable_iter(
                                other_id,
                                var,
                                bindings,
                                |bindings| other.try_bind_to(var, bindings, typ.kind()),
                            )
                        }
                    }
                    TypeBinding::Unbound(_id, Kind::IntegerOrField) => self
                        .try_unify_to_type_variable_iter(other_id, var, bindings, |bindings| {
                            let only_integer = false;
                            other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                        }),
                    TypeBinding::Unbound(_id, Kind::Integer) => self
                        .try_unify_to_type_variable_iter(other_id, var, bindings, |bindings| {
                            let only_integer = true;
                            other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                        }),
                    TypeBinding::Unbound(_id, type_var_kind) => self
                        .try_unify_to_type_variable_iter(other_id, var, bindings, |bindings| {
                            other.try_bind_to(var, bindings, type_var_kind.clone())
                        }),
                }
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                Ok(vec![self.for_unite(len_a, len_b), self.for_unite(elem_a, elem_b)])
            }

            (Slice(elem_a), Slice(elem_b)) => Ok(vec![self.for_unite(elem_a, elem_b)]),

            (String(len_a), String(len_b)) => Ok(vec![self.for_unite(len_a, len_b)]),

            (FmtString(len_a, elements_a), FmtString(len_b, elements_b)) => {
                Ok(vec![self.for_unite(len_a, len_b), self.for_unite(elements_a, elements_b)])
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(UnificationError)
                } else {
                    let mut to_unit = Vec::new();
                    for (a, b) in elements_a.iter().zip(elements_b) {
                        to_unit.push(self.for_unite(a, b));
                    }
                    Ok(to_unit)
                }
            }

            // No recursive try_unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (Struct(id_a, args_a), Struct(id_b, args_b)) => {
                if id_a == id_b && args_a.len() == args_b.len() {
                    let mut to_unit = Vec::new();
                    for (a, b) in args_a.iter().zip(args_b) {
                        to_unit.push(self.for_unite(a, b));
                    }
                    Ok(to_unit)
                } else {
                    Err(UnificationError)
                }
            }

            (CheckedCast { to, .. }, other) | (other, CheckedCast { to, .. }) => {
                Ok(vec![self.for_unite(to, other)])
            }

            (NamedGeneric(binding, _), other) | (other, NamedGeneric(binding, _))
                if !binding.borrow().is_unbound() =>
            {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    Ok(vec![self.for_unite(link, other)])
                } else {
                    unreachable!("If guard ensures binding is bound")
                }
            }

            (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
                // Bound NamedGenerics are caught by the check above
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    self.kind_unify(&binding_a.kind(), &binding_b.kind())?;
                    Ok(vec![])
                } else {
                    Err(UnificationError)
                }
            }

            (
                Function(params_a, ret_a, env_a, unconstrained_a),
                Function(params_b, ret_b, env_b, unconstrained_b),
            ) => {
                if unconstrained_a == unconstrained_b && params_a.len() == params_b.len() {
                    let mut to_unit = Vec::new();
                    for (a, b) in params_a.iter().zip(params_b.iter()) {
                        to_unit.push(self.for_unite(a, b));
                    }
                    to_unit.push(self.for_unite(env_a, env_b));
                    to_unit.push(self.for_unite(ret_b, ret_a));
                    Ok(to_unit)
                } else {
                    Err(UnificationError)
                }
            }

            (MutableReference(elem_a), MutableReference(elem_b)) => {
                Ok(vec![self.for_unite(elem_a, elem_b)])
            }

            (InfixExpr(lhs_a, op_a, rhs_a), InfixExpr(lhs_b, op_b, rhs_b)) => {
                if op_a == op_b {
                    // We need to preserve the original bindings since if syntactic equality
                    // fails we fall back to other equality strategies.
                    let mut new_bindings = bindings.clone();
                    let lhs_result = Unifier::try_unify(lhs_a, lhs_b, &mut new_bindings);
                    let rhs_result = Unifier::try_unify(rhs_a, rhs_b, &mut new_bindings);

                    if lhs_result.is_ok() && rhs_result.is_ok() {
                        *bindings = new_bindings;
                        Ok(Vec::new())
                    } else {
                        lhs.try_unify_by_moving_constant_terms(&rhs, bindings)?;
                        Ok(Vec::new())
                    }
                } else {
                    Err(UnificationError)
                }
            }
            (Constant(value, kind), other) | (other, Constant(value, kind)) => {
                let dummy_span = Span::default();
                if let Ok(other_value) = other.evaluate_to_field_element(kind, dummy_span) {
                    if *value == other_value && self.kind_unifies_iter(kind, &other.kind()) {
                        Ok(Vec::new())
                    } else {
                        Err(UnificationError)
                    }
                } else if let InfixExpr(lhs, op, rhs) = other {
                    if let Some(inverse) = op.approx_inverse() {
                        // Handle cases like `4 = a + b` by trying to solve to `a = 4 - b`
                        let new_type = InfixExpr(
                            Box::new(Constant(*value, kind.clone())),
                            inverse,
                            rhs.clone(),
                        );
                        Ok(vec![self.for_unite(&new_type, lhs)])
                    } else {
                        Err(UnificationError)
                    }
                } else {
                    Err(UnificationError)
                }
            }

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(Vec::new())
                } else {
                    Err(UnificationError)
                }
            }
        }
    }

    pub(crate) fn kind_unifies_iter(&mut self, lhs: &Kind, other: &Kind) -> bool {
        match (lhs, other) {
            // Kind::Any unifies with everything
            (Kind::Any, _) | (_, Kind::Any) => true,

            // Kind::Normal unifies with Kind::Integer and Kind::IntegerOrField
            (Kind::Normal, Kind::Integer | Kind::IntegerOrField)
            | (Kind::Integer | Kind::IntegerOrField, Kind::Normal) => true,

            // Kind::Integer unifies with Kind::IntegerOrField
            (Kind::Integer | Kind::IntegerOrField, Kind::Integer | Kind::IntegerOrField) => true,

            // Kind::Numeric unifies along its Type argument
            (Kind::Numeric(lhs), Kind::Numeric(rhs)) => {
                let mut bindings = TypeBindings::new();
                let unifies = self.unify(lhs, rhs, &mut bindings).is_ok();
                // let unifies = lhs.try_unify_iter(rhs, &mut bindings).is_ok();
                if unifies {
                    Type::apply_type_bindings(bindings);
                }
                unifies
            }

            // everything unifies with itself
            (lhs, rhs) => lhs == rhs,
        }
    }

    pub(crate) fn kind_unify(&mut self, lhs: &Kind, other: &Kind) -> Result<(), UnificationError> {
        if self.kind_unifies_iter(lhs, other) {
            Ok(())
        } else {
            Err(UnificationError)
        }
    }
}
