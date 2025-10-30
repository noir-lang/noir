use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use crate::items::{
    Crate, Crates, Function, Generic, Global, Impl, Item, Module, Struct, StructField, Trait,
    TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias,
};

pub fn to_html(crates: &Crates) -> Vec<(PathBuf, String)> {
    let mut renderer = HTMLRenderer::new(crates);
    renderer.render_crates(crates);
    renderer.files
}

struct HTMLRenderer {
    output: String,

    files: Vec<(PathBuf, String)>,

    /// Maps item IDs to strings so that HTML anchors have meaningful names.
    id_to_string: HashMap<usize, String>,
}

impl HTMLRenderer {
    fn new(crates: &Crates) -> Self {
        let id_to_string = compute_id_to_strings(crates);
        Self { output: String::new(), files: Vec::new(), id_to_string }
    }

    fn render_crates(&mut self, crates: &Crates) {
        self.render_index(crates);

        for krate in &crates.crates {
            self.render_crate(krate);
        }
    }

    fn render_index(&mut self, crates: &Crates) {
        self.html_start(&format!("{} documentation", crates.name));
        self.h1(&format!("{} documentation", crates.name));
        self.h2("Crates");
        self.output.push_str("<ul>\n");
        for krate in &crates.crates {
            self.output.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                path_to_string(&crate_path(krate)),
                krate.name
            ));
        }
        self.output.push_str("</ul>\n");
        self.html_end();
        self.push_file(index_path());
    }

    fn render_crate(&mut self, krate: &Crate) {
        self.html_start(&format!("Crate {}", krate.name));
        self.h1(&format!("Crate {}", krate.name));

        for module in &krate.modules {
            if module.name.is_empty() {
                self.render_comments(&module.comments, 1);
                self.render_items(krate, module);
                break;
            }
        }

        if krate.modules.iter().any(|module| !module.name.is_empty()) {
            self.h2("Modules");
            self.output.push_str("<ul>\n");
            for module in &krate.modules {
                if module.name.is_empty() {
                    continue;
                }
                self.output.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    path_to_string(&module_in_crate_path(module)),
                    module.name
                ));
            }
            self.output.push_str("</ul>\n");
        }

        self.html_end();
        self.push_file(crate_path(krate));

        for module in &krate.modules {
            self.render_module(krate, module);
        }
    }

    fn render_module(&mut self, krate: &Crate, module: &Module) {
        self.html_start(&format!("Module {}", module.name));
        self.h1(&format!("Module {}", module.name));
        self.render_comments(&module.comments, 2);
        self.render_items(krate, module);
        self.html_end();
        self.push_file(module_path(krate, module));
    }

    fn render_items(&mut self, krate: &Crate, module: &Module) {
        self.render_structs(krate, module, &module.items);
        self.render_traits(krate, module, &module.items);
        self.render_type_aliases(krate, module, &module.items);
        self.render_globals(krate, module, &module.items);
        self.render_functions(krate, module, &module.items);
    }

    fn render_structs(&mut self, krate: &Crate, module: &Module, items: &[Item]) {
        let structs = items
            .iter()
            .filter_map(|item| if let Item::Struct(struct_) = item { Some(struct_) } else { None })
            .collect::<Vec<_>>();
        if structs.is_empty() {
            return;
        }

        self.h2("Structs");
        self.output.push_str("<ul>");
        for struct_ in structs {
            self.output.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                path_to_string(&struct_path(krate, module, struct_)),
                struct_.name
            ));
        }
        self.output.push_str("</ul>");
    }

    fn render_traits(&mut self, krate: &Crate, module: &Module, items: &[Item]) {
        let traits = items
            .iter()
            .filter_map(|item| if let Item::Trait(trait_) = item { Some(trait_) } else { None })
            .collect::<Vec<_>>();
        if traits.is_empty() {
            return;
        }

        self.h2("Traits");
        self.output.push_str("<ul>");
        for trait_ in traits {
            self.output.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                path_to_string(&trait_path(krate, module, trait_)),
                trait_.name
            ));
        }
        self.output.push_str("</ul>");
    }

    fn render_struct(&mut self, struct_: &Struct) {
        self.anchor(struct_.id);
        self.h3(&format!("Struct `{}`", struct_.name));
        self.render_struct_code(struct_);
        self.render_comments(&struct_.comments, 3);
        self.render_struct_fields(&struct_.fields);
        self.render_impls(&struct_.impls);
        self.render_trait_impls(&struct_.trait_impls, "Trait implementations");
    }

    fn render_struct_code(&mut self, struct_: &Struct) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub struct {}", struct_.name));
        self.render_generics(&struct_.generics);
        if struct_.fields.is_empty() {
            if struct_.has_private_fields {
                self.output.push_str("\n{ /* private fields */ }\n");
            } else {
                self.output.push_str(" {}\n");
            }
        } else {
            self.output.push_str(" {\n");
            for field in &struct_.fields {
                self.output.push_str(&format!("    pub {}: ", field.name,));
                self.render_type(&field.r#type);
                self.output.push_str(",\n");
            }
            if struct_.has_private_fields {
                self.output.push_str("    /* private fields */\n");
            }
            self.output.push_str("}\n");
        }
        self.output.push_str("</code></pre>\n");
    }

    fn render_struct_fields(&mut self, fields: &[StructField]) {
        if fields.is_empty() {
            return;
        }

        self.h4("Fields");

        for field in fields {
            self.output.push_str("##### ");
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.render_type(&field.r#type);
            self.output.push_str("\n\n");
            self.render_comments(&field.comments, 5);
        }
    }

    fn render_impls(&mut self, impls: &[Impl]) {
        if impls.is_empty() {
            return;
        }

        self.h4("Implementations");

        for impl_ in impls {
            self.render_impl(impl_);
        }
    }

    fn render_impl(&mut self, impl_: &Impl) {
        self.output.push_str("<h5><pre><code>");
        self.output.push_str("impl");
        self.render_generics(&impl_.generics);
        self.output.push(' ');
        self.render_type(&impl_.r#type);
        self.output.push_str("</code></pre></h5>\n\n");
        self.render_methods(&impl_.methods, 5);
    }

    fn render_trait_impls(&mut self, trait_impls: &[TraitImpl], title: &str) {
        if trait_impls.is_empty() {
            return;
        }

        self.h4(title);

        for trait_impl in trait_impls {
            self.render_trait_impl(trait_impl);
        }
    }

    fn render_trait_impl(&mut self, trait_impl: &TraitImpl) {
        self.output.push_str("<h5><pre><code>");
        self.output.push_str("impl");
        self.render_generics(&trait_impl.generics);
        self.output.push(' ');
        self.render_id_reference(trait_impl.trait_id, &trait_impl.trait_name);
        self.render_generic_types(&trait_impl.trait_generics);
        self.output.push_str(" for ");
        self.render_type(&trait_impl.r#type);
        self.render_where_clause(&trait_impl.where_clause);
        self.output.push_str("</code></pre></h5>\n\n");
    }

    fn render_trait(&mut self, trait_: &Trait) {
        self.anchor(trait_.id);
        self.h3(&format!("Trait `{}`", trait_.name));
        self.render_trait_code(trait_);
        self.render_comments(&trait_.comments, 3);
        self.render_trait_methods(&trait_.methods);
        self.render_trait_impls(&trait_.trait_impls, "Implementors");
    }

    fn render_trait_code(&mut self, trait_: &Trait) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub trait {}", trait_.name));
        self.render_generics(&trait_.generics);
        if !trait_.bounds.is_empty() {
            self.output.push_str(":\n");
            for (index, bound) in trait_.bounds.iter().enumerate() {
                if index > 0 {
                    self.output.push('\n');
                }
                if index > 0 {
                    self.output.push_str("    + ");
                }
                self.render_trait_bound(bound);
            }
        }
        self.render_where_clause(&trait_.where_clause);
        self.output.push_str(" {\n}");
        self.output.push_str("</code></pre>\n\n");
    }

    fn render_trait_methods(&mut self, methods: &[Function]) {
        if methods.is_empty() {
            return;
        }

        self.h4("Methods");
        self.render_methods(methods, 4);
    }

    fn render_type_aliases(&mut self, krate: &Crate, module: &Module, items: &[Item]) {
        for item in items {
            if let Item::TypeAlias(alias) = item {
                self.render_type_alias(alias);
            }
        }
    }

    fn render_type_alias(&mut self, alias: &TypeAlias) {
        self.anchor(alias.id);
        self.h3(&format!("Type alias `{}`", alias.name));
        self.render_type_alias_code(alias);
        self.render_comments(&alias.comments, 3);
    }

    fn render_type_alias_code(&mut self, alias: &TypeAlias) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub type {}", alias.name));
        self.render_generics(&alias.generics);
        self.output.push_str(" = ");
        self.render_type(&alias.r#type);
        self.output.push(';');
        self.output.push_str("</code></pre>\n\n");
    }

    fn render_globals(&mut self, krate: &Crate, module: &Module, items: &[Item]) {
        for item in items {
            if let Item::Global(global) = item {
                self.render_global(global);
            }
        }
    }

    fn render_global(&mut self, global: &Global) {
        self.h3(&format!("Global `{}`", global.name));
        self.render_global_code(global);
        self.render_comments(&global.comments, 3);
    }

    fn render_global_code(&mut self, global: &Global) {
        self.output.push_str("<pre><code>");
        self.output.push_str("pub ");
        if global.mutable {
            self.output.push_str("mut ");
        }
        if global.comptime {
            self.output.push_str("comptime ");
        }
        self.output.push_str("global ");
        self.output.push_str(&global.name);
        self.output.push_str(": ");
        self.render_type(&global.r#type);
        self.output.push(';');
        self.output.push_str("</code></pre>\n\n");
    }

    fn render_functions(&mut self, krate: &Crate, module: &Module, items: &[Item]) {
        for item in items {
            if let Item::Function(function) = item {
                self.render_function(function, true /* show header */, 3);
            }
        }
    }

    fn render_methods(&mut self, methods: &[Function], current_heading_level: usize) {
        for method in methods {
            self.render_function(method, false /* show header */, current_heading_level);
        }
    }

    fn render_function(
        &mut self,
        function: &Function,
        show_header: bool,
        current_heading_level: usize,
    ) {
        if show_header {
            self.h3(&format!("Function `{}`", function.name));
        }
        self.render_function_signature(function);
        self.render_comments(
            &function.comments,
            if show_header { 3 } else { current_heading_level },
        );
    }

    fn render_function_signature(&mut self, function: &Function) {
        self.output.push_str("<pre><code>");
        self.output.push_str("pub ");
        if function.unconstrained {
            self.output.push_str("unconstrained ");
        }
        self.output.push_str("fn ");
        self.output.push_str(&function.name);
        self.render_generics(&function.generics);
        self.output.push('(');
        let use_newlines = function.params.len() > 2;
        for (index, param) in function.params.iter().enumerate() {
            if index > 0 && !use_newlines {
                self.output.push_str(", ");
            }
            if use_newlines {
                self.output.push_str("\n    ");
            }
            self.output.push_str(&param.name);
            self.output.push_str(": ");
            self.render_type(&param.r#type);
            if use_newlines {
                self.output.push(',');
            }
        }
        if use_newlines {
            self.output.push('\n');
        }
        self.output.push(')');
        if !matches!(function.return_type, Type::Unit) {
            self.output.push_str(" -> ");
            self.render_type(&function.return_type);
        }
        self.render_where_clause(&function.where_clause);
        self.output.push_str("</code></pre>");
        self.output.push_str("\n\n");
    }

    fn render_generics(&mut self, generics: &[Generic]) {
        if generics.is_empty() {
            return;
        }

        self.output.push_str("&lt;");
        for (index, generic) in generics.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            if let Some(numeric) = &generic.numeric {
                self.output.push_str(&format!("let {}: {}", generic.name, numeric));
            } else {
                self.output.push_str(&generic.name);
            }
        }
        self.output.push_str("&gt;");
    }

    fn render_where_clause(&mut self, where_clause: &[TraitConstraint]) {
        if where_clause.is_empty() {
            return;
        }

        self.output.push_str("\nwhere\n");
        for (index, constraint) in where_clause.iter().enumerate() {
            self.output.push_str("    ");
            self.render_type(&constraint.r#type);
            self.output.push_str(": ");
            self.render_trait_bound(&constraint.bound);
            if index != where_clause.len() - 1 {
                self.output.push(',');
            }
            self.output.push('\n');
        }
    }

    fn render_trait_bound(&mut self, bound: &TraitBound) {
        self.render_id_reference(bound.trait_id, &bound.trait_name);
        self.render_trait_generics(&bound.ordered_generics, &bound.named_generics);
    }

    fn render_id_reference(&mut self, id: usize, name: &str) {
        if let Some(anchor_name) = self.id_to_string.get(&id) {
            self.output.push_str(&format!("<a href=\"#{anchor_name}\">{name}</a>"));
        } else {
            self.output.push_str(name);
        }
    }

    fn render_type(&mut self, typ: &Type) {
        match typ {
            Type::Unit => self.output.push_str("()"),
            Type::Primitive(primitive) => {
                self.output.push_str(primitive);
            }
            Type::Array { length, element } => {
                self.output.push('[');
                self.render_type(element);
                self.output.push_str("; ");
                self.render_type(length);
                self.output.push(']');
            }
            Type::Slice { element } => {
                self.output.push('[');
                self.render_type(element);
                self.output.push(']');
            }
            Type::String { length } => {
                self.output.push_str("str<");
                self.render_type(length);
                self.output.push('>');
            }
            Type::FmtString { length, element } => {
                self.output.push_str("fmtstr<");
                self.render_type(length);
                self.output.push_str(", ");
                self.render_type(element);
                self.output.push('>');
            }
            Type::Tuple(items) => {
                self.output.push('(');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.render_type(item);
                }
                if items.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Type::Reference { r#type, mutable } => {
                self.output.push('&');
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.render_type(r#type);
            }
            Type::Struct { id, name, generics } => {
                self.render_id_reference(*id, name);
                self.render_generic_types(generics);
            }
            Type::TypeAlias { id, name, generics } => {
                self.render_id_reference(*id, name);
                self.render_generic_types(generics);
            }
            Type::Function { params, return_type, env, unconstrained } => {
                if *unconstrained {
                    self.output.push_str("unconstrained ");
                }
                self.output.push_str("fn");
                if !matches!(env.as_ref(), &Type::Unit) {
                    self.output.push('[');
                    self.render_type(env);
                    self.output.push(']');
                }
                self.output.push('(');
                for (index, param) in params.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.render_type(param);
                }
                self.output.push(')');
                if !matches!(return_type.as_ref(), &Type::Unit) {
                    self.output.push_str(" -> ");
                    self.render_type(return_type);
                }
            }
            Type::Constant(value) => {
                self.output.push_str(value);
            }
            Type::Generic(name) => {
                self.output.push_str(name);
            }
            Type::InfixExpr { lhs, operator, rhs } => {
                self.render_type(lhs);
                self.output.push(' ');
                self.output.push_str(operator);
                self.output.push(' ');
                self.render_type(rhs);
            }
            Type::TraitAsType { trait_id, trait_name, ordered_generics, named_generics } => {
                self.output.push_str("impl ");
                self.render_id_reference(*trait_id, trait_name);
                self.render_trait_generics(ordered_generics, named_generics);
            }
        }
    }

    fn render_generic_types(&mut self, generics: &[Type]) {
        if generics.is_empty() {
            return;
        }

        self.output.push_str("&lt;");
        for (index, generic) in generics.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.render_type(generic);
        }
        self.output.push_str("&gt;");
    }

    fn render_trait_generics(&mut self, ordered: &[Type], named: &BTreeMap<String, Type>) {
        if ordered.is_empty() && named.is_empty() {
            return;
        }

        self.output.push('<');
        let mut first = true;
        for generic in ordered {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            self.render_type(generic);
        }
        for (name, typ) in named {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            self.output.push_str(name);
            self.output.push_str(" = ");
            self.render_type(typ);
        }
        self.output.push('>');
    }

    /// Renders the given comments, if any.
    /// Markdown headers that are less or equal than `current_heading_level` will be rendered
    /// as smaller headers (i.e., `###` becomes `####` if `current_heading_level` is 3)
    /// to ensure proper nesting.
    fn render_comments(&mut self, comments: &Option<String>, current_heading_level: usize) {
        let Some(comments) = comments else {
            return;
        };

        let comments = fix_markdown(comments, current_heading_level);
        let html = markdown::to_html(&comments);
        self.output.push_str(&html);
    }

    fn html_start(&mut self, title: &str) {
        self.output.push_str("<!DOCTYPE html>\n");
        self.output.push_str("<html>\n");
        self.output.push_str("<head>\n");
        self.output.push_str("<meta charset=\"UTF-8\">\n");
        self.output.push_str(&format!("<title>{} documentation</title>\n", title));
        self.output.push_str("</head>\n");
        self.output.push_str("<body>\n");
    }

    fn html_end(&mut self) {
        self.output.push_str("</body>\n");
        self.output.push_str("</html>\n");
    }

    fn h1(&mut self, text: &str) {
        self.h(1, text);
    }

    fn h2(&mut self, text: &str) {
        self.h(2, text);
    }

    fn h3(&mut self, text: &str) {
        self.h(3, text);
    }

    fn h4(&mut self, text: &str) {
        self.h(4, text);
    }

    fn h(&mut self, level: usize, text: &str) {
        self.output.push_str(&format!("<h{level}>{text}</h{level}>"));
    }

    fn anchor(&mut self, id: usize) {
        let name = &self.id_to_string[&id];
        self.output.push_str(&format!("<a id=\"{name}\"></a>\n"));
    }

    fn push_file(&mut self, path: PathBuf) {
        let contents = std::mem::take(&mut self.output);
        self.files.push((path, contents));
    }
}

fn index_path() -> PathBuf {
    PathBuf::from("index.html")
}

fn crate_path(krate: &Crate) -> PathBuf {
    PathBuf::from(format!("{}/index.html", krate.name))
}

fn module_path(krate: &Crate, module: &Module) -> PathBuf {
    let module_path = module.name.replace("::", "/");
    PathBuf::from(format!("{}/{}/index.html", krate.name, module_path))
}

fn module_in_crate_path(module: &Module) -> PathBuf {
    let module_path = module.name.replace("::", "/");
    PathBuf::from(format!("{}/index.html", module_path))
}

fn struct_path(krate: &Crate, module: &Module, struct_: &Struct) -> PathBuf {
    let module_path = module.name.replace("::", "/");
    PathBuf::from(format!("{}/{}/struct.{}.html", krate.name, module_path, struct_.name))
}

fn trait_path(krate: &Crate, module: &Module, trait_: &Trait) -> PathBuf {
    let module_path = module.name.replace("::", "/");
    PathBuf::from(format!("{}/{}/trait.{}.html", krate.name, module_path, trait_.name))
}

fn path_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn fix_markdown(markdown: &str, current_heading_level: usize) -> String {
    // Track occurrences of "```" to see if the user forgot to close a code block.
    // If so, we'll close it to prevent ruining the docs.
    let mut open_code_comment = false;
    let mut fixed_comment = String::new();

    'outer_loop: for line in markdown.lines() {
        let trimmed_line = line.trim_start();

        if trimmed_line.starts_with('#') {
            for level in 1..=current_heading_level {
                if trimmed_line.starts_with(&format!("{} ", "#".repeat(level))) {
                    fixed_comment.push_str(&format!(
                        "{} {}",
                        "#".repeat(current_heading_level + 1),
                        &trimmed_line[level + 1..]
                    ));
                    fixed_comment.push('\n');
                    continue 'outer_loop;
                }
            }
        }

        if trimmed_line.starts_with("```") {
            open_code_comment = !open_code_comment;
        }

        fixed_comment.push_str(line);
        fixed_comment.push('\n');
    }

    if open_code_comment {
        fixed_comment.push_str("```");
    }

    fixed_comment.push('\n');
    fixed_comment
}

/// Computes a mapping from item IDs to strings so that HTML anchors have meaningful names.
fn compute_id_to_strings(crates: &Crates) -> HashMap<usize, String> {
    let mut id_strings = HashSet::new();
    let mut id_to_string = HashMap::new();

    for krate in &crates.crates {
        for module in &krate.modules {
            for item in &module.items {
                match item {
                    Item::Struct(struct_) => compute_id_to_string(
                        struct_.id,
                        &struct_.name,
                        &mut id_strings,
                        &mut id_to_string,
                    ),
                    Item::Trait(trait_) => compute_id_to_string(
                        trait_.id,
                        &trait_.name,
                        &mut id_strings,
                        &mut id_to_string,
                    ),
                    Item::TypeAlias(alias_) => compute_id_to_string(
                        alias_.id,
                        &alias_.name,
                        &mut id_strings,
                        &mut id_to_string,
                    ),
                    _ => {}
                }
            }
        }
    }

    id_to_string
}

fn compute_id_to_string(
    id: usize,
    name: &str,
    id_strings: &mut HashSet<String>,
    id_to_string: &mut HashMap<usize, String>,
) {
    if id_strings.contains(name) {
        let name = &format!("{name}_");
        compute_id_to_string(id, name, id_strings, id_to_string);
    } else {
        id_strings.insert(name.to_string());
        id_to_string.insert(id, name.to_string());
    }
}
