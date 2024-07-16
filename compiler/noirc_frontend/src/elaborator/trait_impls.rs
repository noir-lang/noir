use std::collections::HashMap as TypeBindingsMap;

use crate::{
    graph::CrateId,
    hir::def_collector::{dc_crate::UnresolvedTraitImpl, errors::DefCollectorErrorKind},
};
use crate::{
    hir::def_collector::errors::DuplicateType,
    hir_def::{
        traits::{TraitConstraint, TraitFunction},
        types::Generics,
    },
    node_interner::{FuncId, TraitId},
    Type, TypeBindings, TypeVariable, TypeVariableId,
};

use noirc_errors::Location;
use rustc_hash::FxHashSet as HashSet;

use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub(super) fn collect_trait_impl_methods(
        &mut self,
        trait_id: TraitId,
        trait_impl: &mut UnresolvedTraitImpl,
        trait_impl_where_clause: &[TraitConstraint],
    ) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;

        // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
        // for a particular method, the default implementation will be added at that slot.
        let mut ordered_methods = Vec::new();

        // check whether the trait implementation is in the same crate as either the trait or the type
        self.check_trait_impl_crate_coherence(trait_id, trait_impl);

        // set of function ids that have a corresponding method in the trait
        let mut func_ids_in_trait = HashSet::default();

        // Temporarily take ownership of the trait's methods so we can iterate over them
        // while also mutating the interner
        let the_trait = self.interner.get_trait_mut(trait_id);
        let methods = std::mem::take(&mut the_trait.methods);
        for method in &methods {
            let overrides: Vec<_> = trait_impl
                .methods
                .functions
                .iter()
                .filter(|(_, _, f)| f.name() == method.name.0.contents)
                .collect();

            if overrides.is_empty() {
                if let Some(default_impl) = &method.default_impl {
                    // copy 'where' clause from unresolved trait impl
                    let mut default_impl_clone = default_impl.clone();
                    default_impl_clone.def.where_clause.extend(trait_impl.where_clause.clone());

                    let func_id = self.interner.push_empty_fn();
                    let module = self.module_id();
                    let location = Location::new(default_impl.def.span, trait_impl.file_id);
                    self.interner.push_function(func_id, &default_impl.def, module, location);
                    self.define_function_meta(&mut default_impl_clone, func_id, false);
                    func_ids_in_trait.insert(func_id);
                    ordered_methods.push((
                        method.default_impl_module_id,
                        func_id,
                        *default_impl_clone,
                    ));
                } else {
                    self.push_err(DefCollectorErrorKind::TraitMissingMethod {
                        trait_name: self.interner.get_trait(trait_id).name.clone(),
                        method_name: method.name.clone(),
                        trait_impl_span: trait_impl
                            .object_type
                            .span
                            .expect("type must have a span"),
                    });
                }
            } else {
                for (_, func_id, _) in &overrides {
                    self.check_where_clause_against_trait(
                        func_id,
                        method,
                        trait_impl_where_clause,
                        trait_impl.resolved_generics.len(),
                        trait_impl.resolved_trait_generics.len(),
                    );

                    func_ids_in_trait.insert(*func_id);
                }

                if overrides.len() > 1 {
                    self.push_err(DefCollectorErrorKind::Duplicate {
                        typ: DuplicateType::TraitAssociatedFunction,
                        first_def: overrides[0].2.name_ident().clone(),
                        second_def: overrides[1].2.name_ident().clone(),
                    });
                }

                ordered_methods.push(overrides[0].clone());
            }
        }

        // Restore the methods that were taken before the for loop
        let the_trait = self.interner.get_trait_mut(trait_id);
        the_trait.set_methods(methods);

        // Emit MethodNotInTrait error for methods in the impl block that
        // don't have a corresponding method signature defined in the trait
        for (_, func_id, func) in &trait_impl.methods.functions {
            if !func_ids_in_trait.contains(func_id) {
                let trait_name = the_trait.name.clone();
                let impl_method = func.name_ident().clone();
                let error = DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method };
                self.errors.push((error.into(), self.file));
            }
        }

        trait_impl.methods.functions = ordered_methods;
        trait_impl.methods.trait_id = Some(trait_id);
    }

    /// Issue an error if the impl is stricter than the trait.
    ///
    /// # Example
    ///
    /// ```
    /// trait MyTrait { }
    /// trait Foo<T> {
    ///     fn foo<U>();
    /// }
    /// impl<A> Foo<A> for () {
    ///     // Error issued here as `foo` does not have the `MyTrait` constraint
    ///     fn foo<B>() where B: MyTrait {}
    /// }
    /// ```
    fn check_where_clause_against_trait(
        &mut self,
        func_id: &FuncId,
        method: &TraitFunction,
        trait_impl_where_clause: &[TraitConstraint],
        trait_impl_generics_len: usize,
        trait_generics_len: usize,
    ) {
        let func_meta = self.interner.function_meta(func_id);
        let override_generics = func_meta.all_generics.clone();
        let method_generics: Vec<_> = method
            .all_generics
            .clone()
            .into_iter()
            .filter(|generic| generic.name.as_str() != "Self")
            .collect();

        let mut substituted_method_ids = HashSet::default();
        for method_constraint in method.trait_constraints.iter() {
            let substituted_constraint_type = Self::reset_generics_on_constraint_type(
                &method_constraint.typ,
                &method_generics,
                &method_generics,
                0,
                0,
            );
            substituted_method_ids
                .insert((substituted_constraint_type, method_constraint.trait_id));
        }

        for override_trait_constraint in func_meta.trait_constraints.clone() {
            let override_constraint_is_from_impl =
                trait_impl_where_clause.iter().any(|impl_constraint| {
                    impl_constraint.trait_id == override_trait_constraint.trait_id
                });
            if override_constraint_is_from_impl {
                continue;
            }

            let substituted_constraint_type = Self::reset_generics_on_constraint_type(
                &override_trait_constraint.typ,
                &override_generics,
                &method_generics,
                trait_impl_generics_len,
                trait_generics_len,
            );

            if !substituted_method_ids.contains(&(
                substituted_constraint_type.clone(),
                override_trait_constraint.trait_id,
            )) {
                let the_trait = self.interner.get_trait(override_trait_constraint.trait_id);
                self.push_err(DefCollectorErrorKind::ImplIsStricterThanTrait {
                    constraint_typ: override_trait_constraint.typ,
                    constraint_name: the_trait.name.0.contents.clone(),
                    constraint_span: override_trait_constraint.span,
                    trait_method_name: method.name.0.contents.clone(),
                    trait_method_span: method.location.span,
                });
            }
        }
    }

    /// Resets the generics of a trait impl's trait constraint type.
    /// This helps us match the trait constraint type from a trait itself and its trait impl.
    /// If a type contains named generics, this method will reset the type variable ids
    /// of those generics based off its position in the actual trait definition's generics.
    ///
    /// Example
    ///
    /// ```
    /// trait MyTrait { }
    /// trait Foo<T> {
    ///     fn foo_good<U>() where T: MyTrait;
    ///
    ///     fn foo_bad<U>() where T: MyTrait;
    /// }
    /// impl<A> Foo<A> for () {
    ///    // Error issued here as `foo` does not have the `MyTrait` constraint
    ///    fn foo_good<B>() where A: MyTrait {}
    ///
    ///     fn foo_bad<B>() where B: MyTrait {}
    /// ```
    /// `A` in `A: MyTrait` will have an actual type variable id based upon the global interner current type variable id.
    /// In the example, above we will have A -> '2. However, we want to compare again T -> '0.
    /// We can find `A`'s position among the trait generics and reset it to be T -> '0.
    ///  
    /// On the flip side, `B` in `B: MyTrait` will be reset to T -> '1.
    /// This will not match T -> '0 and we can mark that these types are unequal and our impl is stricter than the trait.
    ///
    /// The last two fields `trait_impl_generics_len` and `trait_generics_len` are only necessary to account
    /// for extra impl generics when indexing into the trait definition's generics.
    fn reset_generics_on_constraint_type(
        typ: &Type,
        override_generics: &Generics,
        method_generics: &Generics,
        trait_impl_generics_len: usize,
        trait_generics_len: usize,
    ) -> Type {
        let recur_generics_reset = |typ: &Type| {
            Self::reset_generics_on_constraint_type(
                typ,
                override_generics,
                method_generics,
                trait_impl_generics_len,
                trait_generics_len,
            )
        };

        match &typ {
            Type::NamedGeneric(type_var, name, _) => {
                let generic_index = Self::find_generic_index(name, override_generics);

                let mut bindings: TypeBindings = TypeBindingsMap::new();
                if let Some(mut method_generic_index) = generic_index {
                    // Override generics from a trait impl can possibly contain generics
                    // as part of the trait impl that are not part of the trait method generics.
                    // We need to account for this by checking against the length of the lists for trait impl generics and trait generics.
                    // It is important to note that trait impl generics are expected to also contain the trait generics,
                    // and that is why we add the trait generics list length before subtracting the trait impl generics length.
                    if (method_generic_index + trait_generics_len) < trait_impl_generics_len {
                        return typ.clone();
                    }

                    if trait_impl_generics_len > trait_generics_len {
                        method_generic_index =
                            method_generic_index + trait_generics_len - trait_impl_generics_len;
                    }

                    // To accurately match against the trait function's constraint types, we must
                    // replace the name of any generics in the override function with the respective
                    // name from the original trait.
                    // This substitution is why we also must recompute the method generic index above.
                    bindings.insert(
                        type_var.id(),
                        (
                            method_generics[method_generic_index].type_var.clone(),
                            Type::NamedGeneric(
                                TypeVariable::unbound(TypeVariableId(method_generic_index)),
                                method_generics[method_generic_index].name.clone(),
                                method_generics[method_generic_index].kind.clone(),
                            ),
                        ),
                    );
                }

                typ.substitute(&bindings)
            }
            Type::Struct(struct_type, generics) => {
                let reset_generics = generics.iter().map(recur_generics_reset).collect();
                Type::Struct(struct_type.clone(), reset_generics)
            }
            Type::Alias(type_alias, generics) => {
                let reset_generics = generics.iter().map(recur_generics_reset).collect();
                Type::Alias(type_alias.clone(), reset_generics)
            }
            Type::Array(size, element_type) => {
                let size = recur_generics_reset(size.as_ref());
                let element_type = recur_generics_reset(element_type.as_ref());
                Type::Array(Box::new(size), Box::new(element_type))
            }
            Type::Slice(element_type) => {
                let element_type = recur_generics_reset(element_type.as_ref());
                Type::Slice(Box::new(element_type))
            }
            Type::String(size) => {
                let size = recur_generics_reset(size.as_ref());
                Type::String(Box::new(size))
            }
            Type::FmtString(size, element_types) => {
                let size = recur_generics_reset(size.as_ref());
                let element_types = recur_generics_reset(element_types.as_ref());
                Type::FmtString(Box::new(size), Box::new(element_types))
            }
            Type::Tuple(types) => {
                let reset_types = types.iter().map(recur_generics_reset).collect();
                Type::Tuple(reset_types)
            }
            Type::Function(arguments, return_type, environment) => {
                let arguments = arguments.iter().map(recur_generics_reset).collect();
                let return_type = recur_generics_reset(return_type.as_ref());
                let environment = recur_generics_reset(environment.as_ref());
                Type::Function(arguments, Box::new(return_type), Box::new(environment))
            }
            Type::MutableReference(typ) => {
                let typ = recur_generics_reset(typ.as_ref());
                Type::MutableReference(Box::new(typ))
            }
            _ => typ.clone(),
        }
    }

    fn find_generic_index(target_name: &str, generics: &Generics) -> Option<usize> {
        generics.iter().position(|generic| generic.name.as_str() == target_name)
    }

    fn check_trait_impl_crate_coherence(
        &mut self,
        trait_id: TraitId,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;

        let object_crate = match &trait_impl.resolved_object_type {
            Some(Type::Struct(struct_type, _)) => struct_type.borrow().id.krate(),
            _ => CrateId::Dummy,
        };

        let the_trait = self.interner.get_trait(trait_id);
        if self.crate_id != the_trait.crate_id && self.crate_id != object_crate {
            self.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                span: trait_impl.object_type.span.expect("object type must have a span"),
            });
        }
    }
}
