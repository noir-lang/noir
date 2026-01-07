use std::collections::{BTreeSet, HashMap, HashSet};

use crate::graph::{CrateGraph, CrateId};
use crate::hir::comptime::FormatStringFragment;
use crate::hir::printer::items::ItemBuilder;
use crate::hir::resolution::visibility::module_def_id_visibility;
use crate::node_interner::TraitImplId;
use crate::{
    DataType, Kind, NamedGeneric, ResolvedGenerics, Type,
    ast::{Ident, ItemVisibility},
    graph::Dependency,
    hir::{
        comptime::{Value, tokens_to_string_with_indent},
        def_map::{DefMaps, ModuleDefId, ModuleId},
        type_check::generics::TraitGenerics,
    },
    hir_def::{
        expr::HirExpression,
        stmt::{HirLetStatement, HirPattern},
        traits::{ResolvedTraitBound, TraitConstraint},
    },
    modules::{get_parent_module, module_def_id_is_visible, module_def_id_to_reference_id},
    node_interner::{FuncId, GlobalId, GlobalValue, NodeInterner, ReferenceId, TypeAliasId},
    shared::Visibility,
    token::{FunctionAttributeKind, LocatedToken, SecondaryAttribute, SecondaryAttributeKind},
};

pub mod items;

use items::{Impl, Import, Item, Module, Trait, TraitImpl};

/// Returns the HIR as human-readable code for the given crate.
/// At this point it is expected that all macros and comptime blocks have been expanded.
pub fn display_crate(
    crate_id: CrateId,
    crate_graph: &CrateGraph,
    def_maps: &DefMaps,
    interner: &NodeInterner,
) -> String {
    let module = crate_to_module(crate_id, def_maps, interner);

    let dependencies = &crate_graph[crate_id].dependencies;

    let mut string = String::new();
    let mut printer = ItemPrinter::new(crate_id, interner, def_maps, dependencies, &mut string);
    printer.show_module(module);

    string
}

pub fn crate_to_module(crate_id: CrateId, def_maps: &DefMaps, interner: &NodeInterner) -> Module {
    let root_module_id = def_maps[&crate_id].root();
    let module_id = ModuleId { krate: crate_id, local_id: root_module_id };

    let mut builder = ItemBuilder::new(crate_id, interner, def_maps);
    let mut module = builder.build_module(module_id);
    if crate_id.is_stdlib() {
        builder.add_primitive_types(&mut module.items);
    }
    module
}

struct ItemPrinter<'context, 'string> {
    crate_id: CrateId,
    interner: &'context NodeInterner,
    def_maps: &'context DefMaps,
    dependencies: &'context Vec<Dependency>,
    string: &'string mut String,
    indent: usize,
    module_id: ModuleId,
    imports: HashMap<ModuleDefId, Ident>,
    self_type: Option<Type>,

    /// Trait constraints in scope.
    /// These are set when a trait, trait impl or function is visited.
    trait_constraints: Vec<TraitConstraint>,
    /// Keep track of trait impls that have been printed so we don't show a
    /// same trait impl multiple times.
    trait_impls_printed: HashSet<TraitImplId>,
}

impl<'context, 'string> ItemPrinter<'context, 'string> {
    pub(super) fn new(
        crate_id: CrateId,
        interner: &'context NodeInterner,
        def_maps: &'context DefMaps,
        dependencies: &'context Vec<Dependency>,
        string: &'string mut String,
    ) -> Self {
        let root_id = def_maps[&crate_id].root();
        let module_id = ModuleId { krate: crate_id, local_id: root_id };
        let imports = HashMap::new();
        Self {
            crate_id,
            interner,
            def_maps,
            dependencies,
            string,
            indent: 0,
            module_id,
            imports,
            self_type: None,
            trait_constraints: Vec::new(),
            trait_impls_printed: HashSet::new(),
        }
    }

    pub(super) fn show_item(&mut self, item: Item) {
        match item {
            Item::Module(module) => self.show_module(module),
            Item::DataType(data_type) => self.show_data_type(data_type),
            Item::Trait(trait_) => self.show_trait(trait_),
            Item::TypeAlias(type_alias_id) => self.show_type_alias(type_alias_id),
            Item::PrimitiveType(_) => {
                // TODO: we don't show primitive types yet
            }
            Item::Global(global_id) => self.show_global(global_id),
            Item::Function(func_id) => self.show_function(func_id),
        }
    }

