use fm::{FileId, FileMap};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use noirc_frontend::{
    ast::{ItemVisibility, Visibility},
    hir::def_map::ModuleId,
    hir_def::{
        expr::{HirArrayLiteral, HirExpression, HirLiteral},
        function::FuncMeta,
        stmt::HirPattern,
        traits::Trait,
    },
    node_interner::{
        DefinitionId, DefinitionKind, ExprId, FuncId, GlobalId, NodeInterner, ReferenceId, TraitId,
        TraitImplKind, TypeAliasId, TypeId,
    },
    DataType, EnumVariant, Generics, Shared, StructField, Type, TypeAlias, TypeBinding,
    TypeVariable,
};

use crate::{
    attribute_reference_finder::AttributeReferenceFinder,
    modules::module_full_path,
    requests::{to_lsp_location, ProcessRequestCallbackArgs},
    utils,
};

pub(super) fn hover_from_reference(
    file_id: Option<FileId>,
    position: Position,
    args: &ProcessRequestCallbackArgs,
) -> Option<Hover> {
    file_id
        .and_then(|file_id| {
            utils::position_to_byte_index(args.files, file_id, &position).and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = AttributeReferenceFinder::new(
                    file_id,
                    byte_index,
                    args.crate_id,
                    args.def_maps,
                );
                finder.find(&parsed_module)
            })
        })
        .or_else(|| args.interner.reference_at_location(args.location))
        .and_then(|reference| {
            let location = args.interner.reference_location(reference);
            let lsp_location = to_lsp_location(args.files, location.file, location.span);
            format_reference(reference, args).map(|formatted| Hover {
                range: lsp_location.map(|location| location.range),
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: formatted,
                }),
            })
        })
}

fn format_reference(reference: ReferenceId, args: &ProcessRequestCallbackArgs) -> Option<String> {
    match reference {
        ReferenceId::Module(id) => format_module(id, args),
        ReferenceId::Type(id) => Some(format_type(id, args)),
        ReferenceId::StructMember(id, field_index) => {
            Some(format_struct_member(id, field_index, args))
        }
        ReferenceId::EnumVariant(id, variant_index) => {
            Some(format_enum_variant(id, variant_index, args))
        }
        ReferenceId::Trait(id) => Some(format_trait(id, args)),
        ReferenceId::Global(id) => Some(format_global(id, args)),
        ReferenceId::Function(id) => Some(format_function(id, args)),
        ReferenceId::Alias(id) => Some(format_alias(id, args)),
        ReferenceId::Local(id) => Some(format_local(id, args)),
        ReferenceId::Reference(location, _) => {
            format_reference(args.interner.find_referenced(location).unwrap(), args)
        }
    }
}
fn format_module(id: ModuleId, args: &ProcessRequestCallbackArgs) -> Option<String> {
    let crate_root = args.def_maps[&id.krate].root();

    let mut string = String::new();

    if id.local_id == crate_root {
        let dep = args.dependencies.iter().find(|dep| dep.crate_id == id.krate)?;
        string.push_str("    crate ");
        string.push_str(&dep.name.to_string());
    } else {
        // Note: it's not clear why `try_module_attributes` might return None here, but it happens.
        // This is a workaround to avoid panicking in that case (which brings the LSP server down).
        // Cases where this happens are related to generated code, so once that stops happening
        // this won't be an issue anymore.
        let module_attributes = args.interner.try_module_attributes(&id)?;

        if let Some(parent_local_id) = module_attributes.parent {
            if format_parent_module_from_module_id(
                &ModuleId { krate: id.krate, local_id: parent_local_id },
                args,
                &mut string,
            ) {
                string.push('\n');
            }
        }
        string.push_str("    ");
        string.push_str("mod ");
        string.push_str(&module_attributes.name);
    }

    append_doc_comments(args.interner, ReferenceId::Module(id), &mut string);

    Some(string)
}

