use std::collections::BTreeMap;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{FunctionKind, TraitItem, UnresolvedGenerics, UnresolvedTraitConstraint},
    hir::{
        def_collector::dc_crate::UnresolvedTrait, def_map::ModuleId,
        resolution::path_resolver::StandardPathResolver,
    },
    hir_def::{
        function::{FuncMeta, HirFunction},
        traits::{TraitConstant, TraitFunction, TraitType},
    },
    macros_api::{
        BlockExpression, FunctionDefinition, FunctionReturnType, Ident, ItemVisibility,
        NoirFunction, Param, Pattern, UnresolvedType, Visibility,
    },
    node_interner::{FuncId, TraitId},
    token::Attributes,
    Generics, Type, TypeVariable, TypeVariableKind,
};

use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub fn collect_traits(&mut self, traits: BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in &traits {
            self.interner.push_empty_trait(*trait_id, unresolved_trait);
        }

        for (trait_id, unresolved_trait) in traits {
            let generics = vecmap(&unresolved_trait.trait_def.generics, |_| {
                TypeVariable::unbound(self.interner.next_type_variable_id())
            });

            // Resolve order
            // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
            let _ = self.resolve_trait_types(&unresolved_trait);
            // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
            let _ = self.resolve_trait_constants(&unresolved_trait);
            // 3. Trait Methods
            let methods = self.resolve_trait_methods(trait_id, &unresolved_trait, &generics);

            self.interner.update_trait(trait_id, |trait_def| {
                trait_def.set_methods(methods);
                trait_def.generics = generics;
            });

            // This check needs to be after the trait's methods are set since
            // the interner may set `interner.ordering_type` based on the result type
            // of the Cmp trait, if this is it.
            if self.crate_id.is_stdlib() {
                self.interner.try_add_operator_trait(trait_id);
            }
        }
    }

    fn resolve_trait_types(&mut self, _unresolved_trait: &UnresolvedTrait) -> Vec<TraitType> {
        // TODO
        vec![]
    }

    fn resolve_trait_constants(
        &mut self,
        _unresolved_trait: &UnresolvedTrait,
    ) -> Vec<TraitConstant> {
        // TODO
        vec![]
    }

    fn resolve_trait_methods(
        &mut self,
        trait_id: TraitId,
        unresolved_trait: &UnresolvedTrait,
        trait_generics: &Generics,
    ) -> Vec<TraitFunction> {
        self.local_module = unresolved_trait.module_id;
        self.file = self.def_maps[&self.crate_id].file_id(unresolved_trait.module_id);

        let mut functions = vec![];

        for item in &unresolved_trait.trait_def.items {
            if let TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body: _,
            } = item
            {
                let old_generic_count = self.generics.len();

                let the_trait = self.interner.get_trait(trait_id);
                let self_typevar = the_trait.self_type_typevar.clone();
                let self_type = Type::TypeVariable(self_typevar.clone(), TypeVariableKind::Normal);
                let name_span = the_trait.name.span();

                self.add_generics(generics);
                self.add_existing_generics(&unresolved_trait.trait_def.generics, trait_generics);
                self.add_existing_generic("Self", name_span, self_typevar);
                self.self_type = Some(self_type.clone());

                let func_id = unresolved_trait.method_ids[&name.0.contents];
                self.resolve_trait_function(
                    name,
                    generics,
                    parameters,
                    return_type,
                    where_clause,
                    func_id,
                );

                let arguments = vecmap(parameters, |param| self.resolve_type(param.1.clone()));
                let return_type = self.resolve_type(return_type.get_type().into_owned());

                let generics = vecmap(&self.generics, |(_, type_var, _)| type_var.clone());

                let default_impl_list: Vec<_> = unresolved_trait
                    .fns_with_default_impl
                    .functions
                    .iter()
                    .filter(|(_, _, q)| q.name() == name.0.contents)
                    .collect();

                let default_impl = if default_impl_list.len() == 1 {
                    Some(Box::new(default_impl_list[0].2.clone()))
                } else {
                    None
                };

                let no_environment = Box::new(Type::Unit);
                let function_type =
                    Type::Function(arguments, Box::new(return_type), no_environment);

                functions.push(TraitFunction {
                    name: name.clone(),
                    typ: Type::Forall(generics, Box::new(function_type)),
                    location: Location::new(name.span(), unresolved_trait.file_id),
                    default_impl,
                    default_impl_module_id: unresolved_trait.module_id,
                });

                self.generics.truncate(old_generic_count);
            }
        }
        functions
    }

    pub fn resolve_trait_function(
        &mut self,
        name: &Ident,
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &FunctionReturnType,
        where_clause: &[UnresolvedTraitConstraint],
        func_id: FuncId,
    ) {
        let old_generic_count = self.generics.len();
        self.scopes.start_function();

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();

        self.trait_bounds = where_clause.to_vec();

        let kind = FunctionKind::Normal;
        let def = FunctionDefinition {
            name: name.clone(),
            attributes: Attributes::empty(),
            is_unconstrained: false,
            is_comptime: false,
            visibility: ItemVisibility::Public, // Trait functions are always public
            generics: generics.clone(),
            parameters: vecmap(parameters, |(name, typ)| Param {
                visibility: Visibility::Private,
                pattern: Pattern::Identifier(name.clone()),
                typ: typ.clone(),
                span: name.span(),
            }),
            body: BlockExpression { statements: Vec::new() },
            span: name.span(),
            where_clause: where_clause.to_vec(),
            return_type: return_type.clone(),
            return_visibility: Visibility::Private,
        };

        self.elaborate_function(NoirFunction { kind, def }, func_id);
        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.trait_bounds.clear();
        self.generics.truncate(old_generic_count);
    }
}
