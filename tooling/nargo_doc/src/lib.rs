use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_driver::CrateId;
use noirc_frontend::ast::ItemVisibility;
use noirc_frontend::hir::def_map::ModuleDefId;
use noirc_frontend::hir::printer::items as expand_items;
use noirc_frontend::hir_def::stmt::HirLetStatement;
use noirc_frontend::node_interner::ReferenceId;
use noirc_frontend::{graph::CrateGraph, hir::def_map::DefMaps, node_interner::NodeInterner};

use crate::items::{Function, Global, Item, Module, Struct, StructField, Trait, Type, TypeAlias};

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
            expand_items::Item::DataType(data_type) => {
                let type_id = data_type.id;
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
                let generics = vecmap(&data_type.generics, |generic| generic.name.to_string());

                // TODO: impls
                // TODO: trait impls
                let id = self.get_id(ModuleDefId::TypeId(type_id));
                Item::Struct(Struct {
                    id,
                    name: data_type.name.to_string(),
                    generics,
                    fields,
                    comments,
                })
            }
            expand_items::Item::Trait(trait_) => {
                let trait_id = trait_.id;
                let trait_ = self.interner.get_trait(trait_id);
                let name = trait_.name.to_string();
                let comments = self.doc_comments(ReferenceId::Trait(trait_id));
                let generics = vecmap(&trait_.generics, |generic| generic.name.to_string());

                // TODO: parents
                // TODO: where clauses
                // TODO: methods
                // TODO: trait impls
                let id = self.get_id(ModuleDefId::TraitId(trait_id));
                Item::Trait(Trait { id, name, generics, comments })
            }
            expand_items::Item::TypeAlias(type_alias_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let name = type_alias.name.to_string();
                let r#type = self.convert_type(&type_alias.typ);
                let comments = self.doc_comments(ReferenceId::Alias(type_alias_id));
                let generics = vecmap(&type_alias.generics, |generic| generic.name.to_string());
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
            expand_items::Item::Function(func_id) => {
                let modifiers = self.interner.function_modifiers(&func_id);
                let unconstrained = modifiers.is_unconstrained;
                let comptime = modifiers.is_comptime;
                let name = modifiers.name.to_string();
                let comments = self.doc_comments(ReferenceId::Function(func_id));
                // TODO: generics
                // TODO: args
                // TODO: return type
                // TODO: where clauses
                Item::Function(Function { name, comments, unconstrained, comptime })
            }
        }
    }

    fn convert_type(&self, typ: &noirc_frontend::Type) -> Type {
        Type { name: typ.to_string() }
    }

    fn doc_comments(&self, id: ReferenceId) -> Option<String> {
        self.interner.doc_comments(id).map(|comments| comments.join("\n").trim().to_string())
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
