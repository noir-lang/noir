use std::collections::{HashMap, HashSet};

use noirc_driver::CrateId;
use noirc_errors::Location;
use noirc_frontend::{
    DataType, Generics, Type,
    ast::{ItemVisibility, Visibility},
    hir::{
        comptime::{Value, tokens_to_string},
        def_map::{CrateDefMap, ModuleDefId, ModuleId},
        type_check::generics::TraitGenerics,
    },
    hir_def::{
        expr::HirExpression,
        stmt::{HirLetStatement, HirPattern},
        traits::{ResolvedTraitBound, TraitConstraint},
    },
    node_interner::{
        FuncId, GlobalId, GlobalValue, ImplMethod, Methods, NodeInterner, ReferenceId, TraitId,
        TypeAliasId, TypeId,
    },
};

pub(super) struct Printer<'interner, 'def_map, 'string> {
    crate_id: CrateId,
    interner: &'interner NodeInterner,
    def_map: &'def_map CrateDefMap,
    string: &'string mut String,
    indent: usize,
}

impl<'interner, 'def_map, 'string> Printer<'interner, 'def_map, 'string> {
    pub(super) fn new(
        crate_id: CrateId,
        interner: &'interner NodeInterner,
        def_map: &'def_map CrateDefMap,
        string: &'string mut String,
    ) -> Self {
        Self { crate_id, interner, def_map, string, indent: 0 }
    }

    pub(super) fn show_module(&mut self, module_id: ModuleId) {
        let attributes = self.interner.try_module_attributes(&module_id);
        let name = attributes.map(|attributes| &attributes.name);
        let module_data = &self.def_map.modules()[module_id.local_id.0];
        let is_contract = module_data.is_contract;

        if let Some(name) = name {
            self.write_indent();
            if is_contract {
                self.push_str("contract ");
            } else {
                self.push_str("mod ");
            }
            self.push_str(name);
            self.push_str(" {");
            self.increase_indent();
        }

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

        for (index, (module_def_id, visibility, _location)) in definitions.iter().enumerate() {
            if index == 0 {
                self.push_str("\n");
            } else {
                self.push_str("\n\n");
            }
            self.write_indent();
            self.show_module_def_id(*module_def_id, *visibility);
        }

        if name.is_some() {
            self.push('\n');
            self.decrease_indent();
            self.write_indent();
            self.push_str("}");
        }
    }

    fn show_module_def_id(&mut self, module_def_id: ModuleDefId, visibility: ItemVisibility) {
        if visibility != ItemVisibility::Private {
            self.push_str(&visibility.to_string());
            self.push(' ');
        };

        match module_def_id {
            ModuleDefId::ModuleId(module_id) => {
                self.show_module(module_id);
            }
            ModuleDefId::TypeId(type_id) => self.show_data_type(type_id),
            ModuleDefId::TypeAliasId(type_alias_id) => self.show_type_alias(type_alias_id),
            ModuleDefId::TraitId(trait_id) => self.show_trait(trait_id),
            ModuleDefId::GlobalId(global_id) => self.show_global(global_id),
            ModuleDefId::FunctionId(func_id) => self.show_function(func_id),
        }
    }

    fn show_data_type(&mut self, type_id: TypeId) {
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

        if let Some(methods) =
            self.interner.get_type_methods(&Type::DataType(shared_data_type.clone(), vec![]))
        {
            self.show_data_type_methods(methods);
        }
    }

