use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_driver::CrateId;
use noirc_frontend::ast::{IntegerBitSize, ItemVisibility};
use noirc_frontend::graph::CrateGraph;
use noirc_frontend::hir::def_map::{LocalModuleId, ModuleDefId, ModuleId};
use noirc_frontend::hir::printer::items as expand_items;
use noirc_frontend::hir_def::stmt::{HirLetStatement, HirPattern};
use noirc_frontend::hir_def::traits::ResolvedTraitBound;
use noirc_frontend::node_interner::{FuncId, ReferenceId};
use noirc_frontend::shared::Signedness;
use noirc_frontend::{Kind, NamedGeneric, ResolvedGeneric, TypeBinding};
use noirc_frontend::{hir::def_map::DefMaps, node_interner::NodeInterner};

use crate::ids::{
    get_function_id, get_global_id, get_module_def_id, get_module_id, get_trait_id,
    get_type_alias_id, get_type_id,
};
use crate::items::{
    AssociatedConstant, AssociatedType, Function, FunctionParam, Generic, Global, Impl, Item, Link,
    LinkTarget, Links, Module, PrimitiveType, PrimitiveTypeKind, Reexport, Struct, StructField,
    Trait, TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias,
};
use crate::links::{CurrentType, LinkFinder};
pub use html::to_html;

mod html;
mod ids;
pub mod items;
pub mod links;

/// Returns the root module in a crate.
pub fn crate_module(
    crate_id: CrateId,
    crate_graph: &CrateGraph,
    def_maps: &DefMaps,
    interner: &NodeInterner,
) -> Module {
    let module = noirc_frontend::hir::printer::crate_to_module(crate_id, def_maps, interner);
    let mut builder = DocItemBuilder::new(interner, crate_id, crate_graph, def_maps);
    let mut module = builder.convert_module(module);
    builder.process_module_reexports(&mut module);
    module
}

struct DocItemBuilder<'a> {
    interner: &'a NodeInterner,
    crate_id: CrateId,
    crate_graph: &'a CrateGraph,
    def_maps: &'a DefMaps,
    current_module_id: LocalModuleId,
    current_type: Option<CurrentType>,
    /// The minimum visibility of the current module. For example,
    /// if the visibilities of parents modules are [pub, pub(crate), pub] then
    /// this will be `pub(crate)`.
    visibility: ItemVisibility,
    /// Maps a ModuleDefId to the item it converted to.
    /// This is needed because if an item is publicly exported, but the item
    /// isn't publicly visible (because its parent module is private) then we'll
    /// include the item directly under the module that publicly exports it.
    /// We do this by looking up the item in this map.
    module_def_id_to_item: HashMap<ModuleDefId, ConvertedItem>,
    module_imports: HashMap<ModuleId, Vec<expand_items::Import>>,
    /// Trait constraints in scope.
    /// These are set when a trait or trait impl is visited.
    trait_constraints: Vec<TraitConstraint>,
    link_finder: LinkFinder,
}

impl<'a> DocItemBuilder<'a> {
    fn new(
        interner: &'a NodeInterner,
        crate_id: CrateId,
        crate_graph: &'a CrateGraph,
        def_maps: &'a DefMaps,
    ) -> Self {
        let current_module_id = def_maps[&crate_id].root();
        let link_finder = LinkFinder::default();
        Self {
            interner,
            crate_id,
            crate_graph,
            def_maps,
            current_module_id,
            current_type: None,
            visibility: ItemVisibility::Public,
            module_def_id_to_item: HashMap::new(),
            module_imports: HashMap::new(),
            trait_constraints: Vec::new(),
            link_finder,
        }
    }
}

struct ConvertedItem {
    item: Item,
    // This is the maximum visibility the item has considering it's nested
    // in modules. For example, in `pub(crate) mod a { pub struct B {} }`,
    // struct B will have `pub(crate)` visibility here as it can't be publicly
    // reached.
    visibility: ItemVisibility,
}

