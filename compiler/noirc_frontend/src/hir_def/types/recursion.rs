use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashSet as HashSet, FxHasher};

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

/// The types already seen by a walk, so that a type reachable by many paths is walked once.
///
/// Deeply nested generics otherwise cause a combinatorial explosion of visits to the same type.
/// Hashes are stored rather than types, with the colliding types kept to settle a collision.
#[derive(Default)]
struct VisitedTypes {
    hashes: HashSet<u64>,
    collided: HashSet<Type>,
}

impl VisitedTypes {
    /// Returns whether this is the first time `typ` is seen.
    fn insert(&mut self, typ: &Type) -> bool {
        let mut hasher = FxHasher::default();
        typ.hash(&mut hasher);
        self.hashes.insert(hasher.finish()) || self.collided.insert(typ.clone())
    }
}

/// Whether a walk reaches a data type's generic arguments.
#[derive(Clone, Copy)]
pub(crate) enum DataTypeGenerics {
    /// Reach them: `Phantom<Foo>` names `Foo` whether or not a value of it holds one.
    Named,
    /// Skip them. A generic that a field or variant is built from is reached through that field
    /// or variant, already substituted; one that none of them mentions is not in the value.
    InValue,
}

impl Type {
    /// Whether `predicate` holds for this type or for any type reachable from it, reachable in
    /// the sense of [`Self::visit_reachable`] with [`DataTypeGenerics::InValue`]: the shape of a
    /// *value* of this type.
    pub(crate) fn contains_matching(&self, mut predicate: impl FnMut(&Type) -> bool) -> bool {
        self.visit_reachable(DataTypeGenerics::InValue, &mut predicate)
    }

    /// Calls `visit` on this type and on every type reachable from it, stopping as soon as `visit`
    /// returns `true`. Returns whether it did.
    ///
    /// Reachable is the shape of a *value* of this type, so the walk expands aliases, follows
    /// bound type variables, and descends into a data type's fields or variants — a struct whose
    /// field is a vector reaches that vector. It stops at cycles, so a recursive type terminates.
    /// A data type's generic arguments are reached according to `generics`.
    ///
    /// Two positions are never reachable:
    ///
    /// * A function's parameter and return types. Only its environment is carried in a value of
    ///   the function type; arguments are passed in and the return type is returned.
    /// * A trait-as-type's generics, which describe the bound rather than the represented value.
    ///
    /// `visit` is called on a type before that type's fields, variants or aliased type are read,
    /// so a visitor that resolves a deferred body sees the resolved one on the way down.
    ///
    /// Two guards, for two different problems:
    /// * [`TypeRecursionContext`] breaks cycles and bounds depth, so a recursive type terminates.
    /// * [`VisitedTypes`] walks a type reachable by many paths once.
    pub(crate) fn visit_reachable(
        &self,
        generics: DataTypeGenerics,
        visit: &mut impl FnMut(&Type) -> bool,
    ) -> bool {
        self.visit_reachable_helper(
            visit,
            generics,
            TypeRecursionContext::default(),
            &mut VisitedTypes::default(),
        )
    }

    fn visit_reachable_helper(
        &self,
        visit: &mut impl FnMut(&Type) -> bool,
        generics_mode: DataTypeGenerics,
        mut context: TypeRecursionContext,
        visited: &mut VisitedTypes,
    ) -> bool {
        if !visited.insert(self) {
            return false;
        }

        if visit(self) {
            return true;
        }

        // Every recursive call goes through this, so that a caller cannot descend into one of the
        // positions above by forgetting which arm it was writing.
        let mut descend =
            |typ: &Type, context: TypeRecursionContext, visited: &mut VisitedTypes| {
                typ.visit_reachable_helper(visit, generics_mode, context, visited)
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
                descend(element, context.recur(), visited)
            }
            Type::Forall(_, typ) => descend(typ, context.recur(), visited),

            Type::Array(element, length) | Type::FmtString(length, element) => {
                descend(length, context.clone().recur(), visited)
                    || descend(element, context.recur(), visited)
            }
            Type::CheckedCast { from, to } => {
                descend(from, context.clone().recur(), visited)
                    || descend(to, context.recur(), visited)
            }
            Type::InfixExpr(lhs, _op, rhs, _) => {
                descend(lhs, context.clone().recur(), visited)
                    || descend(rhs, context.recur(), visited)
            }

            // Only the environment is carried in a value of a function type.
            Type::Function(_args, _ret, env, _unconstrained) => {
                descend(env, context.recur(), visited)
            }

            Type::Tuple(elements) => {
                elements.iter().any(|element| descend(element, context.clone().recur(), visited))
            }

            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                // Cloned out so that no borrow of the type variable is held while `visit` runs.
                let bound = match &*type_variable.borrow() {
                    TypeBinding::Bound(bound) => bound.clone(),
                    TypeBinding::Unbound(..) => return false,
                };
                descend(&bound, context.recur(), visited)
            }

            Type::Alias(alias, generics) => {
                if !context.insert_alias(alias.borrow().id, generics.clone()) {
                    return false;
                }
                let aliased = alias.borrow().get_type(generics);
                descend(&aliased, context.recur(), visited)
            }

            Type::DataType(definition, generics) => {
                let id = definition.borrow().id;
                if !context.insert_data_type(id, generics.clone()) {
                    return false;
                }

                if let DataTypeGenerics::Named = generics_mode
                    && generics
                        .iter()
                        .any(|generic| descend(generic, context.clone().recur(), visited))
                {
                    return true;
                }

                // Read out of the definition rather than iterated through it, so that no borrow of
                // it is held while `visit` runs on the types it is built from: a visitor is free to
                // resolve a deferred body, which needs the definition mutably.
                let fields = definition.borrow().get_fields(generics);
                if let Some(fields) = fields {
                    return fields
                        .iter()
                        .any(|(_, field, _)| descend(field, context.clone().recur(), visited));
                }

                let variants = definition.borrow().get_variants(generics);
                if let Some(variants) = variants {
                    return variants
                        .iter()
                        .flat_map(|(_, arguments)| arguments)
                        .any(|argument| descend(argument, context.clone().recur(), visited));
                }

                false
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
