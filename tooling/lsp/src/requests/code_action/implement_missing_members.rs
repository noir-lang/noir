use std::collections::{BTreeMap, HashMap};

use lsp_types::TextEdit;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{NoirTraitImpl, TraitImplItemKind, UnresolvedTypeData},
    graph::CrateId,
    hir::{
        def_map::{CrateDefMap, ModuleId},
        type_check::generics::TraitGenerics,
    },
    hir_def::{function::FuncMeta, stmt::HirPattern, traits::Trait},
    macros_api::{ModuleDefId, NodeInterner},
    node_interner::ReferenceId,
    Kind, ResolvedGeneric, Type, TypeVariableKind,
};

use crate::{byte_span_to_range, modules::relative_module_id_path};

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn implement_missing_members(
        &mut self,
        noir_trait_impl: &NoirTraitImpl,
        span: Span,
    ) {
        if !self.includes_span(span) {
            return;
        }

        let location = Location::new(noir_trait_impl.trait_name.span(), self.file);
        let Some(ReferenceId::Trait(trait_id)) = self.interner.find_referenced(location) else {
            return;
        };

        let trait_ = self.interner.get_trait(trait_id);

        // Get all methods
        let mut method_ids = trait_.method_ids.clone();

        // Also get all associated types
        let mut associated_types = HashMap::new();
        for associated_type in &trait_.associated_types {
            associated_types.insert(associated_type.name.as_ref(), associated_type);
        }

        // Remove the ones that already are implemented
        for item in &noir_trait_impl.items {
            match &item.item.kind {
                TraitImplItemKind::Function(noir_function) => {
                    method_ids.remove(noir_function.name());
                }
                TraitImplItemKind::Constant(..) => (),
                TraitImplItemKind::Type { name, alias } => {
                    if let UnresolvedTypeData::Unspecified = alias.typ {
                        continue;
                    }
                    associated_types.remove(&name.0.contents);
                }
            }
        }

        // Also remove default methods
        for trait_function in &trait_.methods {
            if trait_function.default_impl.is_some() {
                method_ids.remove(&trait_function.name.0.contents);
            }
        }

        if method_ids.is_empty() && associated_types.is_empty() {
            return;
        }

        let bytes = self.source.as_bytes();
        let right_brace_index = span.end() as usize - 1;

        // Let's find out the indent
        let mut cursor = right_brace_index - 1;
        while cursor > 0 {
            let c = bytes[cursor] as char;
            if c == '\n' {
                break;
            }
            if !c.is_whitespace() {
                break;
            }
            cursor -= 1;
        }
        let cursor_char = bytes[cursor] as char;

        let indent = if cursor_char == '\n' { right_brace_index - cursor - 1 } else { 0 };
        let indent_string = " ".repeat(indent + 4);

        let index = cursor + 1;

        let Some(range) = byte_span_to_range(self.files, self.file, index..index) else {
            return;
        };

        let mut method_ids: Vec<_> = method_ids.iter().collect();
        method_ids.sort_by_key(|(name, _)| *name);

        let mut stubs = Vec::new();

        for (name, _) in associated_types {
            stubs.push(format!("{}type {};\n", indent_string, name));
        }

        for (name, func_id) in method_ids {
            let func_meta = self.interner.function_meta(func_id);

            let mut generator = MethodStubGenerator::new(
                name,
                func_meta,
                trait_,
                noir_trait_impl,
                self.interner,
                self.def_maps,
                self.module_id,
                indent + 4,
            );
            let stub = generator.generate();
            stubs.push(stub);
        }

        let mut new_text = stubs.join("\n");
        if cursor_char != '\n' {
            new_text.insert(0, '\n');
        }

        let title = "Implement missing members".to_string();
        let text_edit = TextEdit { range, new_text };
        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

struct MethodStubGenerator<'a> {
    name: &'a str,
    func_meta: &'a FuncMeta,
    trait_: &'a Trait,
    noir_trait_impl: &'a NoirTraitImpl,
    interner: &'a NodeInterner,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
    indent: usize,
    string: String,
}

