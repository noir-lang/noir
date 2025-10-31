use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_driver::CrateId;
use noirc_errors::Location;
use noirc_frontend::ast::ItemVisibility;
use noirc_frontend::hir::printer::items as expand_items;
use noirc_frontend::hir_def::stmt::{HirLetStatement, HirPattern};
use noirc_frontend::hir_def::traits::ResolvedTraitBound;
use noirc_frontend::node_interner::{FuncId, ReferenceId, TraitId, TypeAliasId, TypeId};
use noirc_frontend::{Kind, NamedGeneric, ResolvedGeneric, TypeBinding};
use noirc_frontend::{graph::CrateGraph, hir::def_map::DefMaps, node_interner::NodeInterner};

use crate::items::{
    Function, FunctionParam, Generic, Global, Impl, Item, Module, Struct, StructField, Trait,
    TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias,
};

mod html;
pub mod items;
pub use html::to_html;

/// Returns the root module in a crate.
pub fn crate_module(
    crate_id: CrateId,
    _crate_graph: &CrateGraph,
    def_maps: &DefMaps,
    interner: &NodeInterner,
    ids: &mut ItemIds,
) -> Module {
    let module = noirc_frontend::hir::printer::crate_to_module(crate_id, def_maps, interner);
    let mut builder = DocItemBuilder { interner, ids };
    builder.convert_module(module)
}

struct DocItemBuilder<'a> {
    interner: &'a NodeInterner,
    ids: &'a mut ItemIds,
}

/// Maps an ItemId to a unique identifier.
pub type ItemIds = HashMap<ItemId, usize>;

/// Uniquely identifies an item.
/// This is done by using a type's name, location in source code and kind.
/// With macros, two types might end up being defined in the same location but they will likely
/// have different names.
/// This is just a temporary solution until we have a better way to uniquely identify items
/// across crates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId {
    pub location: Location,
    pub kind: IdKeyKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdKeyKind {
    Type,
    Trait,
    TypeAlias,
}