    fn show_module(&mut self, module: Module) {
        let module_id = module.id;

        if let Some(name) = &module.name {
            if module.is_contract {
                self.push_str("contract ");
            } else {
                self.push_str("mod ");
            }
            self.push_str(name);
            self.push_str(" {");
            self.increase_indent();
        }

        let previous_module_id = self.module_id;
        self.module_id = module_id;

        let previous_imports = std::mem::take(&mut self.imports);

        self.imports =
            module.imports.iter().map(|import| (import.id, import.name.clone())).collect();

        self.show_imports(module.imports);

        for (index, (visibility, item)) in module.items.into_iter().enumerate() {
            if index == 0 {
                self.push_str("\n");
            } else {
                self.push_str("\n\n");
            }
            self.write_indent();
            self.show_item_with_visibility(item, visibility);
        }

        self.module_id = previous_module_id;
        self.imports = previous_imports;

        if module.name.is_some() {
            self.push('\n');
            self.decrease_indent();
            self.write_indent();
            self.push_str("}");
        }
    }

    fn show_item_with_visibility(&mut self, item: Item, visibility: ItemVisibility) {
        let module_def_id = item.module_def_id();
        let reference_id = module_def_id_to_reference_id(module_def_id);
        self.show_doc_comments(reference_id);
        self.show_module_def_id_attributes(module_def_id);
        self.show_item_visibility(visibility);
        self.show_item(item);
    }

    fn show_doc_comments(&mut self, reference_id: ReferenceId) {
        let Some(doc_comments) = self.interner.doc_comments(reference_id) else {
            return;
        };

        for located_comment in doc_comments {
            let comment = &located_comment.contents;
            if comment.contains('\n') {
                let ends_with_newline = comment.ends_with('\n');

                self.push_str("/**");
                for (index, line) in comment.lines().enumerate() {
                    if index != 0 {
                        self.push('\n');
                        self.write_indent();
                    }
                    self.push_str(" * ");
                    self.push_str(line);
                }

                if ends_with_newline {
                    self.push('\n');
                    self.write_indent();
                }

                self.push_str("*/");
            } else {
                self.push_str("/// ");
                self.push_str(comment);
            }
            self.push('\n');
            self.write_indent();
        }
    }

    fn show_module_def_id_attributes(&mut self, module_def_id: ModuleDefId) {
        match module_def_id {
            ModuleDefId::FunctionId(func_id) => {
                let modifiers = self.interner.function_modifiers(&func_id);
                if let Some(attribute) = modifiers.attributes.function() {
                    self.push_str(&attribute.to_string());
                    self.push('\n');
                    self.write_indent();
                }
                self.show_secondary_attributes(&modifiers.attributes.secondary);
            }
            ModuleDefId::TypeId(type_id) => {
                self.show_secondary_attributes(self.interner.type_attributes(&type_id));
            }
            ModuleDefId::GlobalId(global_id) => {
                self.show_secondary_attributes(self.interner.global_attributes(&global_id));
            }
            ModuleDefId::ModuleId(..)
            | ModuleDefId::TypeAliasId(..)
            | ModuleDefId::TraitId(..)
            | ModuleDefId::TraitAssociatedTypeId(..) => {}
        }
    }

    fn show_secondary_attributes(&mut self, attributes: &[SecondaryAttribute]) {
        for attribute in attributes {
            if !matches!(attribute.kind, SecondaryAttributeKind::Meta(..)) {
                self.push_str(&attribute.to_string());
                self.push('\n');
                self.write_indent();
            }
        }
    }

    fn show_item_visibility(&mut self, visibility: ItemVisibility) {
        if visibility != ItemVisibility::Private {
            self.push_str(&visibility.to_string());
            self.push(' ');
        };
    }

    fn show_visibility(&mut self, visibility: Visibility) {
        if visibility != Visibility::Private {
            self.push_str(&visibility.to_string());
            self.push(' ');
        }
    }

    fn show_data_type(&mut self, item_data_type: items::DataType) {
        let type_id = item_data_type.id;
        let shared_data_type = self.interner.get_type(type_id);
        let data_type = shared_data_type.borrow();
        if data_type.is_struct() {
            self.show_struct(&data_type);
        } else if data_type.is_enum() {
            self.show_enum(&data_type);
        } else {
            unreachable!("DataType should either be a struct or an enum")
        }
        drop(data_type);

        self.show_data_type_impls(item_data_type.impls);

        let trait_impls = item_data_type.trait_impls.iter().collect::<Vec<_>>();
        self.show_trait_impls(&trait_impls);
    }

