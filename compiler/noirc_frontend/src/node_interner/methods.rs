use crate::{
    Type, TypeBindings,
    node_interner::{FuncId, TraitId},
};

use super::NodeInterner;

#[derive(Debug, Clone)]
pub struct ImplMethod {
    pub typ: Type,
    pub method: FuncId,
}

#[derive(Debug, Clone)]
pub struct TraitImplMethod {
    pub typ: Type,
    pub method: FuncId,
    pub trait_id: TraitId,
}

/// Represents the methods on a given type that each share the same name.
///
/// Methods are split into inherent methods and trait methods. If there is
/// ever a name that is defined on both a type directly, and defined indirectly
/// via a trait impl, the direct (inherent) name will always take precedence.
///
/// Additionally, types can define specialized impls with methods of the same name
/// as long as these specialized impls do not overlap. E.g. `impl Struct<u32>` and `impl Struct<u64>`
#[derive(Default, Debug, Clone)]
pub struct Methods {
    pub direct: Vec<ImplMethod>,
    pub trait_impl_methods: Vec<TraitImplMethod>,
}

impl Methods {
    /// Adds a method to this collection, without checking for overlaps.
    pub(super) fn add_method(&mut self, method: FuncId, typ: Type, trait_id: Option<TraitId>) {
        if let Some(trait_id) = trait_id {
            let trait_impl_method = TraitImplMethod { typ, method, trait_id };
            self.trait_impl_methods.push(trait_impl_method);
        } else {
            let impl_method = ImplMethod { typ, method };
            self.direct.push(impl_method);
        }
    }

    /// Finds an existing direct (inherent) method whose type overlaps with the given type.
    /// Returns `Some((method_id, method_type))` if an overlap is found.
    ///
    /// Two types overlap if there exist concrete types that could match both.
    /// For example:
    /// - `Foo<T>` and `Foo<U>` overlap
    /// - `Foo<T>` and `Foo<i32>` overlap (T can be i32)
    /// - `Foo<i32>` and `Foo<u64>` don't overlap
    pub(super) fn find_overlapping_method(
        &self,
        method_id: &FuncId,
        typ: &Type,
        interner: &NodeInterner,
    ) -> Option<(FuncId, Type)> {
        if self.direct.is_empty() {
            return None;
        }
        let instantiate_typ = interner.function_meta(method_id).instantiate(typ, interner);
        for existing in &self.direct {
            // Check if two types overlap, by instantiating both types (replacing NamedGenerics
            // with fresh TypeVariables) and then checking if they can unify.
            let existing_func_meta = interner.function_meta(&existing.method);
            let instantiate_existing = existing_func_meta.instantiate(&existing.typ, interner);
            let mut bindings = TypeBindings::default();
            let types_can_unify =
                instantiate_existing.try_unify(&instantiate_typ, &mut bindings).is_ok();
            if types_can_unify {
                return Some((existing.method, existing.typ.clone()));
            }
        }
        None
    }

    pub(super) fn find_direct_method(
        &self,
        typ: &Type,
        check_self_param: bool,
        interner: &NodeInterner,
    ) -> Option<FuncId> {
        for method in &self.direct {
            if Self::method_matches(typ, check_self_param, method.method, &method.typ, interner) {
                return Some(method.method);
            }
        }

        None
    }

    /// Returns the self types of every direct (inherent) method that matches `typ`.
    ///
    /// Unlike keying by `typ`'s method key, this matches each impl's self type against `typ`, so it
    /// distinguishes `Foo<i32>` and `Foo<u64>` (both match `Foo<_>`) from `u8` and `u16` (only `u8`
    /// matches `u8`) — even though same-key types like the integers share a single `Methods` entry.
    /// Used to detect when `TypeName::method` is ambiguous (more than one applies).
    pub(super) fn matching_direct_method_types(
        &self,
        typ: &Type,
        check_self_param: bool,
        interner: &NodeInterner,
    ) -> Vec<Type> {
        self.direct
            .iter()
            .filter(|method| {
                Self::method_matches(typ, check_self_param, method.method, &method.typ, interner)
            })
            .map(|method| method.typ.clone())
            .collect()
    }

    pub(super) fn find_trait_methods(
        &self,
        typ: &Type,
        has_self_param: bool,
        interner: &NodeInterner,
    ) -> Vec<(FuncId, TraitId, Type)> {
        let mut results = Vec::new();

        for trait_impl_method in &self.trait_impl_methods {
            let method = trait_impl_method.method;
            let method_type = &trait_impl_method.typ;
            let trait_id = trait_impl_method.trait_id;

            if Self::method_matches(typ, has_self_param, method, method_type, interner) {
                results.push((method, trait_id, method_type.clone()));
            }
        }

        results
    }

    pub fn find_matching_methods<'a>(
        &'a self,
        typ: &'a Type,
        has_self_param: bool,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = (FuncId, Option<TraitId>)> + 'a {
        self.iter().filter_map(move |(method, method_type, trait_id)| {
            if Self::method_matches(typ, has_self_param, method, method_type, interner) {
                Some((method, trait_id))
            } else {
                None
            }
        })
    }

    /// Iterate through each method, starting with the direct methods
    fn iter(&self) -> impl Iterator<Item = (FuncId, &Type, Option<TraitId>)> {
        let trait_impl_methods =
            self.trait_impl_methods.iter().map(|m| (m.method, &m.typ, Some(m.trait_id)));
        let direct = self.direct.iter().map(|method| (method.method, &method.typ, None));
        direct.chain(trait_impl_methods)
    }

    fn method_matches(
        typ: &Type,
        check_self_param: bool,
        method: FuncId,
        method_type: &Type,
        interner: &NodeInterner,
    ) -> bool {
        let func_meta = interner.function_meta(&method);
        let function_typ = &func_meta.typ;
        match function_typ.instantiate(interner).0 {
            Type::Function(args, _, _, _) => {
                if check_self_param {
                    if let Some(object) = args.first()
                        && Self::receiver_type_matches(object, typ)
                    {
                        return true;
                    }
                } else {
                    let method_type = func_meta.instantiate(method_type, interner);

                    if Self::receiver_type_matches(method_type.as_ref(), typ) {
                        return true;
                    }
                }
            }
            Type::Error => (),
            other => unreachable!("Expected function type, found {other}"),
        }

        false
    }

    fn receiver_type_matches(receiver_type: &Type, typ: &Type) -> bool {
        let mut receiver_type = receiver_type.clone();

        loop {
            if receiver_type.try_unify_with_default_bindings(typ).is_ok() {
                return true;
            }

            let followed = receiver_type.follow_bindings();
            if followed != receiver_type {
                receiver_type = followed;
                continue;
            }

            // Handle auto-dereferencing `&T` and `&mut T` into `T`, including aliases to
            // references and nested references.
            if let Type::Reference(inner, _mutable) = receiver_type {
                receiver_type = *inner;
            } else {
                return false;
            }
        }
    }
}
