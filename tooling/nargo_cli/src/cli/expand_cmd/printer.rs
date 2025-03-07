use noirc_frontend::{
    Generics, Type,
    ast::{ItemVisibility, Visibility},
    hir::def_map::ModuleDefId,
    hir_def::{expr::HirExpression, stmt::HirPattern},
    node_interner::{FuncId, NodeInterner},
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
            self.string.push_str(&visibility.to_string());
            self.string.push(' ');
        };

        match module_def_id {
            ModuleDefId::ModuleId(module_id) => todo!("Show modules"),
            ModuleDefId::FunctionId(func_id) => self.show_function(func_id),
            ModuleDefId::TypeId(_) => todo!("Show types"),
            ModuleDefId::TypeAliasId(type_alias_id) => todo!("Show type aliases"),
            ModuleDefId::TraitId(trait_id) => todo!("Show traits"),
            ModuleDefId::GlobalId(global_id) => todo!("Show globals"),
        }
        self.string.push_str("\n\n");
    }

    fn show_function(&mut self, func_id: FuncId) {
        let modifiers = self.interner.function_modifiers(&func_id);
        let func_meta = self.interner.function_meta(&func_id);
        let name = &modifiers.name;

        if modifiers.is_unconstrained {
            self.string.push_str("unconstrained ");
        }
        if modifiers.is_comptime {
            self.string.push_str("comptime ");
        }

        self.string.push_str("fn ");
        self.string.push_str(name);

        self.show_generics(&func_meta.direct_generics);

        self.string.push('(');
        let parameters = &func_meta.parameters;
        for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
            let is_self = self.pattern_is_self(pattern);

            // `&mut self` is represented as a mutable reference type, not as a mutable pattern
            if is_self && matches!(typ, Type::Reference(..)) {
                self.string.push_str("&mut ");
            }

            self.show_pattern(pattern);

            // Don't add type for `self` param
            if !is_self {
                self.string.push_str(": ");
                if matches!(visibility, Visibility::Public) {
                    self.string.push_str("pub ");
                }
                self.string.push_str(&format!("{}", typ));
            }

            if index != parameters.len() - 1 {
                self.string.push_str(", ");
            }
        }
        self.string.push(')');

        let return_type = func_meta.return_type();
        match return_type {
            Type::Unit => (),
            _ => {
                self.string.push_str(" -> ");
                self.string.push_str(&format!("{}", return_type));
            }
        }

        self.string.push(' ');

        let hir_function = self.interner.function(&func_id);
        let block = hir_function.block(self.interner);
        let block = HirExpression::Block(block);
        let block = block.to_display_ast(self.interner, func_meta.location);
        self.string.push_str(&block.to_string());
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

        self.string.push('<');
        for (index, generic) in generics.iter().enumerate() {
            if index > 0 {
                self.string.push_str(", ");
            }

            if only_show_names {
                self.string.push_str(&generic.name);
            } else {
                match generic.kind() {
                    noirc_frontend::Kind::Any | noirc_frontend::Kind::Normal => {
                        self.string.push_str(&generic.name);
                    }
                    noirc_frontend::Kind::IntegerOrField | noirc_frontend::Kind::Integer => {
                        self.string.push_str("let ");
                        self.string.push_str(&generic.name);
                        self.string.push_str(": u32");
                    }
                    noirc_frontend::Kind::Numeric(typ) => {
                        self.string.push_str("let ");
                        self.string.push_str(&generic.name);
                        self.string.push_str(": ");
                        self.string.push_str(&typ.to_string());
                    }
                }
            }
        }
        self.string.push('>');
    }

    fn show_pattern(&mut self, pattern: &HirPattern) {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                self.string.push_str(&definition.name);
            }
            HirPattern::Mutable(pattern, _) => {
                self.string.push_str("mut ");
                self.show_pattern(pattern);
            }
            HirPattern::Tuple(..) | HirPattern::Struct(..) => {
                self.string.push('_');
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
}