fn format_type(id: TypeId, args: &ProcessRequestCallbackArgs) -> String {
    let typ = args.interner.get_type(id);
    let typ = typ.borrow();
    if let Some(fields) = typ.get_fields_as_written() {
        format_struct(&typ, fields, args)
    } else if let Some(variants) = typ.get_variants_as_written() {
        format_enum(&typ, variants, args)
    } else {
        unreachable!("Type should either be a struct or an enum")
    }
}

fn format_struct(
    typ: &DataType,
    fields: Vec<StructField>,
    args: &ProcessRequestCallbackArgs,
) -> String {
    let mut string = String::new();
    if format_parent_module(ReferenceId::Type(typ.id), args, &mut string) {
        string.push('\n');
    }
    string.push_str("    ");
    string.push_str("struct ");
    string.push_str(&typ.name.0.contents);
    format_generics(&typ.generics, &mut string);
    string.push_str(" {\n");
    for field in fields {
        string.push_str("        ");
        string.push_str(&field.name.0.contents);
        string.push_str(": ");
        string.push_str(&format!("{}", field.typ));
        string.push_str(",\n");
    }
    string.push_str("    }");

    append_doc_comments(args.interner, ReferenceId::Type(typ.id), &mut string);

    string
}

fn format_enum(
    typ: &DataType,
    variants: Vec<EnumVariant>,
    args: &ProcessRequestCallbackArgs,
) -> String {
    let mut string = String::new();
    if format_parent_module(ReferenceId::Type(typ.id), args, &mut string) {
        string.push('\n');
    }
    string.push_str("    ");
    string.push_str("enum ");
    string.push_str(&typ.name.0.contents);
    format_generics(&typ.generics, &mut string);
    string.push_str(" {\n");
    for field in variants {
        string.push_str("        ");
        string.push_str(&field.name.0.contents);

        if !field.params.is_empty() {
            let types = field.params.iter().map(ToString::to_string).collect::<Vec<_>>();
            string.push('(');
            string.push_str(&types.join(", "));
            string.push(')');
        }

        string.push_str(",\n");
    }
    string.push_str("    }");

    append_doc_comments(args.interner, ReferenceId::Type(typ.id), &mut string);

    string
}

fn format_struct_member(
    id: TypeId,
    field_index: usize,
    args: &ProcessRequestCallbackArgs,
) -> String {
    let struct_type = args.interner.get_type(id);
    let struct_type = struct_type.borrow();
    let field = struct_type.field_at(field_index);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Type(id), args, &mut string) {
        string.push_str("::");
    }
    string.push_str(&struct_type.name.0.contents);
    string.push('\n');
    string.push_str("    ");
    string.push_str(&field.name.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", field.typ));
    string.push_str(&go_to_type_links(&field.typ, args.interner, args.files));

    append_doc_comments(args.interner, ReferenceId::StructMember(id, field_index), &mut string);

    string
}

fn format_enum_variant(
    id: TypeId,
    field_index: usize,
    args: &ProcessRequestCallbackArgs,
) -> String {
    let enum_type = args.interner.get_type(id);
    let enum_type = enum_type.borrow();
    let variant = enum_type.variant_at(field_index);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Type(id), args, &mut string) {
        string.push_str("::");
    }
    string.push_str(&enum_type.name.0.contents);
    string.push('\n');
    string.push_str("    ");
    string.push_str(&variant.name.0.contents);
    if !variant.params.is_empty() {
        let types = variant.params.iter().map(ToString::to_string).collect::<Vec<_>>();
        string.push('(');
        string.push_str(&types.join(", "));
        string.push(')');
    }

    for typ in variant.params.iter() {
        string.push_str(&go_to_type_links(typ, args.interner, args.files));
    }

    append_doc_comments(args.interner, ReferenceId::EnumVariant(id, field_index), &mut string);

    string
}

