use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_driver::CrateId;
use noirc_frontend::ast::ItemVisibility;
use noirc_frontend::hir::def_map::ModuleDefId;
use noirc_frontend::hir::printer::items as expand_items;
use noirc_frontend::hir_def::stmt::{HirLetStatement, HirPattern};
use noirc_frontend::node_interner::{FuncId, ReferenceId};
use noirc_frontend::{Kind, ResolvedGeneric};
use noirc_frontend::{graph::CrateGraph, hir::def_map::DefMaps, node_interner::NodeInterner};

use crate::items::{
    Function, FunctionParam, Generic, Global, Impl, Item, Module, Struct, StructField, Trait,
    TraitImpl, Type, TypeAlias,
};

pub mod items;

pub fn crate_to_item(
    crate_id: CrateId,
    _crate_graph: &CrateGraph,
    def_maps: &DefMaps,
    interner: &NodeInterner,
) -> Item {
    let item = noirc_frontend::hir::printer::crate_to_item(crate_id, def_maps, interner);
    let mut builder = DocItemBuilder::new(interner);
    builder.build_item(item)
}

struct DocItemBuilder<'a> {
    interner: &'a NodeInterner,
    ids: HashMap<ModuleDefId, usize>,
}

impl<'a> DocItemBuilder<'a> {
    fn new(interner: &'a NodeInterner) -> Self {
        Self { interner, ids: HashMap::new() }
    }