impl<'a> MethodStubGenerator<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &'a str,
        func_meta: &'a FuncMeta,
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
            trait_,
            noir_trait_impl,
            interner,
            def_maps,
            module_id,
            indent,
            string: String::new(),
        }
    }

    fn generate(&mut self) -> String {
        let indent_string = " ".repeat(self.indent);

        self.string.push_str(&indent_string);
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
                let trait_ = self.interner.get_trait(constraint.trait_id);
                self.string.push_str(&trait_.name.0.contents);
                self.append_trait_generics(&constraint.trait_generics);
            }
        }

        self.string.push_str(" {\n");

        let body_indent_string = " ".repeat(self.indent + 4);
        self.string.push_str(&body_indent_string);
        self.string.push_str("panic(f\"Implement ");
        self.string.push_str(self.name);
        self.string.push_str("\")\n");
        self.string.push_str(&indent_string);
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

                let module_id = struct_type.id.module_id();
                let module_data = &self.def_maps[&module_id.krate].modules()[module_id.local_id.0];
                let parent_module_local_id = module_data.parent.unwrap();
                let parent_module_id =
                    ModuleId { krate: module_id.krate, local_id: parent_module_local_id };

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
            Type::TypeVariable(typevar, _) => {
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
            Type::NamedGeneric(typevar, _name, _kind) => {
                self.append_type(&Type::TypeVariable(typevar.clone(), TypeVariableKind::Normal));
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
            Type::Constant(_)
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
        match &generic.kind {
            Kind::Normal => self.string.push_str(&generic.name),
            Kind::Numeric(typ) => {
                self.string.push_str("let ");
                self.string.push_str(&generic.name);
                self.string.push_str(": ");
                self.append_type(typ);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_add_missing_impl_members_simple() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn foo(x: i32) -> i32;
    fn bar() {}
}

struct Foo {}

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
trait Trait {
    fn foo(x: i32) -> i32;
    fn bar() {}
}

struct Foo {}

impl Trait for Foo {
    fn foo(x: i32) -> i32 {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_multiple_with_self_type() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn bar(self) -> Self;
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
trait Trait {
    fn bar(self) -> Self;
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Trait for Foo {
    fn bar(self) -> Self {
        panic(f"Implement bar")
    }

    fn foo(x: i32) -> i32 {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_qualify_type() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;

impl Trait for Foo {
    fn foo(x: moo::Moo) {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_no_need_to_qualify_type() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;
use moo::Moo;

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;
use moo::Moo;

impl Trait for Foo {
    fn foo(x: Moo) {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_generics() {
        let title = "Implement missing members";

        let src = r#"
trait Bar {}

trait Trait<T> {
    fn foo<let N: u32, M>(x: T) -> [T; N] where M: Bar;
}

struct Foo {}

impl <U> Tra>|<it<[U]> for Foo {
}"#;

        let expected = r#"
trait Bar {}

trait Trait<T> {
    fn foo<let N: u32, M>(x: T) -> [T; N] where M: Bar;
}

struct Foo {}

impl <U> Trait<[U]> for Foo {
    fn foo<let N: u32, M>(x: [U]) -> [[U]; N] where M: Bar {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_associated_types() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem];
}

struct Foo {}

impl Trait>|< for Foo {
}"#;

        let expected = r#"
trait Trait {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem];
}

struct Foo {}

impl Trait for Foo {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem] {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_nested() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    trait Trait {
        fn foo();
        fn bar();
    }

    struct Foo {}

    impl Tra>|<it for Foo {
    }
}"#;

        let expected = r#"
mod moo {
    trait Trait {
        fn foo();
        fn bar();
    }

    struct Foo {}

    impl Trait for Foo {
        fn bar() {
            panic(f"Implement bar")
        }

        fn foo() {
            panic(f"Implement foo")
        }
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_inline() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn foo();
    fn bar();
}

struct Foo {}

impl Tra>|<it for Foo {}"#;

        let expected = r#"
trait Trait {
    fn foo();
    fn bar();
}

struct Foo {}

impl Trait for Foo {
    fn bar() {
        panic(f"Implement bar")
    }

    fn foo() {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }
}
