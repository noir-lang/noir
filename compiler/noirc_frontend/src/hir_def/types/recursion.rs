use crate::{
    NamedGeneric, TYPE_RECURSION_LIMIT, Type, TypeBinding,
    node_interner::{TypeAliasId, TypeId},
};

/// A type to prevent infinite recursion while traversing types recursively.
///
/// Designed to be cloned at branching (e.g. when visiting fields of tuples, or recursing),
/// so that types at the same level are counted separately. The only type recursion we want
/// to prevent is cycles.
#[derive(Clone, Default)]
pub(crate) struct TypeRecursionContext {
    depth: u32,
    data_types: imbl::HashSet<(TypeId, Vec<Type>)>,
    aliases: imbl::HashSet<(TypeAliasId, Vec<Type>)>,
}

impl TypeRecursionContext {
    /// Increases the recursion depth.
    ///
    /// Panics if it would go beyond [`TYPE_RECURSION_LIMIT`].
    pub(crate) fn recur(mut self) -> Self {
        if self.depth >= TYPE_RECURSION_LIMIT {
            panic!("Type recursion limit reached - types are too large");
        }
        self.depth += 1;
        self
    }

    /// Tracks a data type and its generics. Returns whether the data type wasn't already being tracked.
    pub(crate) fn insert_data_type(&mut self, data_type_id: TypeId, generics: Vec<Type>) -> bool {
        self.data_types.insert((data_type_id, generics)).is_none()
    }

    /// Tracks an alias and its generics. Returns whether the alias wasn't already being tracked.
    pub(crate) fn insert_alias(&mut self, alias_id: TypeAliasId, generics: Vec<Type>) -> bool {
        self.aliases.insert((alias_id, generics)).is_none()
    }
}

impl Type {
    /// Whether `predicate` holds for this type or for any type reachable from it.
    ///
    /// "Reachable" is the shape of a *value* of this type, so the walk expands aliases, follows
    /// bound type variables, and descends into a data type's fields or variants — a struct whose
    /// field is a vector contains a vector. It stops at cycles, so a recursive type terminates.
    ///
    /// Two positions are deliberately not reachable:
    ///
    /// * A function's parameter and return types. Only its environment is carried in a value of
    ///   the function type; arguments are passed in and the return type is returned.
    /// * A trait-as-type's generics, which describe the bound rather than the represented value.
    pub(crate) fn contains_matching(&self, mut predicate: impl FnMut(&Type) -> bool) -> bool {
        self.contains_matching_helper(&mut predicate, TypeRecursionContext::default())
    }

    fn contains_matching_helper(
        &self,
        predicate: &mut impl FnMut(&Type) -> bool,
        mut context: TypeRecursionContext,
    ) -> bool {
        if predicate(self) {
            return true;
        }

        // Every recursive call goes through this, so that a caller cannot descend into one of the
        // positions above by forgetting which arm it was writing.
        let mut descend = |typ: &Type, context: TypeRecursionContext| {
            typ.contains_matching_helper(predicate, context)
        };

        match self {
            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::Unit
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::TraitAsType(..)
            | Type::Error => false,

            Type::Vector(element) | Type::String(element) | Type::Reference(element, _) => {
                descend(element, context.recur())
            }
            Type::Forall(_, typ) => descend(typ, context.recur()),

            Type::Array(element, length) | Type::FmtString(length, element) => {
                descend(length, context.clone().recur()) || descend(element, context.recur())
            }
            Type::CheckedCast { from, to } => {
                descend(from, context.clone().recur()) || descend(to, context.recur())
            }
            Type::InfixExpr(lhs, _op, rhs, _) => {
                descend(lhs, context.clone().recur()) || descend(rhs, context.recur())
            }

            // Only the environment is carried in a value of a function type.
            Type::Function(_args, _ret, env, _unconstrained) => descend(env, context.recur()),

            Type::Tuple(elements) => {
                elements.iter().any(|element| descend(element, context.clone().recur()))
            }

            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                let binding = type_variable.borrow();
                match &*binding {
                    TypeBinding::Bound(bound) => descend(bound, context.recur()),
                    TypeBinding::Unbound(..) => false,
                }
            }

            Type::Alias(alias, generics) => {
                if !context.insert_alias(alias.borrow().id, generics.clone()) {
                    return false;
                }
                let aliased = alias.borrow().get_type(generics);
                descend(&aliased, context.recur())
            }

            Type::DataType(definition, generics) => {
                let definition = definition.borrow();
                if !context.insert_data_type(definition.id, generics.clone()) {
                    return false;
                }
                if let Some(fields) = definition.get_fields(generics) {
                    fields.iter().any(|(_, field, _)| descend(field, context.clone().recur()))
                } else if let Some(variants) = definition.get_variants(generics) {
                    variants
                        .iter()
                        .flat_map(|(_, arguments)| arguments)
                        .any(|argument| descend(argument, context.clone().recur()))
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Kind, Type, TypeVariable, TypeVariableId};

    fn contains_vector(typ: &Type) -> bool {
        typ.contains_matching(|typ| matches!(typ, Type::Vector(_)))
    }

    fn vector() -> Type {
        Type::Vector(Box::new(Type::FieldElement))
    }

    #[test]
    fn matches_the_type_itself() {
        assert!(contains_vector(&vector()));
        assert!(!contains_vector(&Type::FieldElement));
    }

    #[test]
    fn descends_through_containers() {
        assert!(contains_vector(&Type::Tuple(vec![Type::Bool, vector()])));
        assert!(contains_vector(&Type::Array(Box::new(Type::constant_u32(2)), Box::new(vector()))));
        assert!(contains_vector(&Type::Reference(Box::new(vector()), true)));
        assert!(!contains_vector(&Type::Tuple(vec![Type::Bool, Type::FieldElement])));
    }

    #[test]
    fn follows_bound_type_variables() {
        let type_variable = TypeVariable::unbound(TypeVariableId(0), Kind::Normal);
        let typ = Type::TypeVariable(type_variable.clone());
        assert!(!contains_vector(&typ));

        type_variable.bind(vector());
        assert!(contains_vector(&typ));
    }

    /// Only a function's environment is carried in a value of the function type: its arguments are
    /// passed in and its return type is returned, so neither is part of the value's shape.
    #[test]
    fn reads_a_function_type_as_its_environment_only() {
        let function = |args, ret, env| Type::Function(args, Box::new(ret), Box::new(env), false);

        assert!(!contains_vector(&function(vec![vector()], Type::Bool, Type::Unit)));
        assert!(!contains_vector(&function(vec![Type::Bool], vector(), Type::Unit)));
        assert!(contains_vector(&function(vec![Type::Bool], Type::Bool, vector())));
    }
}
