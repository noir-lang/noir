use std::{fs, path::PathBuf, str::FromStr};

use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_frontend::{
    ast::{
        Documented, FunctionDefinition, FunctionReturnType, ImportStatement, ItemVisibility,
        NoirFunction, NoirStruct, Pattern, TypeImpl, UnresolvedGeneric, UnresolvedTypeData,
    },
    parser::SortedModule,
    ParsedModule,
};

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let output_dir = current_dir.join("docs/docs/noir/standard_library");

    let stdlib_dir = current_dir.join("noir_stdlib");
    let toml_path = &stdlib_dir.join("Nargo.toml");
    let workspace = resolve_workspace_from_toml(
        toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .unwrap();
    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);

    let parsed_files = parse_all(&file_manager);
    for (file_id, (parsed_module, _)) in parsed_files {
        let path = file_manager.path(file_id).unwrap();
        let path = path.strip_prefix(&stdlib_dir.join("src")).unwrap();
        let path = path.as_os_str().to_string_lossy().to_string();
        let path = path.strip_suffix(".nr").unwrap();
        let mut segments: Vec<_> = path.split('/').collect();
        let last_segment = segments.last();
        if let Some(last_segment) = last_segment {
            if *last_segment == "mod" || *last_segment == "lib" {
                segments.pop();
            }
        }
        segments.insert(0, "std");
        let module_name = segments.join("::");
        let path = PathBuf::from_str(&segments.join("/")).unwrap().with_extension("md");

        generate_module(module_name, &output_dir.join(path), parsed_module);
    }
}

fn generate_module(module_name: String, path: &PathBuf, parsed_module: ParsedModule) {
    let sorted_module = parsed_module.into_sorted();

    let mut generator = DocGenerator::new();

    if module_name == "std" {
        generator.meta("std");
        generator.title(format!("Crate `{}`", &module_name));
    } else {
        generator.meta(module_name.split("::").last().unwrap());
        generator.title(format!("Module `{}`", &module_name));
    }
    generator.doc_comments(&sorted_module.inner_doc_comments);

    generator.public_exports(&sorted_module.imports);
    generator.noir_structs(&sorted_module);
    generator.type_impls(&sorted_module.impls);
    generator.noir_functions(&sorted_module.functions);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    fs::write(path, &generator.string).unwrap();
}

struct DocGenerator {
    pub string: String,
    nesting: usize,
}

impl DocGenerator {
    fn new() -> Self {
        Self { string: String::new(), nesting: 1 }
    }

    fn public_exports(&mut self, imports: &[ImportStatement]) {
        let public_imports: Vec<_> =
            imports.iter().filter(|import| import.visibility == ItemVisibility::Public).collect();
        if public_imports.is_empty() {
            return;
        }

        self.increase_nesting();
        self.title("Public exports");
        for import in public_imports {
            let import = import.to_string();
            let import = import.replace("crate::", "std::");
            self.string.push_str(" - `");
            self.string.push_str(&import);
            self.string.push_str("`\n");
        }

        self.decrease_nesting();
    }

    fn noir_structs(&mut self, sorted_module: &SortedModule) {
        self.increase_nesting();

        for documented_noir_struct in &sorted_module.types {
            self.noir_struct(documented_noir_struct);

            // Find all implementations by name
            let implementations = sorted_module.impls.iter().filter(|implementation| {
                if let UnresolvedTypeData::Named(path, _, _) = &implementation.object_type.typ {
                    path.last_name() == documented_noir_struct.item.name.0.contents
                } else {
                    false
                }
            });

            self.increase_nesting();
            self.title("Methods");
            self.increase_nesting();

            for implementation in implementations {
                for (documented_method, _span) in &implementation.methods {
                    if documented_method.item.def.visibility != ItemVisibility::Public {
                        return;
                    }

                    self.noir_function(documented_method);
                }
            }

            self.decrease_nesting();
            self.decrease_nesting();
        }

        self.decrease_nesting();
    }

    fn noir_struct(&mut self, documented_noir_struct: &Documented<NoirStruct>) {
        let noir_struct = &documented_noir_struct.item;
        let doc_comments = &documented_noir_struct.doc_comments;

        // Don't generate doc comments for undocumented structs
        // (we don't have visibility modifiers yet)
        if doc_comments.is_empty() {
            return;
        }

        self.title(format!("Struct `{}`", noir_struct_name(noir_struct)));
        self.doc_comments(doc_comments);
    }