    fn show_struct(&mut self, data_type: &DataType) {
        self.push_str("struct ");
        self.push_str(&data_type.name.to_string());
        self.show_generics(&data_type.generics);
        self.push_str(" {\n");
        self.increase_indent();
        for (index, field) in data_type.get_fields_as_written().unwrap().into_iter().enumerate() {
            self.write_indent();
            self.show_doc_comments(ReferenceId::StructMember(data_type.id, index));
            self.show_item_visibility(field.visibility);
            self.push_str(&field.name.to_string());
            self.push_str(": ");
            self.show_type(&field.typ);
            self.push_str(",\n");
        }
        self.decrease_indent();
        self.write_indent();
        self.push('}');
    }

    fn show_enum(&mut self, data_type: &DataType) {
        self.push_str("enum ");
        self.push_str(&data_type.name.to_string());
        self.show_generics(&data_type.generics);
        self.push_str(" {\n");
        self.increase_indent();
        for (index, variant) in data_type.get_variants_as_written().unwrap().into_iter().enumerate()
        {
            self.write_indent();
            self.show_doc_comments(ReferenceId::EnumVariant(data_type.id, index));
            self.push_str(&variant.name.to_string());
            if variant.is_function {
                self.push('(');
                for (index, typ) in variant.params.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_type(typ);
                }
                self.push(')');
            }
            self.push_str(",\n");
        }
        self.decrease_indent();
        self.write_indent();
        self.push('}');
    }

    fn show_data_type_impls(&mut self, impls: Vec<Impl>) {
        for impl_ in impls {
            self.push_str("\n\n");
            self.write_indent();
            self.show_impl(impl_);
        }
    }

    fn show_impl(&mut self, impl_: Impl) {
        let typ = impl_.typ;

        self.push_str("impl");
        self.show_generic_type_variables(&impl_.generics);
        self.push(' ');
        self.show_type(&typ);
        self.push_str(" {\n");
        self.increase_indent();

        self.self_type = Some(typ.clone());

        for (index, (visibility, func_id)) in impl_.methods.iter().enumerate() {
            if index != 0 {
                self.push_str("\n\n");
            }
            self.write_indent();

            let item = Item::Function(*func_id);
            self.show_item_with_visibility(item, *visibility);
        }
        self.push('\n');
        self.decrease_indent();
        self.write_indent();
        self.push('}');

        self.self_type = None;
    }

    fn show_trait_impls(&mut self, trait_impls: &[&TraitImpl]) {
        for trait_impl in trait_impls {
            self.push_str("\n\n");
            self.write_indent();
            self.show_trait_impl(trait_impl);
        }
    }

    fn show_type_alias(&mut self, type_alias_id: TypeAliasId) {
        let type_alias = self.interner.get_type_alias(type_alias_id);
        let type_alias = type_alias.borrow();

        self.push_str("type ");
        self.push_str(&type_alias.name.to_string());
        self.show_generics(&type_alias.generics);
        self.push_str(" = ");
        self.show_type(&type_alias.typ);
        self.push(';');
    }

    fn show_trait(&mut self, item_trait: Trait) {
        let trait_id = item_trait.id;
        let trait_ = self.interner.get_trait(trait_id);

        self.push_str("trait ");
        self.push_str(&trait_.name.to_string());
        self.show_generics(&trait_.generics);

        if !trait_.trait_bounds.is_empty() {
            self.push_str(": ");
            self.show_trait_bounds(&trait_.trait_bounds);
        }

        self.show_where_clause(&trait_.where_clause);
        self.push_str(" {\n");
        self.increase_indent();

        self.trait_constraints = trait_.where_clause.clone();

        let mut printed_type_or_function = false;

        for associated_type in &trait_.associated_types {
            if printed_type_or_function {
                self.push_str("\n\n");
            }

            self.write_indent();

            if let Kind::Numeric(numeric_type) = associated_type.kind() {
                self.push_str("let ");
                self.push_str(&associated_type.name);
                self.push_str(": ");
                self.show_type(&numeric_type);
            } else {
                self.push_str("type ");
                self.push_str(&associated_type.name);
                if let Some(trait_bounds) =
                    trait_.associated_type_bounds.get(associated_type.name.as_str())
                {
                    if !trait_bounds.is_empty() {
                        self.push_str(": ");
                        self.show_trait_bounds(trait_bounds);
                    }
                }
            }

            self.push_str(";");
            printed_type_or_function = true;
        }

        for func_id in item_trait.methods {
            if printed_type_or_function {
                self.push_str("\n\n");
            }

            self.write_indent();

            let item = Item::Function(func_id);
            let visibility = ItemVisibility::Private;
            self.show_item_with_visibility(item, visibility);
            printed_type_or_function = true;
        }

        self.push('\n');
        self.decrease_indent();
        self.write_indent();
        self.push('}');

        self.trait_constraints.clear();

        // Only show trait impls for types outside of the current crate:
        // trait impls for types in this crate are already shown alongside the type definition.
        let trait_impls = item_trait
            .trait_impls
            .iter()
            .filter(|trait_impl| trait_impl.external_types)
            .collect::<Vec<_>>();
        self.show_trait_impls(&trait_impls);
    }

    fn show_trait_impl(&mut self, item_trait_impl: &TraitImpl) {
        if !self.trait_impls_printed.insert(item_trait_impl.id) {
            return;
        }

        let trait_impl_id = item_trait_impl.id;

        let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
        let trait_impl = trait_impl.borrow();
        let trait_ = self.interner.get_trait(trait_impl.trait_id);

        self.push_str("impl");
        self.show_generic_type_variables(&item_trait_impl.generics);
        self.push(' ');

        let use_import = true;
        self.show_reference_to_module_def_id(
            ModuleDefId::TraitId(trait_.id),
            trait_.visibility,
            use_import,
        );

        let use_colons = false;
        self.show_generic_types(&trait_impl.trait_generics, use_colons);

        self.push_str(" for ");
        self.show_type(&trait_impl.typ);
        self.show_where_clause(&trait_impl.where_clause);
        self.push_str(" {\n");
        self.increase_indent();

        self.trait_constraints = trait_impl.where_clause.clone();

        self.self_type = Some(trait_impl.typ.clone());

        let mut printed_item = false;

        let named = self.interner.get_associated_types_for_impl(trait_impl_id);
        for named_type in named {
            if printed_item {
                self.push_str("\n\n");
            }

            self.write_indent();

            if let Kind::Numeric(numeric_type) = named_type.typ.kind() {
                self.push_str("let ");
                self.push_str(&named_type.name.to_string());
                self.push_str(": ");
                self.show_type(&numeric_type);
                self.push_str(" = ");
            } else {
                self.push_str("type ");
                self.push_str(&named_type.name.to_string());
                self.push_str(" = ");
            }
            self.show_type(&named_type.typ);
            self.push_str(";");

            printed_item = true;
        }

        for method in &item_trait_impl.methods {
            if printed_item {
                self.push_str("\n\n");
            }
            self.write_indent();

            let item = Item::Function(*method);
            let visibility = ItemVisibility::Private;
            self.show_item_with_visibility(item, visibility);

            printed_item = true;
        }
        self.push('\n');
        self.decrease_indent();
        self.write_indent();
        self.push('}');

        self.self_type = None;
        self.trait_constraints.clear();
    }

    fn show_global(&mut self, global_id: GlobalId) {
        let global_info = self.interner.get_global(global_id);
        let definition_id = global_info.definition_id;
        let definition = self.interner.definition(definition_id);
        let typ = self.interner.definition_type(definition_id);

        if let Some(HirLetStatement { comptime: true, .. }) =
            self.interner.get_global_let_statement(global_id)
        {
            self.push_str("comptime ");
        }
        if definition.mutable {
            self.push_str("mut ");
        }
        self.push_str("global ");
        self.push_str(&global_info.ident.to_string());
        self.push_str(": ");
        self.show_type(&typ);
        if let GlobalValue::Resolved(value) = &global_info.value {
            self.push_str(" = ");
            self.show_value(value);
        };
        self.push_str(";");
    }

    fn show_function(&mut self, func_id: FuncId) {
        let modifiers = self.interner.function_modifiers(&func_id);
        let func_meta = self.interner.function_meta(&func_id);

        if modifiers.is_unconstrained {
            self.push_str("unconstrained ");
        }
        if modifiers.is_comptime {
            self.push_str("comptime ");
        }

        self.push_str("fn ");
        self.push_str(&modifiers.name);

        self.show_generics(&func_meta.direct_generics);

        self.push('(');
        let parameters = &func_meta.parameters;
        for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
            let is_self = self.pattern_is_self(pattern);

            // `&mut self` is represented as a mutable reference type, not as a mutable pattern
            if is_self && matches!(typ, Type::Reference(..)) {
                self.push_str("&mut ");
            }

            self.show_pattern(pattern);

            // Don't add type for `self` param
            if !is_self {
                self.push_str(": ");
                if matches!(visibility, Visibility::Public) {
                    self.push_str("pub ");
                }
                self.show_type(typ);
            }

            if index != parameters.len() - 1 {
                self.push_str(", ");
            }
        }
        self.push(')');

        let return_type = func_meta.return_type();
        match return_type {
            Type::Unit => (),
            _ => {
                self.push_str(" -> ");
                self.show_visibility(func_meta.return_visibility);
                self.show_type(return_type);
            }
        }

        // Only show trait constraints if they aren't already present because they exist in the
        // parent trait/impl.
        let func_trait_constraints = func_meta
            .trait_constraints
            .iter()
            .filter(|trait_constraint| !self.trait_constraints.contains(trait_constraint))
            .cloned()
            .collect::<Vec<_>>();

        self.show_where_clause(&func_trait_constraints);

        let previous_trait_constraints_length = self.trait_constraints.len();
        self.trait_constraints.extend(func_trait_constraints);

        let hir_function = self.interner.function(&func_id);
        if let Some(expr) = hir_function.try_as_expr() {
            let hir_expr = self.interner.expression(&expr);
            if let HirExpression::Block(_) = &hir_expr {
                self.push(' ');
                self.show_hir_expression(hir_expr, expr);
            } else {
                self.push_str(" {\n");
                self.increase_indent();
                self.write_indent();
                self.show_hir_expression(hir_expr, expr);
                self.push('\n');
                self.decrease_indent();
                self.write_indent();
                self.push('}');
            }
        } else {
            match &modifiers.attributes.function {
                Some((attribute, _)) => match attribute.kind {
                    FunctionAttributeKind::Foreign(_)
                    | FunctionAttributeKind::Builtin(_)
                    | FunctionAttributeKind::Oracle(_) => {
                        self.push_str(" {}");
                    }
                    FunctionAttributeKind::Test(..)
                    | FunctionAttributeKind::FuzzingHarness(..)
                    | FunctionAttributeKind::Fold
                    | FunctionAttributeKind::NoPredicates
                    | FunctionAttributeKind::InlineAlways
                    | FunctionAttributeKind::InlineNever => {
                        self.push(';');
                    }
                },
                None => {
                    self.push(';');
                }
            }
        }

        self.trait_constraints.truncate(previous_trait_constraints_length);
    }

    fn show_generic_types(&mut self, types: &[Type], use_colons: bool) {
        if types.is_empty() {
            return;
        }
        if use_colons {
            self.push_str("::");
        }
        self.push('<');
        self.show_types_separated_by_comma(types);
        self.push('>');
    }

    fn show_generics(&mut self, generics: &ResolvedGenerics) {
        if generics.is_empty() {
            return;
        }

        self.push('<');
        for (index, generic) in generics.iter().enumerate() {
            if index > 0 {
                self.push_str(", ");
            }
            self.show_generic_kind(&generic.name, &generic.kind());
        }
        self.push('>');
    }

    fn show_generic_kind(&mut self, name: &str, kind: &Kind) {
        match kind {
            Kind::Any | Kind::Normal => {
                self.push_str(name);
            }
            Kind::IntegerOrField | Kind::Integer => {
                self.push_str("let ");
                self.push_str(name);
                self.push_str(": u32");
            }
            Kind::Numeric(typ) => {
                self.push_str("let ");
                self.push_str(name);
                self.push_str(": ");
                self.show_type(typ);
            }
        }
    }

    fn show_trait_generics(&mut self, generics: &TraitGenerics) {
        let ordered = &generics.ordered;

        // Exclude named generics that are unbound because it's the same as not including them
        let named = generics
            .named
            .iter()
            .filter(|named| {
                let typ = named.typ.follow_bindings();
                match &typ {
                    Type::TypeVariable(type_var)
                    | Type::NamedGeneric(NamedGeneric { type_var, .. })
                        if type_var.borrow().is_unbound() =>
                    {
                        false
                    }
                    Type::Constant(..) => false,
                    _ => true,
                }
            })
            .collect::<Vec<_>>();

        if ordered.is_empty() && named.is_empty() {
            return;
        }

        let mut printed_type = false;

        self.push('<');

        for typ in ordered {
            if printed_type {
                self.push_str(", ");
            }

            self.show_type(typ);
            printed_type = true;
        }

        for named_type in named {
            if printed_type {
                self.push_str(", ");
            }

            self.push_str(&named_type.name.to_string());
            self.push_str(" = ");
            self.show_type(&named_type.typ);
            printed_type = true;
        }

        self.push('>');
    }

    fn show_generic_type_variables(&mut self, generics: &BTreeSet<(String, Kind)>) {
        if generics.is_empty() {
            return;
        }

        self.push('<');
        for (index, (name, kind)) in generics.iter().enumerate() {
            if index != 0 {
                self.push_str(", ");
            }
            self.show_generic_kind(name, kind);
        }
        self.push('>');
    }

    fn show_where_clause(&mut self, constraints: &[TraitConstraint]) {
        if constraints.is_empty() {
            return;
        }

        self.push_str(" where ");
        for (index, constraint) in constraints.iter().enumerate() {
            if index != 0 {
                self.push_str(", ");
            }
            self.show_type(&constraint.typ);
            self.push_str(": ");
            self.show_trait_bound(&constraint.trait_bound);
        }
    }

    fn show_trait_bound(&mut self, bound: &ResolvedTraitBound) {
        let trait_ = self.interner.get_trait(bound.trait_id);
        self.show_reference_to_module_def_id(
            ModuleDefId::TraitId(trait_.id),
            trait_.visibility,
            true,
        );
        self.show_trait_generics(&bound.trait_generics);
    }

    fn show_trait_bounds(&mut self, bounds: &[ResolvedTraitBound]) {
        for (index, trait_bound) in bounds.iter().enumerate() {
            if index != 0 {
                self.push_str(" + ");
            }
            self.show_trait_bound(trait_bound);
        }
    }

    fn show_pattern(&mut self, pattern: &HirPattern) {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                self.push_str(&definition.name);
            }
            HirPattern::Mutable(pattern, _) => {
                self.push_str("mut ");
                self.show_pattern(pattern);
            }
            HirPattern::Tuple(patterns, _) => {
                let len = patterns.len();
                self.push('(');
                for (index, pattern) in patterns.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_pattern(pattern);
                }
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            HirPattern::Struct(typ, fields, _) => {
                self.show_type_name_as_data_type(typ);
                self.push_str(" { ");
                for (index, (name, pattern)) in fields.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.push_str(name.as_str());
                    self.push_str(": ");
                    self.show_pattern(pattern);
                }

                self.push_str(" }");
            }
        }
    }

    fn show_value(&mut self, value: &Value) {
        match value {
            Value::Unit => self.push_str("()"),
            Value::Bool(bool) => self.push_str(&bool.to_string()),
            Value::Field(value) => self.push_str(&value.to_string()),
            Value::I8(value) => self.push_str(&value.to_string()),
            Value::I16(value) => self.push_str(&value.to_string()),
            Value::I32(value) => self.push_str(&value.to_string()),
            Value::I64(value) => self.push_str(&value.to_string()),
            Value::U1(value) => self.push_str(&value.to_string()),
            Value::U8(value) => self.push_str(&value.to_string()),
            Value::U16(value) => self.push_str(&value.to_string()),
            Value::U32(value) => self.push_str(&value.to_string()),
            Value::U64(value) => self.push_str(&value.to_string()),
            Value::U128(value) => self.push_str(&value.to_string()),
            Value::String(string) => self.push_str(&format!("{string:?}")),
            Value::FormatString(fragments, _typ, _) => {
                let has_values = fragments
                    .iter()
                    .any(|fragment| matches!(fragment, FormatStringFragment::Value { .. }));

                if has_values {
                    self.push_str("{\n");

                    let mut seen_names: HashSet<String> = HashSet::default();

                    for fragment in fragments.iter() {
                        if let FormatStringFragment::Value { name, value } = fragment {
                            // A name might be interpolated multiple times. In that case it will always
                            // have the same value: we just need one `let` for it.
                            if !seen_names.insert(name.to_string()) {
                                continue;
                            }

                            self.push_str("let ");
                            self.push_str(name);
                            self.push_str(" = ");
                            self.show_value(value);
                            self.push_str(";\n");
                        }
                    }
                }

                self.push_str("f\"");
                for fragment in fragments.iter() {
                    match fragment {
                        FormatStringFragment::String(string) => {
                            self.push_str(&string.replace('"', "\\\""));
                        }
                        FormatStringFragment::Value { name, value: _ } => {
                            self.push('{');
                            self.push_str(name);
                            self.push('}');
                        }
                    }
                }
                self.push_str("\"");

                if has_values {
                    self.push_str(" }");
                }
            }
            Value::CtString(string) => {
                let std = if self.crate_id.is_stdlib() { "std" } else { "crate" };
                self.push_str(&format!(
                    "{std}::meta::ctstring::AsCtString::as_ctstring({string:?})"
                ));
            }
            Value::Function(func_id, ..) => {
                let use_import = true;
                let visibility = self.interner.function_modifiers(func_id).visibility;
                self.show_reference_to_module_def_id(
                    ModuleDefId::FunctionId(*func_id),
                    visibility,
                    use_import,
                );
            }
            Value::Tuple(values) => {
                self.push('(');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_value(&value.borrow());
                }
                if values.len() == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            Value::Struct(fields, typ) => {
                self.show_type_name_as_data_type(typ);

                if fields.is_empty() {
                    self.push_str(" {}");
                } else {
                    self.push_str(" {\n");
                    self.increase_indent();
                    for (name, value) in fields {
                        self.write_indent();
                        self.push_str(name);
                        self.push_str(": ");
                        self.show_value(&value.borrow());
                        self.push_str(",\n");
                    }
                    self.decrease_indent();
                    self.write_indent();
                    self.push('}');
                }
            }
            Value::Enum(index, args, typ) => {
                self.show_type_name_as_data_type(typ);

                let Type::DataType(data_type, _generics) = typ.follow_bindings() else {
                    panic!("Expected typ to be a data type");
                };
                let data_type = data_type.borrow();

                let variant = data_type.variant_at(*index);
                self.push_str("::");
                self.push_str(&variant.name.to_string());
                if variant.is_function {
                    self.push('(');
                    for (index, arg) in args.iter().enumerate() {
                        if index != 0 {
                            self.push_str(", ");
                        }
                        self.show_value(arg);
                    }
                    self.push(')');
                }
            }
            Value::Array(values, _) => {
                self.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_value(value);
                }
                self.push(']');
            }
            Value::Vector(values, _) => {
                self.push_str("&[");
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_value(value);
                }
                self.push(']');
            }
            Value::Quoted(tokens) => {
                self.show_quoted(tokens);
            }
            Value::Pointer(value, ..) => {
                self.show_value(&value.borrow());
            }
            Value::Zeroed(_) => {
                let std = if self.crate_id.is_stdlib() { "std" } else { "crate" };
                self.push_str(&format!("{std}::mem::zeroed()"));
            }
            Value::Closure(closure) => {
                self.show_hir_lambda(closure.lambda.clone());
            }
            Value::TypeDefinition(_)
            | Value::TraitConstraint(..)
            | Value::TraitDefinition(_)
            | Value::TraitImpl(_)
            | Value::FunctionDefinition(_)
            | Value::ModuleDefinition(_)
            | Value::Type(_)
            | Value::Expr(_)
            | Value::TypedExpr(_)
            | Value::UnresolvedType(_) => {
                if self.crate_id.is_stdlib() {
                    self.push_str(
                        "crate::panic(f\"comptime value that cannot be represented with code\")",
                    );
                } else {
                    self.push_str(
                        "panic(f\"comptime value that cannot be represented with code\")",
                    );
                }
            }
        }
    }

    fn show_type_name_as_data_type(&mut self, typ: &Type) {
        if self.self_type.as_ref() == Some(typ) {
            self.push_str("Self");
            return;
        }

        let Type::DataType(data_type, generics) = typ.follow_bindings() else {
            panic!("Expected a data type, got: {typ:?}");
        };

        let data_type = data_type.borrow();
        let use_import = true;
        self.show_reference_to_module_def_id(
            ModuleDefId::TypeId(data_type.id),
            data_type.visibility,
            use_import,
        );

        let use_colons = true;
        self.show_generic_types(&generics, use_colons);
    }

    fn show_imports(&mut self, imports: Vec<Import>) {
        let mut first = true;

        for import in imports {
            if import.is_prelude {
                continue;
            }

            if first {
                self.push('\n');
                first = false;
            }
            self.write_indent();
            self.show_item_visibility(import.visibility);
            self.push_str("use ");
            let use_import = false;
            let visibility = self.module_def_id_visibility(import.id);
            let name = self.show_reference_to_module_def_id(import.id, visibility, use_import);

            if name != import.name.as_str() {
                self.push_str(" as ");
                self.push_str(import.name.as_str());
            }
            self.push(';');
            self.push('\n');
        }
    }

    fn show_reference_to_module_def_id(
        &mut self,
        module_def_id: ModuleDefId,
        visibility: ItemVisibility,
        use_import: bool,
    ) -> String {
        if let ModuleDefId::FunctionId(func_id) = module_def_id {
            let func_meta = self.interner.function_meta(&func_id);

            if let Some(trait_impl_id) = func_meta.trait_impl {
                let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
                let trait_impl = trait_impl.borrow();
                let trait_ = self.interner.get_trait(trait_impl.trait_id);
                self.show_reference_to_module_def_id(
                    ModuleDefId::TraitId(trait_impl.trait_id),
                    trait_.visibility,
                    use_import,
                );

                let use_colons = true;
                self.show_generic_types(&trait_impl.trait_generics, use_colons);

                self.push_str("::");

                let name = self.interner.function_name(&func_id).to_string();
                self.push_str(&name);
                return name;
            }

            if let Some(trait_id) = func_meta.trait_id {
                let trait_ = self.interner.get_trait(trait_id);
                self.show_reference_to_module_def_id(
                    ModuleDefId::TraitId(trait_id),
                    trait_.visibility,
                    use_import,
                );
                self.push_str("::");

                let name = self.interner.function_name(&func_id).to_string();
                self.push_str(&name);
                return name;
            }

            if let Some(type_id) = func_meta.type_id {
                let typ = self.interner.get_type(type_id);
                let typ = typ.borrow();
                self.show_reference_to_module_def_id(
                    ModuleDefId::TypeId(type_id),
                    typ.visibility,
                    use_import,
                );
                self.push_str("::");

                let name = self.interner.function_name(&func_id).to_string();
                self.push_str(&name);
                return name;
            }

            if let Some(self_type) = &func_meta.self_type {
                if self_type.is_primitive() {
                    // Type path, like `Field::method(...)`
                    self.show_type(self_type);
                    self.push_str("::");

                    let name = self.interner.function_name(&func_id).to_string();
                    self.push_str(&name);
                    return name;
                }
            }
        }

        if use_import {
            if let Some(name) = self.imports.get(&module_def_id) {
                let name = name.to_string();
                self.push_str(&name);
                return name;
            }
        }

        let current_module_parent_id = self.module_id.parent(self.def_maps);

        // Check if module_def_id is the current module's parent
        if let ModuleDefId::ModuleId(module_id) = module_def_id {
            if current_module_parent_id == Some(module_id) {
                // If the parent is actually the crate's root, use "crate"
                if current_module_parent_id.unwrap().parent(self.def_maps).is_none() {
                    self.push_str("crate");
                    return "crate".to_string();
                }

                self.push_str("super");
                return "super".to_string();
            }
        }

        let is_visible = module_def_id_is_visible(
            module_def_id,
            self.module_id,
            visibility,
            None,
            self.interner,
            self.def_maps,
            self.dependencies,
        );
        if !is_visible {
            if let Some(reexport) = self.interner.get_reexports(module_def_id).first() {
                self.show_reference_to_module_def_id(
                    ModuleDefId::ModuleId(reexport.module_id),
                    reexport.visibility,
                    true,
                );
                self.push_str("::");
                self.push_str(reexport.name.as_str());
                return reexport.name.to_string();
            }
        }

        // Recurse on the parent module, but only if the parent module isn't the current module
        // (if so, we can already reach the definition just by printing its name)
        let module_def_id_parent_module =
            get_parent_module(module_def_id, self.interner, self.def_maps);
        if module_def_id_parent_module != Some(self.module_id) {
            if let Some(module_def_id_parent_module) = module_def_id_parent_module {
                let visibility = self
                    .module_def_id_visibility(ModuleDefId::ModuleId(module_def_id_parent_module));
                self.show_reference_to_module_def_id(
                    ModuleDefId::ModuleId(module_def_id_parent_module),
                    visibility,
                    use_import,
                );
                self.push_str("::");
            }
        }

        let name = self.module_def_id_name(module_def_id);
        self.push_str(&name);
        name
    }

    fn show_quoted(&mut self, tokens: &[LocatedToken]) {
        self.push_str("quote {");
        let preserve_unquote_markers = true;
        let string = tokens_to_string_with_indent(
            tokens,
            self.indent + 1,
            preserve_unquote_markers,
            self.interner,
        );
        if string.contains('\n') {
            self.push('\n');
            self.increase_indent();
            self.write_indent();
            self.push_str(string.trim());
            self.push('\n');
            self.decrease_indent();
            self.write_indent();
        } else {
            self.push(' ');
            self.push_str(&string);
            self.push(' ');
        }
        self.push_str("}");
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

    fn pattern_is_self_or_underscore_self(&self, pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                definition.name == "self" || definition.name == "_self"
            }
            HirPattern::Mutable(pattern, _) => self.pattern_is_self(pattern),
            HirPattern::Tuple(..) | HirPattern::Struct(..) => false,
        }
    }

    fn module_def_id_name(&self, module_def_id: ModuleDefId) -> String {
        match module_def_id {
            ModuleDefId::ModuleId(module_id) => {
                let attributes = self.interner.try_module_attributes(module_id);
                let name = attributes.map(|attributes| &attributes.name);
                // A module might not have a name if it's the root module of a crate
                name.cloned().unwrap_or_else(|| "crate".to_string())
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

    fn module_def_id_visibility(&self, module_def_id: ModuleDefId) -> ItemVisibility {
        module_def_id_visibility(module_def_id, self.interner)
    }

    fn show_separated_by_comma<Item, F>(&mut self, items: &[Item], f: F)
    where
        F: Fn(&mut Self, &Item),
    {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.push_str(", ");
            }
            f(self, item);
        }
    }

    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.push_str("    ");
        }
    }

    fn push_str(&mut self, str: &str) {
        self.string.push_str(str);
    }

    fn push(&mut self, char: char) {
        self.string.push(char);
    }
}