fn format_trait(id: TraitId, args: &ProcessRequestCallbackArgs) -> String {
    let a_trait = args.interner.get_trait(id);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Trait(id), args, &mut string) {
        string.push('\n');
    }
    string.push_str("    ");
    string.push_str("trait ");
    string.push_str(&a_trait.name.0.contents);
    format_generics(&a_trait.generics, &mut string);

    append_doc_comments(args.interner, ReferenceId::Trait(id), &mut string);

    string
}

fn format_global(id: GlobalId, args: &ProcessRequestCallbackArgs) -> String {
    let global_info = args.interner.get_global(id);
    let definition_id = global_info.definition_id;
    let definition = args.interner.definition(definition_id);
    let typ = args.interner.definition_type(definition_id);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Global(id), args, &mut string) {
        string.push('\n');
    }

    let mut print_comptime = definition.comptime;
    let mut opt_value = None;

    // See if we can figure out what's the global's value
    if let Some(stmt) = args.interner.get_global_let_statement(id) {
        print_comptime = stmt.comptime;
        opt_value = get_global_value(args.interner, stmt.expression);
    }

    string.push_str("    ");
    if print_comptime {
        string.push_str("comptime ");
    }
    if definition.mutable {
        string.push_str("mut ");
    }
    string.push_str("global ");
    string.push_str(&global_info.ident.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", typ));

    if let Some(value) = opt_value {
        string.push_str(" = ");
        string.push_str(&value);
    }

    string.push_str(&go_to_type_links(&typ, args.interner, args.files));

    append_doc_comments(args.interner, ReferenceId::Global(id), &mut string);

    string
}

fn get_global_value(interner: &NodeInterner, expr: ExprId) -> Option<String> {
    match interner.expression(&expr) {
        HirExpression::Literal(literal) => match literal {
            HirLiteral::Array(hir_array_literal) => {
                get_global_array_value(interner, hir_array_literal, false)
            }
            HirLiteral::Slice(hir_array_literal) => {
                get_global_array_value(interner, hir_array_literal, true)
            }
            HirLiteral::Bool(value) => Some(value.to_string()),
            HirLiteral::Integer(field_element, _) => Some(field_element.to_string()),
            HirLiteral::Str(string) => Some(format!("{:?}", string)),
            HirLiteral::FmtStr(..) => None,
            HirLiteral::Unit => Some("()".to_string()),
        },
        HirExpression::Tuple(values) => {
            get_exprs_global_value(interner, &values).map(|value| format!("({})", value))
        }
        _ => None,
    }
}

fn get_global_array_value(
    interner: &NodeInterner,
    literal: HirArrayLiteral,
    is_slice: bool,
) -> Option<String> {
    match literal {
        HirArrayLiteral::Standard(values) => {
            get_exprs_global_value(interner, &values).map(|value| {
                if is_slice {
                    format!("&[{}]", value)
                } else {
                    format!("[{}]", value)
                }
            })
        }
        HirArrayLiteral::Repeated { repeated_element, length } => {
            get_global_value(interner, repeated_element).map(|value| {
                if is_slice {
                    format!("&[{}; {}]", value, length)
                } else {
                    format!("[{}; {}]", value, length)
                }
            })
        }
    }
}

fn get_exprs_global_value(interner: &NodeInterner, exprs: &[ExprId]) -> Option<String> {
    let strings: Vec<String> =
        exprs.iter().filter_map(|value| get_global_value(interner, *value)).collect();
    if strings.len() == exprs.len() {
        Some(strings.join(", "))
    } else {
        None
    }
}

