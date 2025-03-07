use noirc_errors::Location;
use noirc_frontend::{
    DataType, Generics, Type,
    ast::{ItemVisibility, Visibility},
    hir::{
        comptime::Value,
        def_map::{CrateDefMap, ModuleDefId, ModuleId},
    },
    hir_def::{
        expr::HirExpression,
        stmt::{HirLetStatement, HirPattern},
    },
    node_interner::{
        FuncId, GlobalId, GlobalValue, NodeInterner, ReferenceId, TypeAliasId, TypeId,
    },
};

pub(super) struct Printer<'interner, 'def_map, 'string> {
    interner: &'interner NodeInterner,
    def_map: &'def_map CrateDefMap,
    string: &'string mut String,
    indent: usize,
}

impl<'interner, 'def_map, 'string> Printer<'interner, 'def_map, 'string> {
    pub(super) fn new(
        interner: &'interner NodeInterner,
        def_map: &'def_map CrateDefMap,
        string: &'string mut String,
    ) -> Self {
        Self { interner, def_map, string, indent: 0 }
    }

    pub(super) fn show_module(&mut self, module_id: ModuleId) {
        let attributes = self.interner.try_module_attributes(&module_id);
        let name = attributes.map(|attributes| &attributes.name);

        if let Some(name) = name {
            self.write_indent();
            self.push_str("mod ");
            self.push_str(name);
            self.push_str(" {");
            self.increase_indent();
        }

        let module_data = &self.def_map.modules()[module_id.local_id.0];
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
            ModuleDefId::TraitId(trait_id) => todo!("Show traits"),
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
        self.push('}');
    }

    fn show_enum(&mut self, data_type: &DataType) {
        todo!("Show enums")
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

        self.push(' ');

        let hir_function = self.interner.function(&func_id);
        let block = hir_function.block(self.interner);
        let block = HirExpression::Block(block);
        let block = block.to_display_ast(self.interner, func_meta.location);
        let block_str = block.to_string();
        let block_str = indent_lines(block_str, self.indent);
        self.push_str(&block_str);
    }

    fn show_generics(&mut self, generics: &Generics) {
        self.show_generics_impl(
            generics, false, // only show names
        );
    }

    fn show_generic_names(&mut self, generics: &Generics) {
        self.show_generics_impl(
            generics, true, // only show names
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
            Value::FormatString(_, _) => todo!("Show format string"),
            Value::CtString(_) => todo!("Show CtString"),
            Value::Function(func_id, _, hash_map) => todo!("Show function"),
            Value::Tuple(values) => todo!("Show tuple"),
            Value::Struct(hash_map, _) => todo!("Show struct"),
            Value::Enum(_, values, _) => todo!("Show enum"),
            Value::Array(vector, _) => todo!("Show array"),
            Value::Slice(vector, _) => todo!("Show slice"),
            Value::Quoted(located_tokens) => todo!("Show quoted"),
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
