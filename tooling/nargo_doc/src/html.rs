use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use crate::items::{
    Crate, Crates, Function, Generic, Global, HasNameAndComments, Impl, Item, Module, Struct,
    StructField, Trait, TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias,
};

/// Returns a list of (path, contents) representing the HTML files for the given crates.
/// The paths are relative paths that can be joined to a base directory.
pub fn to_html(crates: &Crates) -> Vec<(PathBuf, String)> {
    let mut creator = HTMLCreator::new(crates);
    creator.process_crates(crates);
    creator.files
}

struct HTMLCreator {
    output: String,
    files: Vec<(PathBuf, String)>,
    current_path: Vec<String>,
    crates_name: String,
    id_to_path: HashMap<usize, String>,
}

impl HTMLCreator {
    fn new(crates: &Crates) -> Self {
        let output = String::new();
        let files = Vec::new();
        let current_path = Vec::new();
        let crates_name = crates.name.clone();
        let id_to_path = compute_id_to_path(crates);
        Self { output, files, current_path, crates_name, id_to_path }
    }

    fn process_crates(&mut self, crates: &Crates) {
        self.create_index(crates);

        for krate in &crates.crates {
            self.create_crate(krate);
        }
    }

    fn create_index(&mut self, crates: &Crates) {
        self.html_start(&format!("{} documentation", crates.name));
        self.h1(&format!("{} documentation", crates.name));
        self.h2("Crates");
        self.output.push_str("<ul>\n");
        for krate in &crates.crates {
            self.output.push_str("<li>");
            self.output
                .push_str(&format!("<a href=\"{}/index.html\">{}</a>", krate.name, krate.name));
            if let Some(comments) = krate.comments() {
                let summary = markdown_summary(comments);
                if !summary.is_empty() {
                    self.output.push_str(&format!(": {summary}"));
                }
            }
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
        self.html_end();
        self.push_file(PathBuf::from("index.html"));
    }

    fn create_crate(&mut self, krate: &Crate) {
        self.current_path.push(krate.name.clone());

        self.html_start(&format!("Crate {}", krate.name));
        self.render_breadcrumbs(false);
        self.h1(&format!("Crate {}", krate.name));
        self.render_comments(&krate.root_module.comments, 1);
        self.render_items(&krate.root_module.items);
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(&krate.root_module.items);

        self.current_path.pop();
    }

    fn render_items(&mut self, items: &[Item]) {
        self.render_modules(items);
        self.render_structs(items);
        self.render_traits(items);
        self.render_type_aliases(items);
        self.render_globals(items);
        self.render_functions(items);
    }

    fn render_modules(&mut self, items: &[Item]) {
        let modules = items
            .iter()
            .filter_map(|item| if let Item::Module(module) = item { Some(module) } else { None })
            .collect::<Vec<_>>();
        if !modules.is_empty() {
            self.render_list("Modules", &modules);
        }
    }

    fn render_structs(&mut self, items: &[Item]) {
        let structs = items
            .iter()
            .filter_map(|item| if let Item::Struct(struct_) = item { Some(struct_) } else { None })
            .collect::<Vec<_>>();
        if !structs.is_empty() {
            self.render_list("Structs", &structs);
        }
    }

    fn render_traits(&mut self, items: &[Item]) {
        let traits = items
            .iter()
            .filter_map(|item| if let Item::Trait(trait_) = item { Some(trait_) } else { None })
            .collect::<Vec<_>>();
        if !traits.is_empty() {
            self.render_list("Traits", &traits);
        }
    }

    fn render_type_aliases(&mut self, items: &[Item]) {
        let type_aliases = items
            .iter()
            .filter_map(|item| if let Item::TypeAlias(alias) = item { Some(alias) } else { None })
            .collect::<Vec<_>>();
        if !type_aliases.is_empty() {
            self.render_list("Type aliases", &type_aliases);
        }
    }

    fn render_globals(&mut self, items: &[Item]) {
        let globals = items
            .iter()
            .filter_map(|item| if let Item::Global(global) = item { Some(global) } else { None })
            .collect::<Vec<_>>();
        if !globals.is_empty() {
            self.render_list("Globals", &globals);
        }
    }

    fn render_functions(&mut self, items: &[Item]) {
        let functions = items
            .iter()
            .filter_map(
                |item| if let Item::Function(function) = item { Some(function) } else { None },
            )
            .collect::<Vec<_>>();
        if !functions.is_empty() {
            self.render_list("Functions", &functions);
        }
    }

    fn render_list<T: HasNameAndComments + HasPath>(&mut self, title: &str, items: &[&T]) {
        self.h2(title);
        self.output.push_str("<ul>");
        for item in items {
            self.output.push_str("<li>");
            self.output.push_str(&format!("<a href=\"{}\">{}</a>", item.path(), item.name(),));
            if let Some(comments) = item.comments() {
                let summary = markdown_summary(comments);
                if !summary.is_empty() {
                    self.output.push_str(&format!(": {summary}"));
                }
            }
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>");
    }

    fn create_items(&mut self, items: &[Item]) {
        for item in items {
            self.create_item(item);
        }
    }

    fn create_item(&mut self, item: &Item) {
        match item {
            Item::Module(module) => self.create_module(module),
            Item::Struct(struct_) => self.create_struct(struct_),
            Item::Trait(trait_) => self.create_trait(trait_),
            Item::TypeAlias(alias) => self.create_alias(alias),
            Item::Function(function) => self.create_function(function),
            Item::Global(global) => self.create_global(global),
        }
    }

    fn create_module(&mut self, module: &Module) {
        self.current_path.push(module.name.clone());

        let title = format!("Module {}", module.name);
        self.html_start(&title);
        self.render_breadcrumbs(false);
        self.h1(&title);
        self.render_comments(&module.comments, 2);
        self.render_items(&module.items);
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(&module.items);

        self.current_path.pop();
    }

    fn create_struct(&mut self, struct_: &Struct) {
        let title = format!("Struct {}", struct_.name);
        self.html_start(&title);
        self.render_breadcrumbs(true);
        self.h1(&title);
        self.render_struct_code(struct_);
        self.render_comments(&struct_.comments, 1);
        self.render_struct_fields(&struct_.fields);
        self.render_impls(&struct_.impls);
        self.render_trait_impls(&struct_.trait_impls, "Trait implementations");
        self.html_end();
        self.push_file(PathBuf::from(struct_.path()));
    }

    fn create_trait(&mut self, trait_: &Trait) {
        let title = format!("Trait {}", trait_.name);
        self.html_start(&title);
        self.render_breadcrumbs(true);
        self.h1(&title);
        self.render_trait_code(trait_);
        self.render_comments(&trait_.comments, 1);
        self.render_trait_methods(&trait_.methods);
        self.render_trait_impls(&trait_.trait_impls, "Implementors");
        self.html_end();
        self.push_file(PathBuf::from(trait_.path()));
    }

    fn create_alias(&mut self, alias: &TypeAlias) {
        let title = format!("Type alias {}", alias.name);
        self.html_start(&title);
        self.render_breadcrumbs(true);
        self.h1(&title);
        self.render_type_alias_code(alias);
        self.render_comments(&alias.comments, 1);
        self.html_end();
        self.push_file(PathBuf::from(alias.path()));
    }

    fn create_function(&mut self, function: &Function) {
        let title = format!("Function {}", function.name);
        self.html_start(&title);
        self.render_breadcrumbs(true);
        self.h1(&title);
        self.render_function(function, 1);
        self.html_end();
        self.push_file(PathBuf::from(function.path()));
    }

    fn create_global(&mut self, global: &Global) {
        let title = format!("Global {}", global.name);
        self.html_start(&title);
        self.render_breadcrumbs(true);
        self.h1(&title);
        self.render_global_code(global);
        self.render_comments(&global.comments, 1);
        self.html_end();
        self.push_file(PathBuf::from(global.path()));
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

        self.h2("Fields");

        for field in fields {
            self.output.push_str("<h3><pre><code>");
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.render_type(&field.r#type);
            self.output.push_str("</code></pre></h3>\n");
            self.render_comments(&field.comments, 5);
        }
    }

    fn render_impls(&mut self, impls: &[Impl]) {
        if impls.is_empty() {
            return;
        }

        self.h2("Implementations");

        for impl_ in impls {
            self.render_impl(impl_);
        }
    }

    fn render_impl(&mut self, impl_: &Impl) {
        self.output.push_str("<h3><pre><code>");
        self.output.push_str("impl");
        self.render_generics(&impl_.generics);
        self.output.push(' ');
        self.render_type(&impl_.r#type);
        self.output.push_str("</code></pre></h3>\n\n");
        self.render_methods(&impl_.methods, 5);
    }

    fn render_trait_impls(&mut self, trait_impls: &[TraitImpl], title: &str) {
        if trait_impls.is_empty() {
            return;
        }

        self.h2(title);

        for trait_impl in trait_impls {
            self.render_trait_impl(trait_impl);
        }
    }

    fn render_trait_impl(&mut self, trait_impl: &TraitImpl) {
        self.output.push_str("<h3><pre><code>");
        self.output.push_str("impl");
        self.render_generics(&trait_impl.generics);
        self.output.push(' ');
        self.render_id_reference(trait_impl.trait_id, &trait_impl.trait_name);
        self.render_generic_types(&trait_impl.trait_generics);
        self.output.push_str(" for ");
        self.render_type(&trait_impl.r#type);
        self.render_where_clause(&trait_impl.where_clause);
        self.output.push_str("</code></pre></h3>\n\n");
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

        self.h2("Methods");
        self.render_methods(methods, 2);
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

    fn render_methods(&mut self, methods: &[Function], current_heading_level: usize) {
        for method in methods {
            self.render_function(method, current_heading_level);
        }
    }

    fn render_function(&mut self, function: &Function, current_heading_level: usize) {
        self.render_function_signature(function);
        self.render_comments(&function.comments, current_heading_level);
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
        if let Some(path) = self.id_to_path.get(&id) {
            let nesting = self.current_path.len();
            self.output
                .push_str(&format!("<a href=\"{}{path}\">{name}</a>", "../".repeat(nesting)));
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

    fn render_breadcrumbs(&mut self, last_is_link: bool) {
        self.output.push_str("<div>");
        let mut nesting = self.current_path.len();
        self.output.push_str(
            format!("<a href=\"{}index.html\">{}</a>", "../".repeat(nesting), self.crates_name)
                .as_str(),
        );
        self.output.push_str(" - ");
        for (index, item) in self.current_path.iter().enumerate() {
            if index > 0 {
                self.output.push_str("::");
            }
            nesting -= 1;
            if !last_is_link && index == self.current_path.len() - 1 {
                self.output.push_str(item);
            } else {
                self.output.push_str(
                    format!("<a href=\"{}index.html\">{}</a>", "../".repeat(nesting), item)
                        .as_str(),
                );
            }
        }
        self.output.push_str("</div>");
    }

    fn html_start(&mut self, title: &str) {
        self.output.push_str("<!DOCTYPE html>\n");
        self.output.push_str("<html>\n");
        self.output.push_str("<head>\n");
        self.output.push_str("<meta charset=\"UTF-8\">\n");
        self.output.push_str(&format!("<title>{title} documentation</title>\n"));
        self.output.push_str("</head>\n");
        self.output.push_str("<body>\n");
    }

    fn html_end(&mut self) {
        self.output.push_str("</body>\n");
        self.output.push_str("</html>\n");
    }

    fn h1(&mut self, text: &str) {
        self.output.push_str(&format!("<h1>{text}</h1>"));
    }

    fn h2(&mut self, text: &str) {
        self.output.push_str(&format!("<h2>{text}</h2>"));
    }

    fn push_file(&mut self, path: PathBuf) {
        let mut full_path = PathBuf::new();
        for item in &self.current_path {
            full_path = full_path.join(item);
        }

        let contents = std::mem::take(&mut self.output);
        let path = full_path.join(path);
        self.files.push((path, contents));
    }
}

fn compute_id_to_path(crates: &Crates) -> HashMap<usize, String> {
    let mut id_to_path = HashMap::new();
    let mut path = Vec::new();

    for krate in &crates.crates {
        path.push(krate.name.to_string());
        for item in &krate.root_module.items {
            compute_id_to_path_in_item(item, &mut id_to_path, &mut path);
        }

        path.pop();
    }

    id_to_path
}

fn compute_id_to_path_in_item(
    item: &Item,
    id_to_path: &mut HashMap<usize, String>,
    path: &mut Vec<String>,
) {
    match item {
        Item::Module(module) => {
            path.push(module.name.clone());
            for item in &module.items {
                compute_id_to_path_in_item(item, id_to_path, path);
            }
            path.pop();
        }
        Item::Struct(struct_) => {
            let path = format!("{}/{}", path.join("/"), struct_.path());
            id_to_path.insert(struct_.id, path);
        }
        Item::Trait(trait_) => {
            let path = format!("{}/{}", path.join("/"), trait_.path());
            id_to_path.insert(trait_.id, path);
        }
        Item::TypeAlias(type_alias) => {
            let path = format!("{}/{}", path.join("/"), type_alias.path());
            id_to_path.insert(type_alias.id, path);
        }
        Item::Function(_) | Item::Global(_) => {}
    }
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

/// Returns a summary of the given markdown (up to the first blank line).
fn markdown_summary(markdown: &str) -> String {
    let mut string = String::new();
    for line in markdown.lines() {
        if line.trim().is_empty() {
            break;
        }
        string.push_str(line);
        string.push('\n');
    }
    string.trim().to_string()
}

trait HasPath {
    fn path(&self) -> String;
}

impl HasPath for Module {
    fn path(&self) -> String {
        format!("{}/index.html", self.name)
    }
}

impl HasPath for Struct {
    fn path(&self) -> String {
        format!("struct.{}.html", self.name)
    }
}

impl HasPath for Trait {
    fn path(&self) -> String {
        format!("trait.{}.html", self.name)
    }
}

impl HasPath for TypeAlias {
    fn path(&self) -> String {
        format!("type.{}.html", self.name)
    }
}

impl HasPath for Global {
    fn path(&self) -> String {
        format!("global.{}.html", self.name)
    }
}

impl HasPath for Function {
    fn path(&self) -> String {
        format!("fn.{}.html", self.name)
    }
}