fn format_function(id: FuncId, args: &ProcessRequestCallbackArgs) -> String {
    let func_meta = args.interner.function_meta(&id);

    // If this points to a trait method, see if we can figure out what's the concrete trait impl method
    if let Some(func_id) = get_trait_impl_func_id(id, args, func_meta) {
        return format_function(func_id, args);
    }

    let func_modifiers = args.interner.function_modifiers(&id);

    let func_name_definition_id = args.interner.definition(func_meta.name.id);

    let enum_variant = match (func_meta.type_id, func_meta.enum_variant_index) {
        (Some(type_id), Some(index)) => Some((type_id, index)),
        _ => None,
    };

    let reference_id = if let Some((type_id, variant_index)) = enum_variant {
        ReferenceId::EnumVariant(type_id, variant_index)
    } else {
        ReferenceId::Function(id)
    };

    let mut string = String::new();
    let formatted_parent_module = format_parent_module(reference_id, args, &mut string);

    let formatted_parent_type = if let Some(trait_impl_id) = func_meta.trait_impl {
        let trait_impl = args.interner.get_trait_implementation(trait_impl_id);
        let trait_impl = trait_impl.borrow();
        let trait_ = args.interner.get_trait(trait_impl.trait_id);

        let generics: Vec<_> =
            trait_impl
                .trait_generics
                .iter()
                .filter_map(|generic| {
                    if let Type::NamedGeneric(_, name) = generic {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();

        string.push('\n');
        string.push_str("    impl");
        if !generics.is_empty() {
            string.push('<');
            for (index, generic) in generics.into_iter().enumerate() {
                if index > 0 {
                    string.push_str(", ");
                }
                string.push_str(generic);
            }
            string.push('>');
        }

        string.push(' ');
        string.push_str(&trait_.name.0.contents);
        if !trait_impl.trait_generics.is_empty() {
            string.push('<');
            for (index, generic) in trait_impl.trait_generics.iter().enumerate() {
                if index > 0 {
                    string.push_str(", ");
                }
                string.push_str(&generic.to_string());
            }
            string.push('>');
        }

        string.push_str(" for ");
        string.push_str(&trait_impl.typ.to_string());

        true
    } else if let Some(trait_id) = func_meta.trait_id {
        let trait_ = args.interner.get_trait(trait_id);
        string.push('\n');
        string.push_str("    trait ");
        string.push_str(&trait_.name.0.contents);
        format_generics(&trait_.generics, &mut string);

        true
    } else if let Some(type_id) = func_meta.type_id {
        let data_type = args.interner.get_type(type_id);
        let data_type = data_type.borrow();
        if formatted_parent_module {
            string.push_str("::");
        }
        string.push_str(&data_type.name.0.contents);
        if enum_variant.is_none() {
            string.push('\n');
            string.push_str("    ");
            string.push_str("impl");

            let impl_generics: Vec<_> = func_meta
                .all_generics
                .iter()
                .take(func_meta.all_generics.len() - func_meta.direct_generics.len())
                .cloned()
                .collect();
            format_generics(&impl_generics, &mut string);

            string.push(' ');
            string.push_str(&data_type.name.0.contents);
            format_generic_names(&impl_generics, &mut string);
        }

        true
    } else {
        false
    };
    if formatted_parent_module || formatted_parent_type {
        string.push('\n');
    }
    string.push_str("    ");

    if func_modifiers.visibility != ItemVisibility::Private
        && func_meta.trait_id.is_none()
        && func_meta.trait_impl.is_none()
    {
        string.push_str(&func_modifiers.visibility.to_string());
        string.push(' ');
    }
    if func_modifiers.is_unconstrained {
        string.push_str("unconstrained ");
    }
    if func_modifiers.is_comptime {
        string.push_str("comptime ");
    }

    let func_name = &func_name_definition_id.name;

    if enum_variant.is_none() {
        string.push_str("fn ");
    }
    string.push_str(func_name);
    format_generics(&func_meta.direct_generics, &mut string);
    string.push('(');
    let parameters = &func_meta.parameters;
    for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
        let is_self = pattern_is_self(pattern, args.interner);

        // `&mut self` is represented as a mutable reference type, not as a mutable pattern
        if is_self && matches!(typ, Type::MutableReference(..)) {
            string.push_str("&mut ");
        }

        if enum_variant.is_some() {
            string.push_str(&format!("{}", typ));
        } else {
            format_pattern(pattern, args.interner, &mut string);

            // Don't add type for `self` param
            if !is_self {
                string.push_str(": ");
                if matches!(visibility, Visibility::Public) {
                    string.push_str("pub ");
                }
                string.push_str(&format!("{}", typ));
            }
        }

        if index != parameters.len() - 1 {
            string.push_str(", ");
        }
    }

    string.push(')');

    if enum_variant.is_none() {
        let return_type = func_meta.return_type();
        match return_type {
            Type::Unit => (),
            _ => {
                string.push_str(" -> ");
                string.push_str(&format!("{}", return_type));
            }
        }

        string.push_str(&go_to_type_links(return_type, args.interner, args.files));
    }

    if enum_variant.is_some() {
        append_doc_comments(args.interner, reference_id, &mut string);
    } else {
        let had_doc_comments = append_doc_comments(args.interner, reference_id, &mut string);
        if !had_doc_comments {
            // If this function doesn't have doc comments, but it's a trait impl method,
            // use the trait method doc comments.
            if let Some(trait_impl_id) = func_meta.trait_impl {
                let trait_impl = args.interner.get_trait_implementation(trait_impl_id);
                let trait_impl = trait_impl.borrow();
                let trait_ = args.interner.get_trait(trait_impl.trait_id);
                if let Some(func_id) = trait_.method_ids.get(func_name) {
                    let reference_id = ReferenceId::Function(*func_id);
                    append_doc_comments(args.interner, reference_id, &mut string);
                }
            }
        }
    }

    string
}

fn get_trait_impl_func_id(
    id: FuncId,
    args: &ProcessRequestCallbackArgs,
    func_meta: &FuncMeta,
) -> Option<FuncId> {
    func_meta.trait_id?;

    let index = args.interner.find_location_index(args.location)?;
    let expr_id = args.interner.get_expr_id_from_index(index)?;
    let Some(TraitImplKind::Normal(trait_impl_id)) =
        args.interner.get_selected_impl_for_expression(expr_id)
    else {
        return None;
    };

    let trait_impl = args.interner.get_trait_implementation(trait_impl_id);
    let trait_impl = trait_impl.borrow();

    let function_name = args.interner.function_name(&id);
    let mut trait_impl_methods = trait_impl.methods.iter();
    let func_id =
        trait_impl_methods.find(|func_id| args.interner.function_name(func_id) == function_name)?;
    Some(*func_id)
}

fn format_alias(id: TypeAliasId, args: &ProcessRequestCallbackArgs) -> String {
    let type_alias = args.interner.get_type_alias(id);
    let type_alias = type_alias.borrow();

    let mut string = String::new();
    format_parent_module(ReferenceId::Alias(id), args, &mut string);
    string.push('\n');
    string.push_str("    ");
    string.push_str("type ");
    string.push_str(&type_alias.name.0.contents);
    string.push_str(" = ");
    string.push_str(&format!("{}", &type_alias.typ));

    append_doc_comments(args.interner, ReferenceId::Alias(id), &mut string);

    string
}

fn format_local(id: DefinitionId, args: &ProcessRequestCallbackArgs) -> String {
    let definition_info = args.interner.definition(id);
    if let DefinitionKind::Global(global_id) = &definition_info.kind {
        return format_global(*global_id, args);
    }

    let DefinitionKind::Local(expr_id) = definition_info.kind else {
        panic!("Expected a local reference to reference a local definition")
    };
    let typ = args.interner.definition_type(id);

    let mut string = String::new();
    string.push_str("    ");
    if definition_info.comptime {
        string.push_str("comptime ");
    }
    if expr_id.is_some() {
        string.push_str("let ");
    }
    if definition_info.mutable {
        if expr_id.is_none() {
            string.push_str("let ");
        }
        string.push_str("mut ");
    }
    string.push_str(&definition_info.name);
    if !matches!(typ, Type::Error) {
        string.push_str(": ");
        string.push_str(&format!("{}", typ));
    }

    string.push_str(&go_to_type_links(&typ, args.interner, args.files));

    string
}

fn format_generics(generics: &Generics, string: &mut String) {
    format_generics_impl(
        generics, false, // only show names
        string,
    );
}

fn format_generic_names(generics: &Generics, string: &mut String) {
    format_generics_impl(
        generics, true, // only show names
        string,
    );
}

fn format_generics_impl(generics: &Generics, only_show_names: bool, string: &mut String) {
    if generics.is_empty() {
        return;
    }

    string.push('<');
    for (index, generic) in generics.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }

        if only_show_names {
            string.push_str(&generic.name);
        } else {
            match generic.kind() {
                noirc_frontend::Kind::Any | noirc_frontend::Kind::Normal => {
                    string.push_str(&generic.name);
                }
                noirc_frontend::Kind::IntegerOrField | noirc_frontend::Kind::Integer => {
                    string.push_str("let ");
                    string.push_str(&generic.name);
                    string.push_str(": u32");
                }
                noirc_frontend::Kind::Numeric(typ) => {
                    string.push_str("let ");
                    string.push_str(&generic.name);
                    string.push_str(": ");
                    string.push_str(&typ.to_string());
                }
            }
        }
    }
    string.push('>');
}

