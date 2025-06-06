//! This module is responsible for building a list of items that represent the final
//! code that is going to be monomorphized and turned into SSA.
//! This final code has all macros expanded and is mainly gathered from data
//! inside `NodeInterner`, modules in `DefMaps` and function bodies from `HirExpression`s.

use noirc_errors::Location;
use noirc_frontend::{
    Kind, NamedGeneric, Type,
    ast::ItemVisibility,
    hir::def_map::ModuleId,
    modules::module_def_id_to_reference_id,
    node_interner::{
        FuncId, GlobalId, ImplMethod, Methods, TraitId, TraitImplId, TypeAliasId, TypeId,
    },
};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use noirc_driver::CrateId;
use noirc_frontend::{
    ast::Ident,
    hir::def_map::{DefMaps, ModuleDefId},
    node_interner::NodeInterner,
};

pub(super) enum Item {
    Module(Module),
    DataType(DataType),
    Trait(Trait),
    TypeAlias(TypeAliasId),
    Global(GlobalId),
    Function(FuncId),
}

impl Item {
    pub(super) fn module_def_id(&self) -> ModuleDefId {
        match self {
            Item::Module(module) => ModuleDefId::ModuleId(module.id),
            Item::DataType(data_type) => ModuleDefId::TypeId(data_type.id),
            Item::Trait(trait_) => ModuleDefId::TraitId(trait_.id),
            Item::TypeAlias(type_alias_id) => ModuleDefId::TypeAliasId(*type_alias_id),
            Item::Global(global_id) => ModuleDefId::GlobalId(*global_id),
            Item::Function(func_id) => ModuleDefId::FunctionId(*func_id),
        }
    }
}

pub(super) struct Module {
    pub(super) id: ModuleId,
    pub(super) name: Option<String>,
    pub(super) is_contract: bool,
    pub(super) imports: Vec<Import>,
    pub(super) items: Vec<(ItemVisibility, Item)>,
}

pub(super) struct DataType {
    pub(super) id: TypeId,
    pub(super) impls: Vec<Impl>,
    pub(super) trait_impls: Vec<TraitImpl>,
}

pub(super) struct Trait {
    pub(super) id: TraitId,
    pub(super) methods: Vec<FuncId>,
    pub(super) trait_impls: Vec<TraitImpl>,
}

pub(super) struct Impl {
    pub(super) generics: BTreeSet<(String, Kind)>,
    pub(super) typ: Type,
    pub(super) methods: Vec<(ItemVisibility, FuncId)>,
}

pub(super) struct TraitImpl {
    pub(super) generics: BTreeSet<(String, Kind)>,
    pub(super) id: TraitImplId,
    pub(super) methods: Vec<FuncId>,
}

pub(super) struct Import {
    pub(super) name: Ident,
    pub(super) id: ModuleDefId,
    pub(super) visibility: ItemVisibility,
    pub(super) is_prelude: bool,
}

pub(super) struct ItemBuilder<'context> {
    crate_id: CrateId,
    interner: &'context NodeInterner,
    def_maps: &'context DefMaps,
    /// This set is initially created with all the trait impls in the crate.
    /// As we traverse traits, will gather trait impls associated to those traits
    /// that aren't associated to types in the current crate.
    /// As we find structs and enums, we'll gather trait impls associated to those types.
    /// Because a trait impl might be associated to multiple types, once we link a trait
    /// impl to a type we'll remove it from this set.
    trait_impls: HashSet<TraitImplId>,
}

impl<'context> ItemBuilder<'context> {
    pub(super) fn new(
        crate_id: CrateId,
        interner: &'context NodeInterner,
        def_maps: &'context DefMaps,
    ) -> Self {
        let trait_impls = interner.get_trait_implementations_in_crate(crate_id);
        Self { crate_id, interner, def_maps, trait_impls }
    }