    fn build_item(&mut self, item: expand_items::Item) -> Item {
        match item {
            expand_items::Item::Module(module) => {
                let comments = self.doc_comments(ReferenceId::Module(module.id));
                let items = module
                    .items
                    .into_iter()
                    .filter(|(visibility, _item)| visibility == &ItemVisibility::Public)
                    .map(|(_, item)| self.build_item(item))
                    .collect();
                Item::Module(Module { name: module.name, comments, items })
            }
            expand_items::Item::DataType(item_data_type) => {
                let type_id = item_data_type.id;
                let shared_data_type = self.interner.get_type(type_id);
                let data_type = shared_data_type.borrow();
                if data_type.is_enum() {
                    panic!("Enums are not supported yet");
                }
                let comments = self.doc_comments(ReferenceId::Type(type_id));
                let fields = data_type
                    .get_fields_as_written()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| field.visibility == ItemVisibility::Public)
                    .map(|(index, field)| {
                        let comments = self.doc_comments(ReferenceId::StructMember(type_id, index));
                        let r#type = self.convert_type(&field.typ);
                        StructField { name: field.name.to_string(), r#type, comments }
                    })
                    .collect();
                let generics = vecmap(&data_type.generics, |generic| convert_generic(generic));
                let impls = vecmap(item_data_type.impls, |impl_| self.convert_impl(impl_));
                let trait_impls =
                    vecmap(item_data_type.trait_impls, |impl_| self.convert_trait_impl(impl_));
                let id = self.get_id(ModuleDefId::TypeId(type_id));
                Item::Struct(Struct {
                    id,
                    name: data_type.name.to_string(),
                    generics,
                    fields,
                    impls,
                    trait_impls,
                    comments,
                })
            }
            expand_items::Item::Trait(item_trait) => {
                let trait_id = item_trait.id;
                let trait_ = self.interner.get_trait(trait_id);
                let name = trait_.name.to_string();
                let comments = self.doc_comments(ReferenceId::Trait(trait_id));
                let generics = vecmap(&trait_.generics, |generic| convert_generic(generic));
                let methods = vecmap(item_trait.methods, |func_id| self.convert_function(func_id));
                let trait_impls = vecmap(item_trait.trait_impls, |trait_impl| {
                    self.convert_trait_impl(trait_impl)
                });

                // TODO: parents
                // TODO: where clauses
                let id = self.get_id(ModuleDefId::TraitId(trait_id));
                Item::Trait(Trait { id, name, generics, comments, methods, trait_impls })
            }
            expand_items::Item::TypeAlias(type_alias_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let name = type_alias.name.to_string();
                let r#type = self.convert_type(&type_alias.typ);
                let comments = self.doc_comments(ReferenceId::Alias(type_alias_id));
                let generics = vecmap(&type_alias.generics, |generic| convert_generic(generic));
                let id = self.get_id(ModuleDefId::TypeAliasId(type_alias_id));
                Item::TypeAlias(TypeAlias { id, name, comments, r#type, generics })
            }
            expand_items::Item::Global(global_id) => {
                let global_info = self.interner.get_global(global_id);
                let definition_id = global_info.definition_id;
                let definition = self.interner.definition(definition_id);
                let comptime = matches!(
                    self.interner.get_global_let_statement(global_id),
                    Some(HirLetStatement { comptime: true, .. })
                );
                let mutable = definition.mutable;
                let name = global_info.ident.to_string();
                let typ = self.interner.definition_type(definition_id);
                let r#type = self.convert_type(&typ);
                let comments = self.doc_comments(ReferenceId::Global(global_id));
                Item::Global(Global { name, comments, comptime, mutable, r#type })
            }
            expand_items::Item::Function(func_id) => Item::Function(self.convert_function(func_id)),
        }
    }

    fn convert_impl(&self, impl_: expand_items::Impl) -> Impl {
        let generics = vecmap(impl_.generics, |(name, kind)| {
            let numeric = kind_to_numeric(kind);
            Generic { name, numeric }
        });
        let r#type = self.convert_type(&impl_.typ);
        let methods = impl_
            .methods
            .into_iter()
            .filter(|(visibility, _)| visibility == &ItemVisibility::Public)
            .map(|(_, func_id)| self.convert_function(func_id))
            .collect();
        Impl { generics, r#type, methods }
    }

    fn convert_trait_impl(&mut self, item_trait_impl: expand_items::TraitImpl) -> TraitImpl {
        let generics = vecmap(item_trait_impl.generics, |(name, kind)| {
            let numeric = kind_to_numeric(kind);
            Generic { name, numeric }
        });
        let methods = vecmap(item_trait_impl.methods, |func_id| self.convert_function(func_id));

        let trait_impl_id = item_trait_impl.id;

        let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
        let trait_impl = trait_impl.borrow();
        let trait_ = self.interner.get_trait(trait_impl.trait_id);
        let trait_name = trait_.name.to_string();
        let trait_id = self.get_id(ModuleDefId::TraitId(trait_.id));
        let trait_generics = vecmap(&trait_impl.trait_generics, |typ| self.convert_type(typ));

        // TODO: where clause
        TraitImpl { generics, methods, trait_id, trait_name, trait_generics }
    }

    fn convert_type(&self, typ: &noirc_frontend::Type) -> Type {
        Type { name: typ.to_string() }
    }

    fn convert_function(&self, func_id: FuncId) -> Function {
        let modifiers = self.interner.function_modifiers(&func_id);
        let func_meta = self.interner.function_meta(&func_id);
        let unconstrained = modifiers.is_unconstrained;
        let comptime = modifiers.is_comptime;
        let name = modifiers.name.to_string();
        let comments = self.doc_comments(ReferenceId::Function(func_id));
        let generics = vecmap(&func_meta.direct_generics, |generic| convert_generic(generic));
        let params = vecmap(func_meta.parameters.iter(), |(pattern, typ, _visibility)| {
            let is_self = self.pattern_is_self(pattern);

            // `&mut self` is represented as a mutable reference type, not as a mutable pattern
            let name = if is_self && matches!(typ, noirc_frontend::Type::Reference(..)) {
                "&mut self".to_string()
            } else {
                self.pattern_to_string(pattern)
            };

            let r#type = self.convert_type(typ);
            FunctionParam { name, r#type }
        });
        let return_type = self.convert_type(func_meta.return_type());
        // TODO: where clauses
        Function { name, comments, unconstrained, comptime, generics, params, return_type }
    }

    fn doc_comments(&self, id: ReferenceId) -> Option<String> {
        self.interner.doc_comments(id).map(|comments| comments.join("\n").trim().to_string())
    }

    fn pattern_to_string(&self, pattern: &HirPattern) -> String {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                definition.name.to_string()
            }
            HirPattern::Mutable(inner_pattern, _) => self.pattern_to_string(&*inner_pattern),
            HirPattern::Tuple(..) | HirPattern::Struct(..) => "_".to_string(),
        }
    }

    fn pattern_is_self(&self, pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                definition.name == "self"
            }
            HirPattern::Mutable(pattern, _) => self.pattern_is_self(pattern),
            HirPattern::Tuple(..) | HirPattern::Struct(..) => false,
        }
    }

    fn get_id(&mut self, id: ModuleDefId) -> usize {
        if let Some(existing_id) = self.ids.get(&id) {
            *existing_id
        } else {
            let new_id = self.ids.len();
            self.ids.insert(id, new_id);
            new_id
        }
    }
}

fn convert_generic(generic: &ResolvedGeneric) -> Generic {
    let numeric = kind_to_numeric(generic.kind());
    let name = generic.name.to_string();
    Generic { name, numeric }
}

fn kind_to_numeric(kind: Kind) -> Option<String> {
    match kind {
        Kind::Any | Kind::Normal | Kind::IntegerOrField | Kind::Integer => None,
        Kind::Numeric(typ) => Some(typ.to_string()),
    }
}