fn format_pattern(pattern: &HirPattern, interner: &NodeInterner, string: &mut String) {
    match pattern {
        HirPattern::Identifier(ident) => {
            let definition = interner.definition(ident.id);
            string.push_str(&definition.name);
        }
        HirPattern::Mutable(pattern, _) => {
            string.push_str("mut ");
            format_pattern(pattern, interner, string);
        }
        HirPattern::Tuple(..) | HirPattern::Struct(..) => {
            string.push('_');
        }
    }
}

fn pattern_is_self(pattern: &HirPattern, interner: &NodeInterner) -> bool {
    match pattern {
        HirPattern::Identifier(ident) => {
            let definition = interner.definition(ident.id);
            definition.name == "self"
        }
        HirPattern::Mutable(pattern, _) => pattern_is_self(pattern, interner),
        HirPattern::Tuple(..) | HirPattern::Struct(..) => false,
    }
}

fn format_parent_module(
    referenced: ReferenceId,
    args: &ProcessRequestCallbackArgs,
    string: &mut String,
) -> bool {
    let Some(module) = args.interner.reference_module(referenced) else {
        return false;
    };

    format_parent_module_from_module_id(module, args, string)
}

fn format_parent_module_from_module_id(
    module: &ModuleId,
    args: &ProcessRequestCallbackArgs,
    string: &mut String,
) -> bool {
    let full_path =
        module_full_path(module, args.interner, args.crate_id, &args.crate_name, args.dependencies);
    if full_path.is_empty() {
        return false;
    }

    string.push_str("    ");
    string.push_str(&full_path);
    true
}