impl DocItemBuilder<'_> {
    fn convert_item(&mut self, item: expand_items::Item, visibility: ItemVisibility) -> Item {
        let module_def_id = if let expand_items::Item::PrimitiveType(..) = item {
            None
        } else {
            Some(item.module_def_id())
        };
        let converted_item = match item {
            expand_items::Item::Module(module) => {
                let old_visibility = self.visibility;
                self.visibility = old_visibility.min(visibility);

                let old_module_id = self.current_module_id;
                self.current_module_id = module.id.local_id;

                let module = self.convert_module(module);

                self.visibility = old_visibility;
                self.current_module_id = old_module_id;

                Item::Module(module)
            }
            expand_items::Item::DataType(item_data_type) => {
                let type_id = item_data_type.id;
                self.current_type = Some(CurrentType::Type(type_id));
                let shared_data_type = self.interner.get_type(type_id);
                let data_type = shared_data_type.borrow();
                let comments = self.doc_comments(ReferenceId::Type(type_id));
                let mut has_private_fields = false;
                let fields = if data_type.is_enum() {
                    // Enums are shown as structs for now
                    Vec::new()
                } else {
                    data_type
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
                            let comments =
                                self.doc_comments(ReferenceId::StructMember(type_id, index));
                            let r#type = self.convert_type(&field.typ);
                            StructField { name: field.name.to_string(), r#type, comments }
                        })
                        .collect()
                };
                let generics = vecmap(&data_type.generics, |generic| self.convert_generic(generic));
                let impls = vecmap(item_data_type.impls, |impl_| self.convert_impl(impl_));
                let trait_impls =
                    vecmap(item_data_type.trait_impls, |impl_| self.convert_trait_impl(impl_));
                let id = get_type_id(type_id, self.interner);
                self.current_type = None;
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
                self.current_type = Some(CurrentType::Trait(trait_id));
                let trait_ = self.interner.get_trait(trait_id);
                let name = trait_.name.to_string();
                let comments = self.doc_comments(ReferenceId::Trait(trait_id));
                let generics = vecmap(&trait_.generics, |generic| self.convert_generic(generic));
                let where_clause = vecmap(&trait_.where_clause, |constraint| {
                    self.convert_trait_constraint(constraint)
                });
                self.trait_constraints = where_clause.clone();
                let mut required_methods = Vec::new();
                let mut provided_methods = Vec::new();
                for func_id in &item_trait.methods {
                    let has_body = self.interner.function(func_id).try_as_expr().is_some();
                    let function = self.convert_function(*func_id);
                    if has_body {
                        provided_methods.push(function);
                    } else {
                        required_methods.push(function);
                    }
                }
                self.trait_constraints.clear();
                let trait_impls = vecmap(item_trait.trait_impls, |trait_impl| {
                    self.convert_trait_impl(trait_impl)
                });
                let parents = vecmap(&trait_.trait_bounds, |bound| self.convert_trait_bound(bound));

                let mut associated_types = Vec::new();
                let mut associated_constants = Vec::new();

                for associated_type in &trait_.associated_types {
                    let name = associated_type.name.to_string();

                    if let Kind::Numeric(numeric_type) = associated_type.kind() {
                        let r#type = self.convert_type(&numeric_type);
                        associated_constants.push(AssociatedConstant { name, r#type });
                    } else {
                        let bounds = if let Some(trait_bounds) =
                            trait_.associated_type_bounds.get(associated_type.name.as_str())
                        {
                            vecmap(trait_bounds, |trait_bound| {
                                self.convert_trait_bound(trait_bound)
                            })
                        } else {
                            Vec::new()
                        };
                        associated_types.push(AssociatedType { name, bounds });
                    }
                }

                let id = get_trait_id(trait_id, self.interner);

                self.current_type = None;

                Item::Trait(Trait {
                    id,
                    name,
                    generics,
                    bounds: parents,
                    where_clause,
                    associated_types,
                    associated_constants,
                    comments,
                    required_methods,
                    provided_methods,
                    trait_impls,
                })
            }
            expand_items::Item::TypeAlias(type_alias_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let name = type_alias.name.to_string();
                let r#type = self.convert_type(&type_alias.typ);
                let comments = self.doc_comments(ReferenceId::Alias(type_alias_id));
                let generics =
                    vecmap(&type_alias.generics, |generic| self.convert_generic(generic));
                let id = get_type_alias_id(type_alias_id, self.interner);
                Item::TypeAlias(TypeAlias { id, name, comments, r#type, generics })
            }
            expand_items::Item::PrimitiveType(primitive_type) => {
                let kind = match &primitive_type.typ {
                    noirc_frontend::Type::String(..) => PrimitiveTypeKind::Str,
                    noirc_frontend::Type::FmtString(..) => PrimitiveTypeKind::Fmtstr,
                    noirc_frontend::Type::Array(..) => PrimitiveTypeKind::Array,
                    noirc_frontend::Type::Vector(..) => PrimitiveTypeKind::Vector,
                    _ => {
                        let Type::Primitive(kind) = self.convert_type(&primitive_type.typ) else {
                            panic!("Expected primitive type");
                        };
                        kind
                    }
                };
                self.current_type = Some(CurrentType::PrimitiveType(kind));
                let impls = vecmap(primitive_type.impls, |impl_| self.convert_impl(impl_));
                let trait_impls =
                    vecmap(primitive_type.trait_impls, |impl_| self.convert_trait_impl(impl_));
                let comments =
                    self.interner.primitive_docs.get(&kind.to_string()).cloned().map(|comments| {
                        vecmap(comments, |comment| comment.contents).join("\n").trim().to_string()
                    });
                let comments = comments.map(|comments| {
                    let links = self.find_links_in_comments(&comments);
                    (comments, links)
                });
                self.current_type = None;
                Item::PrimitiveType(PrimitiveType { kind, impls, trait_impls, comments })
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
                let id = get_global_id(global_id, self.interner);
                Item::Global(Global { id, name, comments, comptime, mutable, r#type })
            }
            expand_items::Item::Function(func_id) => Item::Function(self.convert_function(func_id)),
        };
        if let Some(module_def_id) = module_def_id {
            self.module_def_id_to_item.insert(
                module_def_id,
                ConvertedItem { item: converted_item.clone(), visibility: self.visibility },
            );
        }
        converted_item
    }

    fn convert_module(&mut self, module: expand_items::Module) -> Module {
        let name = module.name.unwrap_or_default();
        let comments = self.doc_comments(ReferenceId::Module(module.id));
        let items = module
            .items
            .into_iter()
            .map(|(visibility, item)| (visibility, self.convert_item(item, visibility)))
            .collect();
        self.module_imports.insert(module.id, module.imports);
        let is_contract = module.is_contract;
        let id = get_module_id(module.id, self.interner);
        Module { id, module_id: module.id, name, comments, items, is_contract }
    }

    fn convert_impl(&mut self, impl_: expand_items::Impl) -> Impl {
        let generics = vecmap(impl_.generics, |(name, kind)| {
            let numeric = self.kind_to_numeric(kind);
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
        let trait_impl_id = item_trait_impl.id;

        let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
        let trait_impl = trait_impl.borrow();

        let generics = vecmap(item_trait_impl.generics, |(name, kind)| {
            let numeric = self.kind_to_numeric(kind);
            Generic { name, numeric }
        });
        let where_clause = vecmap(&trait_impl.where_clause, |constraint| {
            self.convert_trait_constraint(constraint)
        });
        self.trait_constraints = where_clause.clone();
        let methods = vecmap(item_trait_impl.methods, |func_id| self.convert_function(func_id));
        self.trait_constraints.clear();

        let trait_ = self.interner.get_trait(trait_impl.trait_id);
        let trait_name = trait_.name.to_string();
        let trait_id = get_trait_id(trait_.id, self.interner);
        let trait_generics = vecmap(&trait_impl.trait_generics, |typ| self.convert_type(typ));
        let r#type = self.convert_type(&trait_impl.typ);
        TraitImpl { r#type, generics, methods, trait_id, trait_name, trait_generics, where_clause }
    }

    fn convert_trait_constraint(
        &self,
        constraint: &noirc_frontend::hir_def::traits::TraitConstraint,
    ) -> TraitConstraint {
        let r#type = self.convert_type(&constraint.typ);
        let bound = self.convert_trait_bound(&constraint.trait_bound);
        TraitConstraint { r#type, bound }
    }

    fn convert_trait_bound(&self, trait_bound: &ResolvedTraitBound) -> TraitBound {
        let trait_ = self.interner.get_trait(trait_bound.trait_id);
        let trait_name = trait_.name.to_string();
        let trait_id = get_trait_id(trait_.id, self.interner);
        let (ordered_generics, named_generics) =
            self.convert_trait_generics(&trait_bound.trait_generics);
        TraitBound { trait_id, trait_name, ordered_generics, named_generics }
    }

    fn convert_type(&self, typ: &noirc_frontend::Type) -> Type {
        match typ {
            noirc_frontend::Type::Unit => Type::Unit,
            noirc_frontend::Type::FieldElement => Type::Primitive(PrimitiveTypeKind::Field),
            noirc_frontend::Type::Bool => Type::Primitive(PrimitiveTypeKind::Bool),
            noirc_frontend::Type::Integer(signedness, bit_size) => match signedness {
                Signedness::Unsigned => match bit_size {
                    IntegerBitSize::One => Type::Primitive(PrimitiveTypeKind::U1),
                    IntegerBitSize::Eight => Type::Primitive(PrimitiveTypeKind::U8),
                    IntegerBitSize::Sixteen => Type::Primitive(PrimitiveTypeKind::U16),
                    IntegerBitSize::ThirtyTwo => Type::Primitive(PrimitiveTypeKind::U32),
                    IntegerBitSize::SixtyFour => Type::Primitive(PrimitiveTypeKind::U64),
                    IntegerBitSize::HundredTwentyEight => Type::Primitive(PrimitiveTypeKind::U128),
                },
                Signedness::Signed => match bit_size {
                    IntegerBitSize::One => panic!("There is no signed 1-bit integer"),
                    IntegerBitSize::Eight => Type::Primitive(PrimitiveTypeKind::I8),
                    IntegerBitSize::Sixteen => Type::Primitive(PrimitiveTypeKind::I16),
                    IntegerBitSize::ThirtyTwo => Type::Primitive(PrimitiveTypeKind::I32),
                    IntegerBitSize::SixtyFour => Type::Primitive(PrimitiveTypeKind::I64),
                    IntegerBitSize::HundredTwentyEight => {
                        panic!("There is no signed 128-bit integer")
                    }
                },
            },
            noirc_frontend::Type::Quoted(quoted) => match quoted {
                noirc_frontend::QuotedType::Expr => Type::Primitive(PrimitiveTypeKind::Expr),
                noirc_frontend::QuotedType::Quoted => Type::Primitive(PrimitiveTypeKind::Quoted),
                noirc_frontend::QuotedType::Type => Type::Primitive(PrimitiveTypeKind::Type),
                noirc_frontend::QuotedType::TypedExpr => {
                    Type::Primitive(PrimitiveTypeKind::TypedExpr)
                }
                noirc_frontend::QuotedType::TypeDefinition => {
                    Type::Primitive(PrimitiveTypeKind::TypeDefinition)
                }
                noirc_frontend::QuotedType::TraitConstraint => {
                    Type::Primitive(PrimitiveTypeKind::TraitConstraint)
                }
                noirc_frontend::QuotedType::TraitDefinition => {
                    Type::Primitive(PrimitiveTypeKind::TraitDefinition)
                }
                noirc_frontend::QuotedType::TraitImpl => {
                    Type::Primitive(PrimitiveTypeKind::TraitImpl)
                }
                noirc_frontend::QuotedType::UnresolvedType => {
                    Type::Primitive(PrimitiveTypeKind::UnresolvedType)
                }
                noirc_frontend::QuotedType::FunctionDefinition => {
                    Type::Primitive(PrimitiveTypeKind::FunctionDefinition)
                }
                noirc_frontend::QuotedType::Module => Type::Primitive(PrimitiveTypeKind::Module),
                noirc_frontend::QuotedType::CtString => {
                    Type::Primitive(PrimitiveTypeKind::CtString)
                }
            },
            noirc_frontend::Type::Array(length, element) => Type::Array {
                length: Box::new(self.convert_type(length)),
                element: Box::new(self.convert_type(element)),
            },
            noirc_frontend::Type::Vector(element) => {
                Type::Vector { element: Box::new(self.convert_type(element)) }
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
                // Enums are shown as structs for now
                let id = get_type_id(data_type.id, self.interner);
                let name = data_type.name.to_string();
                let generics = vecmap(generics, |typ| self.convert_type(typ));
                Type::Struct { id, name, generics }
            }
            noirc_frontend::Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let id = get_type_alias_id(type_alias.id, self.interner);
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
                let trait_id = get_trait_id(trait_.id, self.interner);
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
            noirc_frontend::Type::Constant(signed_field, _kind) => {
                Type::Constant(signed_field.to_string())
            }
            noirc_frontend::Type::InfixExpr(lhs, operator, rhs, _) => Type::InfixExpr {
                lhs: Box::new(self.convert_type(lhs)),
                operator: operator.to_string(),
                rhs: Box::new(self.convert_type(rhs)),
            },
            noirc_frontend::Type::Forall(..) => {
                panic!("Should not need to print Type::Forall")
            }
            noirc_frontend::Type::Error => {
                panic!("Should not need to print Type::Error")
            }
        }
    }

    fn convert_trait_generics(
        &self,
        trait_generics: &noirc_frontend::hir::type_check::generics::TraitGenerics,
    ) -> (Vec<Type>, std::collections::BTreeMap<String, Type>) {
        let ordered_generics = vecmap(&trait_generics.ordered, |typ| self.convert_type(typ));
        let named_generics = trait_generics
            .named
            .iter()
            .filter(|named_type| {
                if let noirc_frontend::Type::NamedGeneric(NamedGeneric { implicit: true, .. }) =
                    &named_type.typ
                {
                    return false;
                }
                true
            })
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
        let generics = vecmap(&func_meta.direct_generics, |generic| self.convert_generic(generic));
        let params = vecmap(func_meta.parameters.iter(), |(pattern, typ, _visibility)| {
            let is_self = self.pattern_is_self(pattern);

            // `&mut self` is represented as a mutable reference type, not as a mutable pattern
            let mut mut_ref = false;
            let name = if is_self && matches!(typ, noirc_frontend::Type::Reference(..)) {
                mut_ref = true;
                "self".to_string()
            } else {
                self.pattern_to_string(pattern)
            };

            let r#type = self.convert_type(typ);
            FunctionParam { name, r#type, mut_ref }
        });
        let return_type = self.convert_type(func_meta.return_type());
        let trait_constraints = func_meta.trait_constraints.clone();
        let where_clause =
            vecmap(trait_constraints, |constraint| self.convert_trait_constraint(&constraint));

        // Only keep trait constraints if they aren't already present because they exist in the
        // parent trait/impl.
        let where_clause = where_clause
            .iter()
            .filter(|trait_constraint| !self.trait_constraints.contains(trait_constraint))
            .cloned()
            .collect::<Vec<_>>();

        let attributes = self.interner.function_attributes(&func_id);
        let deprecated = attributes.get_deprecated_note();

        let id = get_function_id(func_id, self.interner);

        Function {
            id,
            name,
            comments,
            unconstrained,
            comptime,
            generics,
            params,
            return_type,
            where_clause,
            deprecated,
        }
    }

    fn convert_generic(&self, generic: &ResolvedGeneric) -> Generic {
        let numeric = self.kind_to_numeric(generic.kind());
        let name = generic.name.to_string();
        Generic { name, numeric }
    }

    fn kind_to_numeric(&self, kind: Kind) -> Option<Type> {
        match kind {
            Kind::Any
            | Kind::Normal
            | Kind::IntegerOrField
            | Kind::Integer
            | Kind::TypeInteger
            | Kind::TypeIntegerOrField => None,
            Kind::Numeric(typ) => Some(self.convert_type(&typ)),
        }
    }

    /// Goes over a module's imports. If an import is a re-export of a private item,
    /// the item is added to the module's items.
    fn process_module_reexports(&mut self, module: &mut Module) {
        // Process this module's sub-modules first because when we actually process the
        // imports we might add a publicly exported module into this module's items,
        // visiting it twice.
        for (_visibility, item) in &mut module.items {
            if let Item::Module(sub_module) = item {
                self.process_module_reexports(sub_module);
            }
        }

        let imports = self.module_imports.remove(&module.module_id).unwrap();
        for import in imports {
            if import.visibility == ItemVisibility::Private {
                continue;
            }

            if let Some(converted_item) = self.module_def_id_to_item.get(&import.id) {
                if converted_item.visibility < import.visibility {
                    // This is a re-export of a private item. The private item won't show up in
                    // its module docs (because it's private) so it's included directly under
                    // the module that re-exports it (this is how rustdoc works too).
                    let mut item = converted_item.item.clone();
                    item.set_name(import.name.to_string());

                    module.items.push((import.visibility, item));
                    continue;
                }
            }

            // This is an internal or external re-export
            let id = get_module_def_id(import.id, self.interner);
            module.items.push((
                import.visibility,
                Item::Reexport(Reexport {
                    id,
                    item_name: self.get_module_def_id_name(import.id),
                    name: import.name.to_string(),
                }),
            ));
        }
    }

    fn doc_comments(&mut self, id: ReferenceId) -> Option<(String, Links)> {
        let comments = self.interner.doc_comments(id)?;
        let comments =
            vecmap(comments, |comment| comment.contents.clone()).join("\n").trim().to_string();
        let links = self.find_links_in_comments(&comments);
        Some((comments, links))
    }

    /// The idea of this method is to find occurrences of markdown links and references in comments.
    /// For each of these we try to resolve them to a ModuleDefId of sort, which
    /// is actually represented as a Link to a type, method, module, etc.
    ///
    /// The doc generator ([html::to_html]) will then replace occurrences of these links
    /// with resolved HTML links.
    fn find_links_in_comments(&mut self, comments: &str) -> Links {
        let current_module_id = ModuleId { krate: self.crate_id, local_id: self.current_module_id };
        self.link_finder.reset();
        let links = self.link_finder.find_links(
            comments,
            current_module_id,
            self.current_type,
            self.interner,
            self.def_maps,
            self.crate_graph,
        );
        vecmap(links, |link| {
            let target = match link.target {
                links::LinkTarget::TopLevelItem(module_def_id) => {
                    LinkTarget::TopLevelItem(get_module_def_id(module_def_id, self.interner))
                }
                links::LinkTarget::Method(module_def_id, func_id) => {
                    let name = self.interner.function_name(&func_id).to_string();
                    LinkTarget::Method(get_module_def_id(module_def_id, self.interner), name)
                }
                links::LinkTarget::StructMember(type_id, index) => {
                    let data_type = self.interner.get_type(type_id);
                    let name = data_type.borrow().field_at(index).name.to_string();
                    LinkTarget::StructMember(get_type_id(type_id, self.interner), name)
                }
                links::LinkTarget::PrimitiveType(primitive_type_kind) => {
                    LinkTarget::PrimitiveType(primitive_type_kind)
                }
                links::LinkTarget::PrimitiveTypeFunction(primitive_type_kind, func_id) => {
                    let name = self.interner.function_name(&func_id).to_string();
                    LinkTarget::PrimitiveTypeFunction(primitive_type_kind, name)
                }
            };
            Link {
                name: link.name,
                path: link.path,
                target,
                line: link.line,
                start: link.start,
                end: link.end,
            }
        })
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

    fn get_module_def_id_name(&self, id: ModuleDefId) -> String {
        match id {
            ModuleDefId::ModuleId(module_id) => {
                let attributes = self.interner.try_module_attributes(module_id);
                attributes.map(|attributes| &attributes.name).cloned().unwrap_or_else(String::new)
            }
            ModuleDefId::FunctionId(func_id) => self.interner.function_name(&func_id).to_string(),
            ModuleDefId::TypeId(type_id) => {
                let data_type = self.interner.get_type(type_id);
                let data_type = data_type.borrow();
                data_type.name.to_string()
            }
            ModuleDefId::TypeAliasId(type_alias_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                type_alias.name.to_string()
            }
            ModuleDefId::TraitId(trait_id) => {
                let trait_ = self.interner.get_trait(trait_id);
                trait_.name.to_string()
            }
            ModuleDefId::TraitAssociatedTypeId(id) => {
                let associated_type = self.interner.get_trait_associated_type(id);
                associated_type.name.to_string()
            }
            ModuleDefId::GlobalId(global_id) => {
                let global_info = self.interner.get_global(global_id);
                global_info.ident.to_string()
            }
        }
    }
}

pub(crate) fn convert_primitive_type(
    primitive_type: noirc_frontend::elaborator::PrimitiveType,
) -> PrimitiveTypeKind {
    match primitive_type {
        noirc_frontend::elaborator::PrimitiveType::Field => PrimitiveTypeKind::Field,
        noirc_frontend::elaborator::PrimitiveType::Bool => PrimitiveTypeKind::Bool,
        noirc_frontend::elaborator::PrimitiveType::U1 => PrimitiveTypeKind::U1,
        noirc_frontend::elaborator::PrimitiveType::U8 => PrimitiveTypeKind::U8,
        noirc_frontend::elaborator::PrimitiveType::U16 => PrimitiveTypeKind::U16,
        noirc_frontend::elaborator::PrimitiveType::U32 => PrimitiveTypeKind::U32,
        noirc_frontend::elaborator::PrimitiveType::U64 => PrimitiveTypeKind::U64,
        noirc_frontend::elaborator::PrimitiveType::U128 => PrimitiveTypeKind::U128,
        noirc_frontend::elaborator::PrimitiveType::I8 => PrimitiveTypeKind::I8,
        noirc_frontend::elaborator::PrimitiveType::I16 => PrimitiveTypeKind::I16,
        noirc_frontend::elaborator::PrimitiveType::I32 => PrimitiveTypeKind::I32,
        noirc_frontend::elaborator::PrimitiveType::I64 => PrimitiveTypeKind::I64,
        noirc_frontend::elaborator::PrimitiveType::Str => PrimitiveTypeKind::Str,
        noirc_frontend::elaborator::PrimitiveType::Fmtstr => PrimitiveTypeKind::Fmtstr,
        noirc_frontend::elaborator::PrimitiveType::Expr => PrimitiveTypeKind::Expr,
        noirc_frontend::elaborator::PrimitiveType::Quoted => PrimitiveTypeKind::Quoted,
        noirc_frontend::elaborator::PrimitiveType::Type => PrimitiveTypeKind::Type,
        noirc_frontend::elaborator::PrimitiveType::TypedExpr => PrimitiveTypeKind::TypedExpr,
        noirc_frontend::elaborator::PrimitiveType::TypeDefinition => {
            PrimitiveTypeKind::TypeDefinition
        }
        noirc_frontend::elaborator::PrimitiveType::TraitConstraint => {
            PrimitiveTypeKind::TraitConstraint
        }
        noirc_frontend::elaborator::PrimitiveType::CtString => PrimitiveTypeKind::CtString,
        noirc_frontend::elaborator::PrimitiveType::FunctionDefinition => {
            PrimitiveTypeKind::FunctionDefinition
        }
        noirc_frontend::elaborator::PrimitiveType::Module => PrimitiveTypeKind::Module,
        noirc_frontend::elaborator::PrimitiveType::StructDefinition => {
            PrimitiveTypeKind::TypeDefinition
        }
        noirc_frontend::elaborator::PrimitiveType::TraitDefinition => {
            PrimitiveTypeKind::TraitDefinition
        }
        noirc_frontend::elaborator::PrimitiveType::TraitImpl => PrimitiveTypeKind::TraitImpl,
        noirc_frontend::elaborator::PrimitiveType::UnresolvedType => {
            PrimitiveTypeKind::UnresolvedType
        }
    }
}