    fn type_impls(&mut self, type_impls: &[TypeImpl]) {
        // Output impls for primitive types
        self.increase_nesting();
        for implementation in type_impls {
            if let UnresolvedTypeData::Named(..) = implementation.object_type.typ {
                continue;
            };
            self.type_impl(implementation);
        }
        self.decrease_nesting();
    }

    fn type_impl(&mut self, type_impl: &TypeImpl) {
        self.title(format!("`{}` methods", type_impl.object_type));

        self.increase_nesting();
        for (documented_noir_method, _span) in &type_impl.methods {
            self.noir_function(documented_noir_method);
        }
        self.decrease_nesting();
    }

    fn noir_functions(&mut self, documented_noir_functions: &[Documented<NoirFunction>]) {
        self.increase_nesting();
        for documented_noir_function in documented_noir_functions {
            if documented_noir_function.item.def.visibility != ItemVisibility::Public {
                continue;
            }

            self.noir_function(documented_noir_function);
        }
        self.decrease_nesting();
    }

    fn noir_function(&mut self, documented_noir_function: &Documented<NoirFunction>) {
        let function = &documented_noir_function.item;
        let doc_comments = &documented_noir_function.doc_comments;

        self.title(function.name());
        self.code(function_signature(&function.def));
        self.doc_comments(doc_comments);
    }

    fn increase_nesting(&mut self) {
        self.nesting += 1;
    }

    fn decrease_nesting(&mut self) {
        self.nesting -= 1;
    }

    fn meta(&mut self, title: impl Into<String>) {
        self.string.push_str("---\n");
        self.string.push_str("title: ");
        self.string.push_str(&title.into());
        self.string.push_str("\n---\n\n");
    }

    fn title(&mut self, title: impl Into<String>) {
        for _ in 0..self.nesting {
            self.string.push('#');
        }
        self.string.push(' ');
        self.string.push_str(&title.into());
        self.string.push('\n');
        self.string.push('\n');
    }

    fn doc_comments(&mut self, doc_comments: &[String]) {
        if doc_comments.is_empty() {
            return;
        }

        for doc_comment in doc_comments {
            let line = doc_comment.trim();

            // The doc viewer doesn't know about Noir, but it knows about Rust
            // so let's use that to get syntax highlighting.
            if line == "```noir" {
                self.string.push_str("```rust");
            } else {
                self.string.push_str(line);
            }

            self.string.push('\n');
        }
        self.string.push('\n');
    }

    fn code(&mut self, code: impl Into<String>) {
        self.string.push_str("```rust\n");
        self.string.push_str(&code.into());
        self.string.push_str("\n```\n\n");
    }
}

fn noir_struct_name(noir_struct: &NoirStruct) -> String {
    let mut string = String::new();
    string.push_str(&noir_struct.name.0.contents);
    append_generics(&noir_struct.generics, &mut string);
    string
}

fn function_signature(func: &FunctionDefinition) -> String {
    let mut string = String::new();
    string.push_str("fn ");
    string.push_str(&func.name.0.contents);
    append_generics(&func.generics, &mut string);
    string.push('(');
    for (index, param) in func.parameters.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }
        string.push_str(&param.pattern.to_string());
        if !pattern_is_self(&param.pattern) {
            string.push_str(": ");
            string.push_str(&param.typ.to_string());
        }
    }

    string.push(')');
    if let FunctionReturnType::Ty(typ) = &func.return_type {
        string.push_str(" -> ");
        string.push_str(&typ.to_string());
    }

    if !func.where_clause.is_empty() {
        string.push('\n');
        string.push_str("    where ");
        for (index, where_clause) in func.where_clause.iter().enumerate() {
            if index > 0 {
                string.push_str(",\n          ");
            }
            string.push_str(&where_clause.to_string());
        }
    }

    string
}

fn append_generics(generics: &[UnresolvedGeneric], string: &mut String) {
    if generics.is_empty() {
        return;
    }

    string.push('<');
    for (index, generic) in generics.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }
        string.push_str(&generic.to_string());
    }
    string.push('>');
}

fn pattern_is_self(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Identifier(ident) => ident.0.contents == "self" || ident.0.contents == "_self",
        Pattern::Mutable(pattern, _, _) => pattern_is_self(pattern),
        Pattern::Tuple(_, _) | Pattern::Struct(_, _, _) | Pattern::Interned(_, _) => false,
    }
}