fn go_to_type_links(typ: &Type, interner: &NodeInterner, files: &FileMap) -> String {
    let mut gatherer = TypeLinksGatherer { interner, files, links: Vec::new() };
    gatherer.gather_type_links(typ);

    let links = gatherer.links;
    if links.is_empty() {
        "".to_string()
    } else {
        let mut string = String::new();
        string.push_str("\n\n");
        string.push_str("Go to ");
        for (index, link) in links.iter().enumerate() {
            if index > 0 {
                string.push_str(" | ");
            }
            string.push_str(link);
        }
        string
    }
}

struct TypeLinksGatherer<'a> {
    interner: &'a NodeInterner,
    files: &'a FileMap,
    links: Vec<String>,
}

impl<'a> TypeLinksGatherer<'a> {
    fn gather_type_links(&mut self, typ: &Type) {
        match typ {
            Type::Array(typ, _) => self.gather_type_links(typ),
            Type::Slice(typ) => self.gather_type_links(typ),
            Type::Tuple(types) => {
                for typ in types {
                    self.gather_type_links(typ);
                }
            }
            Type::DataType(data_type, generics) => {
                self.gather_struct_type_links(data_type);
                for generic in generics {
                    self.gather_type_links(generic);
                }
            }
            Type::Alias(type_alias, generics) => {
                self.gather_type_alias_links(type_alias);
                for generic in generics {
                    self.gather_type_links(generic);
                }
            }
            Type::TypeVariable(var) => {
                self.gather_type_variable_links(var);
            }
            Type::TraitAsType(trait_id, _, generics) => {
                let some_trait = self.interner.get_trait(*trait_id);
                self.gather_trait_links(some_trait);
                for generic in &generics.ordered {
                    self.gather_type_links(generic);
                }
                for named_type in &generics.named {
                    self.gather_type_links(&named_type.typ);
                }
            }
            Type::NamedGeneric(var, _) => {
                self.gather_type_variable_links(var);
            }
            Type::Function(args, return_type, env, _) => {
                for arg in args {
                    self.gather_type_links(arg);
                }
                self.gather_type_links(return_type);
                self.gather_type_links(env);
            }
            Type::MutableReference(typ) => self.gather_type_links(typ),
            Type::InfixExpr(lhs, _, rhs, _) => {
                self.gather_type_links(lhs);
                self.gather_type_links(rhs);
            }
            Type::CheckedCast { to, .. } => self.gather_type_links(to),
            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Forall(_, _)
            | Type::Constant(..)
            | Type::Quoted(_)
            | Type::Error => (),
        }
    }

