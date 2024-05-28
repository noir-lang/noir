use std::collections::BTreeMap;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{FunctionKind, TraitItem, UnresolvedGenerics, UnresolvedTraitConstraint},
    hir::def_collector::dc_crate::UnresolvedTrait,
    hir_def::traits::{TraitConstant, TraitFunction, TraitType},
    macros_api::{
        BlockExpression, FunctionDefinition, FunctionReturnType, Ident, ItemVisibility,
        NoirFunction, Param, Pattern, UnresolvedType, Visibility,
    },
    node_interner::{FuncId, TraitId},
    token::Attributes,
    Type, TypeVariableKind,
};

use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub fn collect_traits(&mut self, traits: BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            self.recover_generics(|this| {
                this.add_generics(&unresolved_trait.trait_def.generics);

                // Resolve order
                // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
                let _ = this.resolve_trait_types(&unresolved_trait);
                // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
                let _ = this.resolve_trait_constants(&unresolved_trait);
                // 3. Trait Methods
                let methods = this.resolve_trait_methods(trait_id, &unresolved_trait);

                this.interner.update_trait(trait_id, |trait_def| {
                    trait_def.set_methods(methods);
                    trait_def.generics = vecmap(&this.generics, |(_, generic, _)| generic.clone());
                });
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
                self.recover_generics(|this| {
                    let the_trait = this.interner.get_trait(trait_id);
                    let self_typevar = the_trait.self_type_typevar.clone();
                    let self_type =
                        Type::TypeVariable(self_typevar.clone(), TypeVariableKind::Normal);
                    let name_span = the_trait.name.span();

                    this.add_existing_generic("Self", name_span, self_typevar);
                    this.self_type = Some(self_type.clone());

                    let func_id = unresolved_trait.method_ids[&name.0.contents];
                    this.resolve_trait_function(
                        name,
                        generics,
                        parameters,
                        return_type,
                        where_clause,
                        func_id,
                    );

                    let func_meta = this.interner.function_meta(&func_id);

                    let arguments = vecmap(&func_meta.parameters.0, |(_, typ, _)| typ.clone());
                    let return_type = func_meta.return_type().clone();

                    let generics = vecmap(&this.generics, |(_, type_var, _)| type_var.clone());

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
                });
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

        let mut function = NoirFunction { kind, def };
        self.define_function_meta(&mut function, func_id, true);
        self.elaborate_function(function, func_id);
        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.trait_bounds.clear();
        self.generics.truncate(old_generic_count);
    }
}