    pub(super) fn build_module(&mut self, module_id: ModuleId) -> Item {
        let attributes = self.interner.try_module_attributes(&module_id);
        let name = attributes.map(|attributes| attributes.name.clone());
        let module_data = &self.def_maps[&self.crate_id][module_id.local_id];
        let is_contract = module_data.is_contract;

        let definitions = module_data.definitions();

        let mut definitions = definitions
            .types()
            .iter()
            .chain(definitions.values())
            .flat_map(|(_name, scope)| scope.values())
            .map(|(module_def_id, visibility, _is_prelude)| {
                let location = self.module_def_id_location(*module_def_id);
                (*module_def_id, *visibility, location)
            })
            .collect::<Vec<_>>();

        // Make sure definitions are sorted according to location so the output is more similar to the original code
        definitions.sort_by_key(|(_module_def_id, _visibility, location)| *location);

        // Gather all ModuleDefId's for definitions so we can exclude them when we'll list imports now
        let definitions_module_def_ids =
            definitions.iter().map(|(module_def_id, ..)| *module_def_id).collect::<HashSet<_>>();

        let scope = module_data.scope();
        let mut imports = scope
            .types()
            .iter()
            .chain(scope.values())
            .flat_map(|(name, scope)| scope.values().map(|value| (name.clone(), value)))
            .filter_map(|(name, (module_def_id, visibility, is_prelude))| {
                if !definitions_module_def_ids.contains(module_def_id) {
                    Some(Import {
                        name,
                        id: *module_def_id,
                        visibility: *visibility,
                        is_prelude: *is_prelude,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        imports.sort_by_key(|import| import.name.location());

        let items = definitions
            .into_iter()
            .map(|(module_def_id, visibility, _location)| {
                let structure = self.build_module_def_id(module_def_id);
                (visibility, structure)
            })
            .collect();

        Item::Module(Module { id: module_id, name, is_contract, imports, items })
    }

    fn module_def_id_location(&self, module_def_id: ModuleDefId) -> Location {
        // We already have logic to go from a ReferenceId to a location, so we use that here
        let reference_id = module_def_id_to_reference_id(module_def_id);
        self.interner.reference_location(reference_id)
    }

    fn build_module_def_id(&mut self, module_def_id: ModuleDefId) -> Item {
        match module_def_id {
            ModuleDefId::ModuleId(module_id) => self.build_module(module_id),
            ModuleDefId::TypeId(type_id) => self.build_data_type(type_id),
            ModuleDefId::TypeAliasId(type_alias_id) => Item::TypeAlias(type_alias_id),
            ModuleDefId::TraitId(trait_id) => self.build_trait(trait_id),
            ModuleDefId::GlobalId(global_id) => Item::Global(global_id),
            ModuleDefId::FunctionId(func_id) => Item::Function(func_id),
        }
    }

    fn build_data_type(&mut self, type_id: TypeId) -> Item {
        let data_type = self.interner.get_type(type_id);

        let impls = if let Some(methods) =
            self.interner.get_type_methods(&Type::DataType(data_type.clone(), vec![]))
        {
            self.build_data_type_impls(methods.values())
        } else {
            Vec::new()
        };

        let data_type = data_type.borrow();
        let trait_impls = self.build_data_type_trait_impls(&data_type);

        Item::DataType(DataType { id: type_id, impls, trait_impls })
    }

    fn build_data_type_impls<'a, 'b>(
        &'a mut self,
        methods: impl Iterator<Item = &'b Methods>,
    ) -> Vec<Impl> {
        // Gather all impl methods
        // First split methods by impl methods and trait impl methods
        let mut impl_methods = Vec::new();

        for methods in methods {
            impl_methods.extend(methods.direct.clone());
        }

        // Don't show enum variant functions
        impl_methods.retain(|method| {
            let meta = self.interner.function_meta(&method.method);
            meta.enum_variant_index.is_none()
        });

        // Split them by the impl type. For example here we'll group
        // all of `Foo<i32>` methods in one bucket, all of `Foo<Field>` in another, and
        // all of `Foo<T>` in another one.
        #[allow(clippy::mutable_key_type)]
        let mut impl_methods_by_type: BTreeMap<Type, Vec<ImplMethod>> = BTreeMap::new();
        for method in impl_methods {
            impl_methods_by_type.entry(method.typ.clone()).or_default().push(method);
        }

        impl_methods_by_type
            .into_iter()
            .map(|(typ, methods)| self.build_impl(typ, methods))
            .collect()
    }

    fn build_impl(&mut self, typ: Type, methods: Vec<ImplMethod>) -> Impl {
        let mut generics = BTreeSet::new();
        gather_named_type_vars(&typ, &mut generics);

        let mut methods = methods
            .into_iter()
            .map(|method| {
                let func_id = method.method;
                let func_meta = self.interner.function_meta(&func_id);
                let modifiers = self.interner.function_modifiers(&func_id);
                let location = func_meta.name.location;
                (modifiers.visibility, func_id, location)
            })
            .collect::<Vec<_>>();

        methods.sort_by_key(|(_, _, location)| *location);

        let methods =
            methods.into_iter().map(|(visibility, func_id, _)| (visibility, func_id)).collect();

        Impl { generics, typ, methods }
    }

    fn build_data_type_trait_impls(
        &mut self,
        data_type: &noirc_frontend::DataType,
    ) -> Vec<TraitImpl> {
        let mut trait_impls = self
            .trait_impls
            .iter()
            .filter_map(|trait_impl_id| {
                let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
                let trait_impl = trait_impl.borrow();
                if type_mentions_data_type(&trait_impl.typ, data_type) {
                    Some((*trait_impl_id, trait_impl.location))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.sort_trait_impls(&mut trait_impls);

        trait_impls.into_iter().map(|(trait_impl, _)| self.build_trait_impl(trait_impl)).collect()
    }

    /// Builds trait impls for traits, but only when those impls are
    /// for types outside of the current crate as they are likely defined next
    /// to the trait.
    fn build_trait_impls_for_trait(&mut self, trait_id: TraitId) -> Vec<TraitImpl> {
        let mut trait_impls = self
            .trait_impls
            .iter()
            .filter_map(|trait_impl_id| {
                let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
                let trait_impl = trait_impl.borrow();
                if trait_impl.trait_id == trait_id
                    && self.type_only_mention_types_outside_current_crate(&trait_impl.typ)
                {
                    Some((*trait_impl_id, trait_impl.location))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.sort_trait_impls(&mut trait_impls);

        trait_impls.into_iter().map(|(trait_impl, _)| self.build_trait_impl(trait_impl)).collect()
    }

    fn sort_trait_impls(&mut self, trait_impls: &mut [(TraitImplId, Location)]) {
        trait_impls.sort_by_key(|(trait_impl_id, location)| {
            let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
            let trait_impl = trait_impl.borrow();
            let trait_ = self.interner.get_trait(trait_impl.trait_id);
            (*location, trait_.name.to_string())
        });
    }

    fn build_trait_impl(&mut self, trait_impl_id: TraitImplId) -> TraitImpl {
        // Remove the trait impl from the set so we don't show it again
        self.trait_impls.remove(&trait_impl_id);

        let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
        let trait_impl = trait_impl.borrow();

        let mut type_var_names = BTreeSet::new();
        for generic in &trait_impl.trait_generics {
            gather_named_type_vars(generic, &mut type_var_names);
        }
        gather_named_type_vars(&trait_impl.typ, &mut type_var_names);

        TraitImpl {
            generics: type_var_names,
            id: trait_impl_id,
            methods: trait_impl.methods.clone(),
        }
    }

    fn build_trait(&mut self, trait_id: TraitId) -> Item {
        let trait_ = self.interner.get_trait(trait_id);

        let mut func_ids = trait_
            .method_ids
            .values()
            .map(|func_id| {
                let location = self.interner.function_meta(func_id).location;
                (func_id, location)
            })
            .collect::<Vec<_>>();

        // Make sure functions are shown in the same order they were defined
        func_ids.sort_by_key(|(_func_id, location)| *location);

        let methods = func_ids.into_iter().map(|(func_id, _)| *func_id).collect();
        let trait_impls = self.build_trait_impls_for_trait(trait_id);

        Item::Trait(Trait { id: trait_id, methods, trait_impls })
    }

    fn type_only_mention_types_outside_current_crate(&self, typ: &Type) -> bool {
        match typ {
            Type::Array(length, typ) => {
                self.type_only_mention_types_outside_current_crate(length)
                    && self.type_only_mention_types_outside_current_crate(typ)
            }
            Type::Slice(typ) => self.type_only_mention_types_outside_current_crate(typ),
            Type::FmtString(length, typ) => {
                self.type_only_mention_types_outside_current_crate(length)
                    && self.type_only_mention_types_outside_current_crate(typ)
            }
            Type::Tuple(types) => {
                types.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::DataType(data_type, generics) => {
                let data_type = data_type.borrow();
                data_type.id.krate() != self.crate_id
                    && generics
                        .iter()
                        .all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::Alias(_type_alias, generics) => {
                generics.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::TraitAsType(trait_id, _, generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                trait_.id.0.krate != self.crate_id
                    && generics
                        .ordered
                        .iter()
                        .all(|typ| self.type_only_mention_types_outside_current_crate(typ))
                    && generics.named.iter().all(|named_type| {
                        self.type_only_mention_types_outside_current_crate(&named_type.typ)
                    })
            }
            Type::CheckedCast { from, to: _ } => {
                self.type_only_mention_types_outside_current_crate(from)
            }
            Type::Function(args, ret, env, _) => {
                args.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
                    && self.type_only_mention_types_outside_current_crate(ret)
                    && self.type_only_mention_types_outside_current_crate(env)
            }
            Type::Reference(typ, _) => self.type_only_mention_types_outside_current_crate(typ),
            Type::Forall(_, typ) => self.type_only_mention_types_outside_current_crate(typ),
            Type::InfixExpr(lhs, _, rhs, _) => {
                self.type_only_mention_types_outside_current_crate(lhs)
                    && self.type_only_mention_types_outside_current_crate(rhs)
            }
            Type::Unit
            | Type::Bool
            | Type::Integer(..)
            | Type::FieldElement
            | Type::String(_)
            | Type::Quoted(_)
            | Type::Constant(..)
            | Type::TypeVariable(..)
            | Type::NamedGeneric(..)
            | Type::Error => true,
        }
    }
}

fn gather_named_type_vars(typ: &Type, type_vars: &mut BTreeSet<(String, Kind)>) {
    match typ {
        Type::Array(length, typ) => {
            gather_named_type_vars(length, type_vars);
            gather_named_type_vars(typ, type_vars);
        }
        Type::Slice(typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::FmtString(length, typ) => {
            gather_named_type_vars(length, type_vars);
            gather_named_type_vars(typ, type_vars);
        }
        Type::Tuple(types) => {
            for typ in types {
                gather_named_type_vars(typ, type_vars);
            }
        }
        Type::DataType(_, generics) | Type::Alias(_, generics) => {
            for typ in generics {
                gather_named_type_vars(typ, type_vars);
            }
        }
        Type::TraitAsType(_, _, trait_generics) => {
            for typ in &trait_generics.ordered {
                gather_named_type_vars(typ, type_vars);
            }
            for named_type in &trait_generics.named {
                gather_named_type_vars(&named_type.typ, type_vars);
            }
        }
        Type::NamedGeneric(NamedGeneric { type_var, name, .. }) => {
            type_vars.insert((name.to_string(), type_var.kind()));
        }
        Type::CheckedCast { from, to: _ } => {
            gather_named_type_vars(from, type_vars);
        }
        Type::Function(args, ret, env, _) => {
            for typ in args {
                gather_named_type_vars(typ, type_vars);
            }
            gather_named_type_vars(ret, type_vars);
            gather_named_type_vars(env, type_vars);
        }
        Type::Reference(typ, _) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::Forall(_, typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::InfixExpr(lhs, _, rhs, _) => {
            gather_named_type_vars(lhs, type_vars);
            gather_named_type_vars(rhs, type_vars);
        }
        Type::String(typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::Unit
        | Type::FieldElement
        | Type::Integer(..)
        | Type::Bool
        | Type::Quoted(_)
        | Type::Constant(..)
        | Type::TypeVariable(_)
        | Type::Error => (),
    }
}

fn type_mentions_data_type(typ: &Type, data_type: &noirc_frontend::DataType) -> bool {
    match typ {
        Type::Array(length, typ) => {
            type_mentions_data_type(length, data_type) || type_mentions_data_type(typ, data_type)
        }
        Type::Slice(typ) => type_mentions_data_type(typ, data_type),
        Type::FmtString(length, typ) => {
            type_mentions_data_type(length, data_type) || type_mentions_data_type(typ, data_type)
        }
        Type::Tuple(types) => types.iter().any(|typ| type_mentions_data_type(typ, data_type)),
        Type::DataType(other_data_type, generics) => {
            let other_data_type = other_data_type.borrow();
            data_type.id == other_data_type.id
                || generics.iter().any(|typ| type_mentions_data_type(typ, data_type))
        }
        Type::Alias(_type_alias, generics) => {
            generics.iter().any(|typ| type_mentions_data_type(typ, data_type))
        }
        Type::TraitAsType(_, _, generics) => {
            generics.ordered.iter().any(|typ| type_mentions_data_type(typ, data_type))
                || generics
                    .named
                    .iter()
                    .any(|named_type| type_mentions_data_type(&named_type.typ, data_type))
        }
        Type::CheckedCast { from: _, to } => type_mentions_data_type(to, data_type),
        Type::Function(args, ret, env, _) => {
            args.iter().any(|typ| type_mentions_data_type(typ, data_type))
                || type_mentions_data_type(ret, data_type)
                || type_mentions_data_type(env, data_type)
        }
        Type::Reference(typ, _) => type_mentions_data_type(typ, data_type),
        Type::Forall(_, typ) => type_mentions_data_type(typ, data_type),
        Type::InfixExpr(lhs, _, rhs, _) => {
            type_mentions_data_type(lhs, data_type) || type_mentions_data_type(rhs, data_type)
        }
        Type::Unit
        | Type::Bool
        | Type::Integer(..)
        | Type::FieldElement
        | Type::String(_)
        | Type::Quoted(_)
        | Type::Constant(..)
        | Type::TypeVariable(..)
        | Type::NamedGeneric(..)
        | Type::Error => false,
    }
}
