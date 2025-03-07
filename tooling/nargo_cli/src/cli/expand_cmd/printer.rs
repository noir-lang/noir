use noirc_frontend::{
    DataType, Generics, Type,
    ast::{ItemVisibility, Visibility},
    hir::def_map::ModuleDefId,
    hir_def::{expr::HirExpression, stmt::HirPattern},
    node_interner::{FuncId, NodeInterner, TypeId},
};

pub(super) struct Printer<'interner, 'string> {
    interner: &'interner NodeInterner,
    string: &'string mut String,
}

impl<'interner, 'string> Printer<'interner, 'string> {
    pub(super) fn new(interner: &'interner NodeInterner, string: &'string mut String) -> Self {
        Self { interner, string }
    }

    pub(super) fn show_module_def_id(
        &mut self,
        module_def_id: ModuleDefId,
        visibility: ItemVisibility,
    ) {
        if visibility != ItemVisibility::Private {
            self.push_str(&visibility.to_string());
            self.push(' ');
        };

        match module_def_id {
            ModuleDefId::ModuleId(module_id) => todo!("Show modules"),
            ModuleDefId::TypeId(type_id) => self.show_type(type_id),
            ModuleDefId::TypeAliasId(type_alias_id) => todo!("Show type aliases"),
            ModuleDefId::TraitId(trait_id) => todo!("Show traits"),
            ModuleDefId::GlobalId(global_id) => todo!("Show globals"),
            ModuleDefId::FunctionId(func_id) => self.show_function(func_id),
        }
        self.push_str("\n\n");
    }

    fn show_type(&mut self, type_id: TypeId) {
        let data_type = self.interner.get_type(type_id);
        let data_type = data_type.borrow();
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
        for field in data_type.get_fields_as_written().unwrap() {
            self.push_str("    ");
            self.push_str(&field.name.to_string());
            self.push_str(": ");
            self.push_str(&field.typ.to_string());
            self.push_str(",\n");
        }
        self.push('}');
    }

    fn show_enum(&mut self, data_type: &DataType) {
        todo!("Show enums")
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
                self.push_str(&format!("{}", typ));
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
                self.push_str(&format!("{}", return_type));
            }
        }

        self.push(' ');

        let hir_function = self.interner.function(&func_id);
        let block = hir_function.block(self.interner);
        let block = HirExpression::Block(block);
        let block = block.to_display_ast(self.interner, func_meta.location);
        self.push_str(&block.to_string());
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
                        self.push_str(&typ.to_string());
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

    fn push_str(&mut self, str: &str) {
        self.string.push_str(str);
    }

    fn push(&mut self, char: char) {
        self.string.push(char);
    }
}