    fn show_struct(&mut self, data_type: &DataType) {
        self.push_str("struct ");
        self.push_str(&data_type.name.to_string());
        self.show_generics(&data_type.generics);
        self.push_str(" {\n");
        self.increase_indent();
        for field in data_type.get_fields_as_written().unwrap() {
            self.write_indent();
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
        for variant in data_type.get_variants_as_written().unwrap() {
            self.write_indent();
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

    fn show_data_type_methods(&mut self, methods: &rustc_hash::FxHashMap<String, Methods>) {
        // First split methods by impl methods and trait impl methods
        let mut impl_methods = Vec::new();
        let mut trait_impl_methods = Vec::new();

        for (_, methods) in methods {
            impl_methods.extend(methods.direct.clone());
            trait_impl_methods.extend(methods.trait_impl_methods.clone());
        }

        // For impl methods, split them by the impl type. For example here we'll group
        // all of `Foo<i32>` methods in one bucket, all of `Foo<Field>` in another, and
        // all of `Foo<T>` in another one.
        let mut impl_methods_by_type: HashMap<Type, Vec<ImplMethod>> = HashMap::new();
        for method in impl_methods {
            impl_methods_by_type.entry(method.typ.clone()).or_default().push(method);
        }

        for (typ, methods) in impl_methods_by_type {
            self.push_str("\n\n");
            self.write_indent();
            self.show_impl(typ, methods);
        }
    }

    fn show_impl(&mut self, typ: Type, methods: Vec<ImplMethod>) {
        self.push_str("impl");

        let mut type_vars = HashSet::new();
        gather_named_type_vars(&typ, &mut type_vars);

        if !type_vars.is_empty() {
            self.push('<');
            for (index, name) in type_vars.iter().enumerate() {
                if index != 0 {
                    self.push_str(", ");
                }
                self.push_str(name);
            }
            self.push('>');
        }

        self.push(' ');
        self.show_type(&typ);
        self.push_str(" {\n");
        self.increase_indent();
        for (index, method) in methods.iter().enumerate() {
            if index != 0 {
                self.push_str("\n\n");
            }
            self.write_indent();
            self.show_function(method.method);
        }
        self.push('\n');
        self.decrease_indent();
        self.write_indent();
        self.push('}');
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

    fn show_trait(&mut self, trait_id: TraitId) {
        let trait_ = self.interner.get_trait(trait_id);

        self.push_str("trait ");
        self.push_str(&trait_.name.to_string());
        self.show_generics(&trait_.generics);

        if !trait_.trait_bounds.is_empty() {
            self.push_str(": ");
            for (index, trait_bound) in trait_.trait_bounds.iter().enumerate() {
                if index != 0 {
                    self.push_str(", ");
                }
                self.show_trait_bound(trait_bound);
            }
        }

        self.show_where_clause(&trait_.where_clause);
        self.push_str(" {\n");
        self.increase_indent();

        let mut printed_type_or_function = false;

        for associated_type in &trait_.associated_types {
            if printed_type_or_function {
                self.push_str("\n\n");
            }

            self.write_indent();
            self.push_str("type ");
            self.push_str(&associated_type.name);
            self.push_str(";");
            printed_type_or_function = true;
        }

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

        for (func_id, _location) in func_ids {
            if printed_type_or_function {
                self.push_str("\n\n");
            }

            self.write_indent();
            self.show_function(*func_id);
            printed_type_or_function = true;
        }

        self.push('\n');
        self.decrease_indent();
        self.write_indent();
        self.push('}');
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
        let name = &modifiers.name;

        if modifiers.is_unconstrained {
            self.push_str("unconstrained ");
        }
        if modifiers.is_comptime {
            self.push_str("comptime ");
        }

        self.push_str("fn ");
        self.push_str(name);

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
                self.show_type(return_type);
            }
        }

        self.show_where_clause(&func_meta.trait_constraints);

        let hir_function = self.interner.function(&func_id);
        if hir_function.try_as_expr().is_some() {
            let block = hir_function.block(self.interner);
            let block = HirExpression::Block(block);
            let block = block.to_display_ast(self.interner, func_meta.location);
            let block_str = block.to_string();
            let block_str = indent_lines(block_str, self.indent);
            self.push(' ');
            self.push_str(&block_str);
        } else {
            self.push(';');
        }
    }

    fn show_generics(&mut self, generics: &Generics) {
        self.show_generics_impl(
            generics, false, // only show names
        );
    }

    fn show_generics_impl(&mut self, generics: &Generics, only_show_names: bool) {
        if generics.is_empty() {
            return;
        }

        self.push('<');
        for (index, generic) in generics.iter().enumerate() {
            if index > 0 {
                self.push_str(", ");
            }

            if only_show_names {
                self.push_str(&generic.name);
            } else {
                match generic.kind() {
                    noirc_frontend::Kind::Any | noirc_frontend::Kind::Normal => {
                        self.push_str(&generic.name);
                    }
                    noirc_frontend::Kind::IntegerOrField | noirc_frontend::Kind::Integer => {
                        self.push_str("let ");
                        self.push_str(&generic.name);
                        self.push_str(": u32");
                    }
                    noirc_frontend::Kind::Numeric(typ) => {
                        self.push_str("let ");
                        self.push_str(&generic.name);
                        self.push_str(": ");
                        self.show_type(&typ);
                    }
                }
            }
        }
        self.push('>');
    }

    fn show_trait_generics(&mut self, generics: &TraitGenerics) {
        if generics.is_empty() {
            return;
        }

        let mut printed_type = false;

        self.push('<');

        for typ in &generics.ordered {
            if printed_type {
                self.push_str(", ");
            }

            self.show_type(typ);
            printed_type = true;
        }

        for named_type in &generics.named {
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
        self.push_str(&trait_.name.to_string());
        self.show_trait_generics(&bound.trait_generics);
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
            HirPattern::Tuple(..) | HirPattern::Struct(..) => {
                self.push('_');
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
            Value::String(string) => self.push_str(&format!("{:?}", string)),
            Value::FormatString(string, _typ) => {
                // Note: at this point the format string was already expanded so we can't recover the original
                // interpolation and this will result in a compile-error. But... the expanded code is meant
                // to be browsed, not compiled.
                self.push_str(&format!("f{:?}", string));
            }
            Value::CtString(string) => {
                let std = if self.crate_id.is_stdlib() { "std" } else { "crate" };
                self.push_str(&format!(
                    "{}::meta::ctstring::AsCtString::as_ctstring({:?})",
                    std, string
                ));
            }
            Value::Function(func_id, ..) => {
                // TODO: the name might need to be fully-qualified
                let name = self.interner.function_name(func_id);
                self.push_str(name);
            }
            Value::Tuple(values) => {
                self.push('(');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_value(value);
                }
                self.push(')');
            }
            Value::Struct(fields, typ) => {
                self.show_type(typ);
                if fields.is_empty() {
                    self.push_str(" {}");
                } else {
                    self.push_str(" {\n");
                    self.increase_indent();
                    for (name, value) in fields {
                        self.write_indent();
                        self.push_str(name);
                        self.push_str(": ");
                        self.show_value(value);
                        self.push_str(",\n");
                    }
                    self.decrease_indent();
                    self.write_indent();
                    self.push('}');
                }
            }
            Value::Enum(index, args, typ) => {
                let Type::DataType(data_type, generics) = typ else {
                    panic!("Expected typ to be a data type");
                };
                let data_type = data_type.borrow();

                // TODO: we might need to fully-qualify this enum
                self.push_str(&data_type.name.to_string());

                if !generics.is_empty() {
                    self.push_str("::<");
                    for (index, generic) in generics.iter().enumerate() {
                        if index != 0 {
                            self.push_str(", ");
                        }
                        self.show_type(generic);
                    }
                    self.push('>');
                }

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
            Value::Slice(values, _) => {
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
                self.push_str("quote { ");
                self.push_str(&tokens_to_string(tokens, self.interner));
                self.push_str(" }");
            }
            Value::Pointer(value, ..) => {
                self.show_value(&value.borrow());
            }
            Value::Closure(_)
            | Value::StructDefinition(_)
            | Value::TraitConstraint(..)
            | Value::TraitDefinition(_)
            | Value::TraitImpl(_)
            | Value::FunctionDefinition(_)
            | Value::ModuleDefinition(_)
            | Value::Type(_)
            | Value::Zeroed(_)
            | Value::Expr(_)
            | Value::TypedExpr(_)
            | Value::UnresolvedType(_) => {
                panic!("Theis value shouldn't be held by globals: {:?}", value)
            }
        }
    }

    fn show_type(&mut self, typ: &Type) {
        self.push_str(&typ.to_string());
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

    fn module_def_id_location(&self, module_def_id: ModuleDefId) -> Location {
        // We already have logic to go from a ReferenceId to a location, so we use that here
        let reference_id = match module_def_id {
            ModuleDefId::ModuleId(module_id) => ReferenceId::Module(module_id),
            ModuleDefId::FunctionId(func_id) => ReferenceId::Function(func_id),
            ModuleDefId::TypeId(type_id) => ReferenceId::Type(type_id),
            ModuleDefId::TypeAliasId(type_alias_id) => ReferenceId::Alias(type_alias_id),
            ModuleDefId::TraitId(trait_id) => ReferenceId::Trait(trait_id),
            ModuleDefId::GlobalId(global_id) => ReferenceId::Global(global_id),
        };
        self.interner.reference_location(reference_id)
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

fn indent_lines(string: String, indent: usize) -> String {
    if indent == 0 {
        return string;
    }

    let lines = string.lines();
    let lines_count = lines.clone().count();

    lines
        .enumerate()
        .map(|(index, line)| {
            if index == lines_count - 1 {
                format!("{}{}", "    ".repeat(indent), line)
            } else if index == 0 {
                format!("{}\n", line)
            } else {
                format!("{}{}\n", "    ".repeat(indent), line)
            }
        })
        .collect()
}

fn gather_named_type_vars(typ: &Type, type_vars: &mut HashSet<String>) {
    match typ {
        Type::Array(length, typ) => {
            gather_named_type_vars(length, type_vars);
            gather_named_type_vars(typ, type_vars);
        }
        Type::Slice(typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::FmtString(length, typ) => {
            gather_named_type_vars(&length, type_vars);
            gather_named_type_vars(&typ, type_vars);
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
                gather_named_type_vars(&typ, type_vars);
            }
            for named_type in &trait_generics.named {
                gather_named_type_vars(&named_type.typ, type_vars);
            }
        }
        Type::NamedGeneric(_, name) => {
            type_vars.insert(name.to_string());
        }
        Type::CheckedCast { from, to: _ } => {
            gather_named_type_vars(&from, type_vars);
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
        Type::Unit
        | Type::FieldElement
        | Type::Integer(..)
        | Type::Bool
        | Type::String(_)
        | Type::Quoted(_)
        | Type::Constant(..)
        | Type::TypeVariable(_)
        | Type::Error => (),
    }
}