    fn gather_struct_type_links(&mut self, struct_type: &Shared<DataType>) {
        let struct_type = struct_type.borrow();
        if let Some(lsp_location) =
            to_lsp_location(self.files, struct_type.location.file, struct_type.name.span())
        {
            self.push_link(format_link(struct_type.name.to_string(), lsp_location));
        }
    }

    fn gather_type_alias_links(&mut self, type_alias: &Shared<TypeAlias>) {
        let type_alias = type_alias.borrow();
        if let Some(lsp_location) =
            to_lsp_location(self.files, type_alias.location.file, type_alias.name.span())
        {
            self.push_link(format_link(type_alias.name.to_string(), lsp_location));
        }
    }

    fn gather_trait_links(&mut self, some_trait: &Trait) {
        if let Some(lsp_location) =
            to_lsp_location(self.files, some_trait.location.file, some_trait.name.span())
        {
            self.push_link(format_link(some_trait.name.to_string(), lsp_location));
        }
    }

    fn gather_type_variable_links(&mut self, var: &TypeVariable) {
        let var = &*var.borrow();
        match var {
            TypeBinding::Bound(typ) => {
                self.gather_type_links(typ);
            }
            TypeBinding::Unbound(..) => (),
        }
    }

    fn push_link(&mut self, link: String) {
        if !self.links.contains(&link) {
            self.links.push(link);
        }
    }
}

fn format_link(name: String, location: lsp_types::Location) -> String {
    format!(
        "[{}]({}#L{},{}-{},{})",
        name,
        location.uri,
        location.range.start.line + 1,
        location.range.start.character + 1,
        location.range.end.line + 1,
        location.range.end.character + 1
    )
}

fn append_doc_comments(interner: &NodeInterner, id: ReferenceId, string: &mut String) -> bool {
    if let Some(doc_comments) = interner.doc_comments(id) {
        string.push_str("\n\n---\n\n");
        for comment in doc_comments {
            string.push_str(comment);
            string.push('\n');
        }
        true
    } else {
        false
    }
}