impl DocItemBuilder<'_> {
    fn convert_item(&mut self, item: expand_items::Item) -> Item {
        match item {
            expand_items::Item::Module(module) => {
                let module = self.convert_module(module);
                Item::Module(module)
            }
            expand_items::Item::DataType(item_data_type) => {
                let type_id = item_data_type.id;
                let shared_data_type = self.interner.get_type(type_id);
                let data_type = shared_data_type.borrow();
                if data_type.is_enum() {
                    panic!("Enums are not supported yet");
                }
                let comments = self.doc_comments(ReferenceId::Type(type_id));
                let mut has_private_fields = false;
                let fields = data_type
                    .get_fields_as_written()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| {
                        if field.visibility == ItemVisibility::Public {
                            true
                        } else {
                            has_private_fields = true;
                            false
                        }
                    })
                    .map(|(index, field)| {
                        let comments = self.doc_comments(ReferenceId::StructMember(type_id, index));
                        let r#type = self.convert_type(&field.typ);
                        StructField { name: field.name.to_string(), r#type, comments }
                    })
                    .collect();
                let generics = vecmap(&data_type.generics, convert_generic);
                let impls = vecmap(item_data_type.impls, |impl_| self.convert_impl(impl_));
                let trait_impls =
                    vecmap(item_data_type.trait_impls, |impl_| self.convert_trait_impl(impl_));
                let id = self.get_type_id(type_id);
                Item::Struct(Struct {
                    id,
                    name: data_type.name.to_string(),
                    generics,
                    fields,
                    has_private_fields,
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
                let generics = vecmap(&trait_.generics, convert_generic);
                let methods = vecmap(item_trait.methods, |func_id| self.convert_function(func_id));
                let trait_impls = vecmap(item_trait.trait_impls, |trait_impl| {
                    self.convert_trait_impl(trait_impl)
                });
                let where_clause = vecmap(&trait_.where_clause, |constraint| {
                    self.convert_trait_constraint(constraint)
                });
                let parents = vecmap(&trait_.trait_bounds, |bound| self.convert_trait_bound(bound));
                let id = self.get_trait_id(trait_id);
                Item::Trait(Trait {
                    id,
                    name,
                    generics,
                    bounds: parents,
                    where_clause,
                    comments,
                    methods,
                    trait_impls,
                })
            }
            expand_items::Item::TypeAlias(type_alias_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let name = type_alias.name.to_string();
                let r#type = self.convert_type(&type_alias.typ);
                let comments = self.doc_comments(ReferenceId::Alias(type_alias_id));
                let generics = vecmap(&type_alias.generics, convert_generic);
                let id = self.get_type_alias_id(type_alias_id);
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

    fn convert_module(&mut self, module: expand_items::Module) -> Module {
        let name = module.name.unwrap_or_default();
        let comments = self.doc_comments(ReferenceId::Module(module.id));
        let items = module
            .items
            .into_iter()
            .filter(|(visibility, _item)| visibility == &ItemVisibility::Public)
            .map(|(_, item)| self.convert_item(item))
            .collect();
        Module { name, comments, items }
    }

    fn convert_impl(&mut self, impl_: expand_items::Impl) -> Impl {
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
        let trait_id = self.get_trait_id(trait_.id);
        let trait_generics = vecmap(&trait_impl.trait_generics, |typ| self.convert_type(typ));
        let r#type = self.convert_type(&trait_impl.typ);
        let where_clause = vecmap(&trait_impl.where_clause, |constraint| {
            self.convert_trait_constraint(constraint)
        });
        TraitImpl { r#type, generics, methods, trait_id, trait_name, trait_generics, where_clause }
    }

    fn convert_trait_constraint(
        &mut self,
        constraint: &noirc_frontend::hir_def::traits::TraitConstraint,
    ) -> TraitConstraint {
        let r#type = self.convert_type(&constraint.typ);
        let bound = self.convert_trait_bound(&constraint.trait_bound);
        TraitConstraint { r#type, bound }
    }

    fn convert_trait_bound(&mut self, trait_bound: &ResolvedTraitBound) -> TraitBound {
        let trait_ = self.interner.get_trait(trait_bound.trait_id);
        let trait_name = trait_.name.to_string();
        let trait_id = self.get_trait_id(trait_.id);
        let (ordered_generics, named_generics) =
            self.convert_trait_generics(&trait_bound.trait_generics);
        TraitBound { trait_id, trait_name, ordered_generics, named_generics }
    }

    fn convert_type(&mut self, typ: &noirc_frontend::Type) -> Type {
        match typ {
            noirc_frontend::Type::Unit => Type::Unit,
            noirc_frontend::Type::FieldElement => Type::Primitive("Field".to_string()),
            noirc_frontend::Type::Bool
            | noirc_frontend::Type::Integer(..)
            | noirc_frontend::Type::Quoted(..)
            | noirc_frontend::Type::Error => Type::Primitive(typ.to_string()),
            noirc_frontend::Type::Array(length, element) => Type::Array {
                length: Box::new(self.convert_type(length)),
                element: Box::new(self.convert_type(element)),
            },
            noirc_frontend::Type::Slice(element) => {
                Type::Slice { element: Box::new(self.convert_type(element)) }
            }
            noirc_frontend::Type::String(length) => {
                Type::String { length: Box::new(self.convert_type(length)) }
            }
            noirc_frontend::Type::FmtString(length, element) => Type::FmtString {
                length: Box::new(self.convert_type(length)),
                element: Box::new(self.convert_type(element)),
            },
            noirc_frontend::Type::Tuple(types) => {
                Type::Tuple(vecmap(types, |typ| self.convert_type(typ)))
            }
            noirc_frontend::Type::DataType(data_type, generics) => {
                let data_type = data_type.borrow();
                if data_type.is_enum() {
                    panic!("Enums are not supported yet");
                }
                let id = self.get_type_id(data_type.id);
                let name = data_type.name.to_string();
                let generics = vecmap(generics, |typ| self.convert_type(typ));
                Type::Struct { id, name, generics }
            }
            noirc_frontend::Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let id = self.get_type_alias_id(type_alias.id);
                let name = type_alias.name.to_string();
                let generics = vecmap(generics, |typ| self.convert_type(typ));
                Type::TypeAlias { id, name, generics }
            }
            noirc_frontend::Type::TypeVariable(type_var) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    self.convert_type(typ)
                } else {
                    Type::Generic("_".to_string())
                }
            }
            noirc_frontend::Type::TraitAsType(trait_id, _, trait_generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                let trait_id = self.get_trait_id(trait_.id);
                let trait_name = trait_.name.to_string();
                let (ordered_generics, named_generics) =
                    self.convert_trait_generics(trait_generics);
                Type::TraitAsType { trait_id, trait_name, ordered_generics, named_generics }
            }
            noirc_frontend::Type::NamedGeneric(NamedGeneric { name, type_var, .. }) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    self.convert_type(typ)
                } else {
                    Type::Generic(name.to_string())
                }
            }
            noirc_frontend::Type::CheckedCast { from: _, to } => self.convert_type(to),
            noirc_frontend::Type::Function(args, return_type, env, unconstrained) => {
                Type::Function {
                    params: vecmap(args, |typ| self.convert_type(typ)),
                    return_type: Box::new(self.convert_type(return_type)),
                    env: Box::new(self.convert_type(env)),
                    unconstrained: *unconstrained,
                }
            }
            noirc_frontend::Type::Reference(typ, mutable) => {
                Type::Reference { r#type: Box::new(self.convert_type(typ)), mutable: *mutable }
            }
            noirc_frontend::Type::Forall(..) => {
                panic!("Should not need to print Type::Forall")
            }
            noirc_frontend::Type::Constant(signed_field, _kind) => {
                Type::Constant(signed_field.to_string())
            }
            noirc_frontend::Type::InfixExpr(lhs, operator, rhs, _) => Type::InfixExpr {
                lhs: Box::new(self.convert_type(lhs)),
                operator: operator.to_string(),
                rhs: Box::new(self.convert_type(rhs)),
            },
        }
    }

    fn convert_trait_generics(
        &mut self,
        trait_generics: &noirc_frontend::hir::type_check::generics::TraitGenerics,
    ) -> (Vec<Type>, std::collections::BTreeMap<String, Type>) {
        let ordered_generics = vecmap(&trait_generics.ordered, |typ| self.convert_type(typ));
        let named_generics = trait_generics
            .named
            .iter()
            .map(|named_type| (named_type.name.to_string(), self.convert_type(&named_type.typ)))
            .collect();
        (ordered_generics, named_generics)
    }

    fn convert_function(&mut self, func_id: FuncId) -> Function {
        let modifiers = self.interner.function_modifiers(&func_id);
        let func_meta = self.interner.function_meta(&func_id);
        let unconstrained = modifiers.is_unconstrained;
        let comptime = modifiers.is_comptime;
        let name = modifiers.name.to_string();
        let comments = self.doc_comments(ReferenceId::Function(func_id));
        let generics = vecmap(&func_meta.direct_generics, convert_generic);
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
        let trait_constraints = func_meta.trait_constraints.clone();
        let where_clause =
            vecmap(trait_constraints, |constraint| self.convert_trait_constraint(&constraint));
        Function {
            name,
            comments,
            unconstrained,
            comptime,
            generics,
            params,
            return_type,
            where_clause,
        }
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
            HirPattern::Mutable(inner_pattern, _) => self.pattern_to_string(inner_pattern),
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

    fn get_type_id(&mut self, id: TypeId) -> usize {
        let data_type = self.interner.get_type(id);
        let data_type = data_type.borrow();
        let location = data_type.location;
        let name = data_type.name.to_string();
        let kind = IdKeyKind::Type;
        let id = ItemId { location, kind, name };
        self.get_id(id)
    }

    fn get_trait_id(&mut self, id: TraitId) -> usize {
        let trait_ = self.interner.get_trait(id);
        let location = trait_.location;
        let name = trait_.name.to_string();
        let kind = IdKeyKind::Trait;
        let id = ItemId { location, kind, name };
        self.get_id(id)
    }

    fn get_type_alias_id(&mut self, id: TypeAliasId) -> usize {
        let alias = self.interner.get_type_alias(id);
        let alias = alias.borrow();
        let location = alias.location;
        let name = alias.name.to_string();
        let kind = IdKeyKind::TypeAlias;
        let id = ItemId { location, kind, name };
        self.get_id(id)
    }

    fn get_id(&mut self, key: ItemId) -> usize {
        if let Some(existing_id) = self.ids.get(&key) {
            *existing_id
        } else {
            let new_id = self.ids.len();
            self.ids.insert(key, new_id);
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
