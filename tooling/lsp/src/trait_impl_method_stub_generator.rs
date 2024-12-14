use std::collections::BTreeMap;

use noirc_frontend::{
    ast::NoirTraitImpl,
    graph::CrateId,
    hir::def_map::ModuleDefId,
    hir::{
        def_map::{CrateDefMap, ModuleId},
        type_check::generics::TraitGenerics,
    },
    hir_def::{function::FuncMeta, stmt::HirPattern, traits::Trait},
    node_interner::{FunctionModifiers, NodeInterner, ReferenceId},
    Kind, ResolvedGeneric, Type,
};

use crate::modules::relative_module_id_path;

pub(crate) struct TraitImplMethodStubGenerator<'a> {
    name: &'a str,
    func_meta: &'a FuncMeta,
    modifiers: &'a FunctionModifiers,
    trait_: &'a Trait,
    noir_trait_impl: &'a NoirTraitImpl,
    interner: &'a NodeInterner,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
    indent: usize,
    body: Option<String>,
    string: String,
}

impl<'a> TraitImplMethodStubGenerator<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: &'a str,
        func_meta: &'a FuncMeta,
        modifiers: &'a FunctionModifiers,
        trait_: &'a Trait,
        noir_trait_impl: &'a NoirTraitImpl,
        interner: &'a NodeInterner,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        module_id: ModuleId,
        indent: usize,
    ) -> Self {
        Self {
            name,
            func_meta,
            modifiers,
            trait_,
            noir_trait_impl,
            interner,
            def_maps,
            module_id,
            indent,
            body: None,
            string: String::new(),
        }
    }

    /// Sets the body to include in the stub method. By default an empty body will be generated.
    pub(crate) fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }

    pub(crate) fn generate(&mut self) -> String {
        let indent_string = " ".repeat(self.indent);

        self.string.push_str(&indent_string);
        if self.modifiers.is_unconstrained {
            self.string.push_str("unconstrained ");
        }
        self.string.push_str("fn ");
        self.string.push_str(self.name);
        self.append_resolved_generics(&self.func_meta.direct_generics);
        self.string.push('(');
        for (index, (pattern, typ, _visibility)) in self.func_meta.parameters.iter().enumerate() {
            if index > 0 {
                self.string.push_str(", ");
            }
            if self.append_pattern(pattern) {
                self.string.push_str(": ");
                self.append_type(typ);
            }
        }
        self.string.push(')');

        let return_type = self.func_meta.return_type();
        if return_type != &Type::Unit {
            self.string.push_str(" -> ");
            self.append_type(return_type);
        }

        if !self.func_meta.trait_constraints.is_empty() {
            self.string.push_str(" where ");
            for (index, constraint) in self.func_meta.trait_constraints.iter().enumerate() {
                if index > 0 {
                    self.string.push_str(", ");
                }
                self.append_type(&constraint.typ);
                self.string.push_str(": ");
                let trait_ = self.interner.get_trait(constraint.trait_bound.trait_id);
                self.string.push_str(&trait_.name.0.contents);
                self.append_trait_generics(&constraint.trait_bound.trait_generics);
            }
        }

        self.string.push_str(" {\n");

        if let Some(body) = &self.body {
            let body_indent_string = " ".repeat(self.indent + 4);
            self.string.push_str(&body_indent_string);
            self.string.push_str(body);
            self.string.push('\n');
            self.string.push_str(&indent_string);
        }

        self.string.push_str("}\n");
        std::mem::take(&mut self.string)
    }

    /// Appends a pattern and returns true if this was not the self type
    fn append_pattern(&mut self, pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Identifier(hir_ident) => {
                let definition = self.interner.definition(hir_ident.id);
                self.string.push_str(&definition.name);
                &definition.name != "self"
            }
            HirPattern::Mutable(pattern, _) => {
                self.string.push_str("mut ");
                self.append_pattern(pattern)
            }
            HirPattern::Tuple(patterns, _) => {
                self.string.push('(');
                for (index, pattern) in patterns.iter().enumerate() {
                    if index > 0 {
                        self.string.push_str(", ");
                    }
                    self.append_pattern(pattern);
                }
                self.string.push(')');
                true
            }
            HirPattern::Struct(typ, patterns, _) => {
                self.append_type(typ);
                self.string.push_str(" { ");
                for (index, (name, _pattern)) in patterns.iter().enumerate() {
                    if index > 0 {
                        self.string.push_str(", ");
                    }
                    self.string.push_str(&name.0.contents);
                }
                self.string.push_str(" }");
                true
            }
        }
    }

    fn append_type(&mut self, typ: &Type) {
        match typ {
            Type::FieldElement => self.string.push_str("Field"),
            Type::Array(n, e) => {
                self.string.push('[');
                self.append_type(e);
                self.string.push_str("; ");
                self.append_type(n);
                self.string.push(']');
            }
            Type::Slice(typ) => {
                self.string.push('[');
                self.append_type(typ);
                self.string.push(']');
            }
            Type::Tuple(types) => {
                self.string.push('(');
                for (index, typ) in types.iter().enumerate() {
                    if index > 0 {
                        self.string.push_str(", ");
                    }
                    self.append_type(typ);
                }
                self.string.push(')');
            }
            Type::Struct(struct_type, generics) => {
                let struct_type = struct_type.borrow();

                let current_module_data =
                    &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];

                // Check if the struct type is already imported/visible in this module
                let per_ns = current_module_data.find_name(&struct_type.name);
                if let Some((module_def_id, _, _)) = per_ns.types {
                    if module_def_id == ModuleDefId::TypeId(struct_type.id) {
                        self.string.push_str(&struct_type.name.0.contents);
                        self.append_generics(generics);
                        return;
                    }
                }

                let parent_module_id = struct_type.id.parent_module_id(self.def_maps);
                let current_module_parent_id = current_module_data
                    .parent
                    .map(|parent| ModuleId { krate: self.module_id.krate, local_id: parent });

                let relative_path = relative_module_id_path(
                    parent_module_id,
                    &self.module_id,
                    current_module_parent_id,
                    self.interner,
                );

                if !relative_path.is_empty() {
                    self.string.push_str(&relative_path);
                    self.string.push_str("::");
                }
                self.string.push_str(&struct_type.name.0.contents);
                self.append_generics(generics);
            }
            Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();

                let current_module_data =
                    &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];

                // Check if the alias type is already imported/visible in this module
                let per_ns = current_module_data.find_name(&type_alias.name);
                if let Some((module_def_id, _, _)) = per_ns.types {
                    if module_def_id == ModuleDefId::TypeAliasId(type_alias.id) {
                        self.string.push_str(&type_alias.name.0.contents);
                        self.append_generics(generics);
                        return;
                    }
                }

                let parent_module_id =
                    self.interner.reference_module(ReferenceId::Alias(type_alias.id)).unwrap();

                let current_module_parent_id = current_module_data
                    .parent
                    .map(|parent| ModuleId { krate: self.module_id.krate, local_id: parent });

                let relative_path = relative_module_id_path(
                    *parent_module_id,
                    &self.module_id,
                    current_module_parent_id,
                    self.interner,
                );

                if !relative_path.is_empty() {
                    self.string.push_str(&relative_path);
                    self.string.push_str("::");
                }
                self.string.push_str(&type_alias.name.0.contents);
                self.append_generics(generics);
            }
            Type::TraitAsType(trait_id, _, trait_generics) => {
                let trait_ = self.interner.get_trait(*trait_id);

                let current_module_data =
                    &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];

                // Check if the trait type is already imported/visible in this module
                let per_ns = current_module_data.find_name(&trait_.name);
                if let Some((module_def_id, _, _)) = per_ns.types {
                    if module_def_id == ModuleDefId::TraitId(*trait_id) {
                        self.string.push_str(&trait_.name.0.contents);
                        self.append_trait_generics(trait_generics);
                        return;
                    }
                }

                let parent_module_id =
                    self.interner.reference_module(ReferenceId::Trait(*trait_id)).unwrap();

                let current_module_parent_id = current_module_data
                    .parent
                    .map(|parent| ModuleId { krate: self.module_id.krate, local_id: parent });

                let relative_path = relative_module_id_path(
                    *parent_module_id,
                    &self.module_id,
                    current_module_parent_id,
                    self.interner,
                );

                if !relative_path.is_empty() {
                    self.string.push_str(&relative_path);
                    self.string.push_str("::");
                }
                self.string.push_str(&trait_.name.0.contents);
                self.append_trait_generics(trait_generics);
            }
            Type::TypeVariable(typevar) => {
                if typevar.id() == self.trait_.self_type_typevar.id() {
                    self.string.push_str("Self");
                    return;
                }

                let generics = &self.trait_.generics;
                if let Some(index) =
                    generics.iter().position(|generic| generic.type_var.id() == typevar.id())
                {
                    if let Some(typ) = self.noir_trait_impl.trait_generics.ordered_args.get(index) {
                        self.string.push_str(&typ.to_string());
                        return;
                    }
                }

                for associated_type in &self.trait_.associated_types {
                    if typevar.id() == associated_type.type_var.id() {
                        self.string.push_str("Self::");
                        self.string.push_str(&associated_type.name);
                        return;
                    }
                }

                for generic in &self.func_meta.direct_generics {
                    if typevar.id() == generic.type_var.id() {
                        self.string.push_str(&generic.name);
                        return;
                    }
                }

                self.string.push_str("error");
            }
            Type::NamedGeneric(typevar, _name) => {
                self.append_type(&Type::TypeVariable(typevar.clone()));
            }
            Type::Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    self.string.push_str("unconstrained ");
                }
                self.string.push_str("fn");

                if let Type::Unit = **env {
                } else {
                    self.string.push('[');
                    self.append_type(env);
                    self.string.push(']');
                }

                self.string.push('(');
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        self.string.push_str(", ");
                    }
                    self.append_type(arg);
                }
                self.string.push(')');

                if let Type::Unit = **ret {
                } else {
                    self.string.push_str(" -> ");
                    self.append_type(ret);
                }
            }
            Type::MutableReference(typ) => {
                self.string.push_str("&mut ");
                self.append_type(typ);
            }
            Type::Forall(_, _) => {
                panic!("Shouldn't get a Type::Forall");
            }
            Type::InfixExpr(left, op, right) => {
                self.append_type(left);
                self.string.push(' ');
                self.string.push_str(&op.to_string());
                self.string.push(' ');
                self.append_type(right);
            }
            Type::CheckedCast { to, .. } => self.append_type(to),
            Type::Constant(..)
            | Type::Integer(_, _)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Quoted(_)
            | Type::Error => self.string.push_str(&typ.to_string()),
        }
    }

    fn append_generics(&mut self, generics: &[Type]) {
        if generics.is_empty() {
            return;
        }

        self.string.push('<');
        for (index, typ) in generics.iter().enumerate() {
            if index > 0 {
                self.string.push_str(", ");
            }
            self.append_type(typ);
        }
        self.string.push('>');
    }

    fn append_trait_generics(&mut self, generics: &TraitGenerics) {
        if generics.named.is_empty() && generics.ordered.is_empty() {
            return;
        }

        let mut index = 0;

        self.string.push('<');
        for generic in &generics.ordered {
            if index > 0 {
                self.string.push_str(", ");
            }
            self.append_type(generic);
            index += 1;
        }
        for named_type in &generics.named {
            if index > 0 {
                self.string.push_str(", ");
            }
            self.string.push_str(&named_type.name.0.contents);
            self.string.push_str(" = ");
            self.append_type(&named_type.typ);
            index += 1;
        }
        self.string.push('>');
    }

    fn append_resolved_generics(&mut self, generics: &[ResolvedGeneric]) {
        if generics.is_empty() {
            return;
        }

        self.string.push('<');
        for (index, generic) in self.func_meta.direct_generics.iter().enumerate() {
            if index > 0 {
                self.string.push_str(", ");
            }
            self.append_resolved_generic(generic);
        }
        self.string.push('>');
    }

    fn append_resolved_generic(&mut self, generic: &ResolvedGeneric) {
        match &generic.kind() {
            Kind::Any | Kind::Normal | Kind::Integer | Kind::IntegerOrField => {
                self.string.push_str(&generic.name);
            }
            Kind::Numeric(ref typ) => {
                self.string.push_str("let ");
                self.string.push_str(&generic.name);
                self.string.push_str(": ");
                self.append_type(typ);
            }
        }
    }
}
