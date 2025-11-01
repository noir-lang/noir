use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    html::{
        all_items::AllItems,
        has_class::HasClass,
        has_uri::HasUri,
        id_to_uri::compute_id_to_uri,
        markdown_utils::{fix_markdown, markdown_summary},
        trait_impls::gather_all_trait_impls,
    },
    items::{
        Crate, Function, Generic, Global, HasNameAndComments, Impl, Item, Module, Struct,
        StructField, Trait, TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias, TypeId,
        Workspace,
    },
};

mod all_items;
mod has_class;
mod has_uri;
mod id_to_uri;
mod markdown_utils;
mod trait_impls;

/// Returns a list of (path, contents) representing the HTML files for the given crates.
/// The paths are relative paths that can be joined to a base directory.
///
/// Not all returned files are linked to items. For example, there's an "all.html" file
/// that lists all items, and a "styles.css" file for styling.
pub fn to_html(workspace: &Workspace) -> Vec<(PathBuf, String)> {
    let mut creator = HTMLCreator::new(workspace);
    creator.process_workspace(workspace);
    creator.files
}

/// Builds all the files for the HTML documentation.
struct HTMLCreator {
    output: String,
    files: Vec<(PathBuf, String)>,
    current_path: Vec<String>,
    workspace_name: String,
    id_to_uri: HashMap<TypeId, String>,
    /// Maps a trait ID to all its implementations across all crates.
    all_trait_impls: HashMap<TypeId, HashSet<TraitImpl>>,
}

// Implementation details:
// - there's a single `output: String` buffer. Content is written there as HTML is generated
//   and, once a new page needs to be "written", the buffer is moved to `files` and `output` is
//   emptied.
// - `create_*` methods create new files by writing to `output`, pushing to `files`, then
//   clearing `output`.
// - `render_*` methods just write to `output`.

impl HTMLCreator {
    fn new(workspace: &Workspace) -> Self {
        let output = String::new();
        let files = Vec::new();
        let current_path = Vec::new();
        let workspace_name = workspace.name.clone();
        let id_to_uri = compute_id_to_uri(workspace);

        // Each trait in each create will have trait impls that are found in that crate.
        // When showing a trait we want to show all impls across all crates, so we gather
        // them now.
        let all_trait_impls = gather_all_trait_impls(workspace);
        Self { output, files, current_path, workspace_name, id_to_uri, all_trait_impls }
    }

    fn process_workspace(&mut self, workspace: &Workspace) {
        self.create_styles();
        self.create_all_items(workspace);
        self.create_index(workspace);

        for krate in workspace.all_crates() {
            if krate.root_module.items.is_empty() {
                continue;
            }

            self.create_crate(workspace, krate);
        }
    }

    fn create_styles(&mut self) {
        let contents = include_str!("styles.css");
        self.output.push_str(contents);
        self.push_file(PathBuf::from("styles.css"));
    }

    fn create_all_items(&mut self, workspace: &Workspace) {
        let all_items = all_items::compute_all_items(workspace);
        self.html_start(&format!("All items in {}", workspace.name));
        self.sidebar_start();
        self.render_all_items_sidebar(&all_items);
        self.sidebar_end();
        self.main_start();
        self.h1(&format!("All items in {}", workspace.name));
        self.render_all_items_list("Structs", "struct", &all_items.structs);
        self.render_all_items_list("Traits", "trait", &all_items.traits);
        self.render_all_items_list("Type aliases", "type", &all_items.type_aliases);
        self.render_all_items_list("Globals", "global", &all_items.globals);
        self.render_all_items_list("Functions", "fn", &all_items.functions);
        self.main_end();
        self.html_end();

        self.push_file(PathBuf::from("all.html"));
    }

    fn render_all_items_list(&mut self, title: &str, class: &str, items: &[(Vec<String>, String)]) {
        if items.is_empty() {
            return;
        }

        self.output.push_str(&format!("<span id=\"{class}\"></span>"));
        self.h2(title);
        self.output.push_str("<ul class=\"item-list\">");
        for (path, name) in items {
            let url_path = path.join("/");
            let module = path.join("::");
            self.output.push_str("<li>");
            self.output.push_str(&format!(
                "<a href=\"{url_path}/{class}.{name}.html\">{module}::{name}</a>",
            ));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>");
    }

    fn render_all_items_sidebar(&mut self, all_items: &AllItems) {
        self.h2("Workspace items");
        self.output.push_str("<ul class=\"sidebar-list\">");
        if !all_items.structs.is_empty() {
            self.output.push_str("<li><a href=\"#struct\">Structs</a></li>");
        }
        if !all_items.traits.is_empty() {
            self.output.push_str("<li><a href=\"#trait\">Traits</a></li>");
        }
        if !all_items.type_aliases.is_empty() {
            self.output.push_str("<li><a href=\"#type\">Type aliases</a></li>");
        }
        if !all_items.functions.is_empty() {
            self.output.push_str("<li><a href=\"#fn\">Functions</a></li>");
        }
        if !all_items.globals.is_empty() {
            self.output.push_str("<li><a href=\"#global\">Globals</a></li>");
        }
        self.output.push_str("</ul>");
    }

    fn create_index(&mut self, workspace: &Workspace) {
        let crates = workspace
            .crates
            .iter()
            .filter(|krate| !krate.root_module.items.is_empty())
            .collect::<Vec<_>>();
        let redirect =
            if crates.len() == 1 { Some(format!("{}/index.html", crates[0].name)) } else { None };
        self.html_start_with_redirect(&format!("{} documentation", workspace.name), redirect);

        // This sidebar is empty because there's not much we can list here.
        // It's here so that every page has a sidebar.
        self.sidebar_start();
        self.sidebar_end();

        self.main_start();
        self.h1(&format!("{} documentation", workspace.name));
        self.render_list("Crates", "crates", false, false, &crates);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));
    }

    fn create_crate(&mut self, workspace: &Workspace, krate: &Crate) {
        self.current_path.push(krate.name.clone());

        self.html_start(&format!("Crate {}", krate.name));
        self.sidebar_start();
        self.render_crate_sidebar(workspace);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(false);
        self.h1(&format!("Crate <span class=\"crate\">{}</span>", krate.name));
        self.render_comments(&krate.root_module.comments, 1);
        self.render_items(&krate.root_module.items, false, false);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(&krate.root_module, &krate.root_module.items);

        self.current_path.pop();
    }

    fn render_crate_sidebar(&mut self, workspace: &Workspace) {
        self.h3("Crates");
        self.output.push_str("<ul class=\"sidebar-list\">");
        for krate in &workspace.crates {
            if krate.root_module.items.is_empty() {
                continue;
            }

            self.output.push_str("<li>");
            self.output.push_str(&format!(
                "<a href=\"../{}\" class=\"{}\">{}</a>",
                krate.uri(),
                krate.class(),
                krate.name(),
            ));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>");
    }

    fn render_items(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        self.render_modules(items, sidebar, reference_parent);
        self.render_structs(items, sidebar, reference_parent);
        self.render_traits(items, sidebar, reference_parent);
        self.render_type_aliases(items, sidebar, reference_parent);
        self.render_functions(items, sidebar, reference_parent);
        self.render_globals(items, sidebar, reference_parent);
    }

    fn render_modules(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let modules = get_modules(items);
        if !modules.is_empty() {
            self.render_list("Modules", "modules", sidebar, reference_parent, &modules);
        }
    }

    fn render_structs(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let structs = get_structs(items);
        if !structs.is_empty() {
            self.render_list("Structs", "structs", sidebar, reference_parent, &structs);
        }
    }

    fn render_traits(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let traits = get_traits(items);
        if !traits.is_empty() {
            self.render_list("Traits", "traits", sidebar, reference_parent, &traits);
        }
    }

    fn render_type_aliases(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let type_aliases = get_type_aliases(items);
        if !type_aliases.is_empty() {
            self.render_list(
                "Type aliases",
                "type-aliases",
                sidebar,
                reference_parent,
                &type_aliases,
            );
        }
    }

    fn render_globals(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let globals = get_globals(items);
        if !globals.is_empty() {
            self.render_list("Globals", "globals", sidebar, reference_parent, &globals);
        }
    }

    fn render_functions(&mut self, items: &[Item], sidebar: bool, reference_parent: bool) {
        let functions = get_functions(items);
        if !functions.is_empty() {
            self.render_list("Functions", "functions", sidebar, reference_parent, &functions);
        }
    }

    fn render_list<T: HasNameAndComments + HasUri + HasClass>(
        &mut self,
        title: &str,
        anchor: &str,
        sidebar: bool,
        reference_parent: bool,
        items: &[&T],
    ) {
        if sidebar {
            self.output.push_str(&format!("<h3>{title}</h3>"));
        } else {
            self.output.push_str(&format!("<h2 id=\"{anchor}\">{title}</h2>"));
        }
        self.output.push_str("<ul class=\"item-list\">");

        let mut items = items.to_vec();
        items.sort_by_key(|item| item.name());

        for item in items {
            self.output.push_str("<li>");
            if !sidebar {
                self.output.push_str("<div class=\"item-name\">");
            }
            self.output.push_str(&format!(
                "<a href=\"{}{}\" class=\"{}\">{}</a>",
                if reference_parent { "../" } else { "" },
                item.uri(),
                item.class(),
                item.name(),
            ));
            if !sidebar {
                self.output.push_str("</div>");
                self.output.push_str("<div class=\"item-description\">");
                if let Some(comments) = item.comments() {
                    let summary = markdown_summary(comments);
                    self.output.push_str(&summary);
                }
            }
            self.output.push_str("</div>");
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>");
    }

    fn create_items(&mut self, parent_module: &Module, items: &[Item]) {
        for item in items {
            self.create_item(parent_module, item);
        }
    }

    fn create_item(&mut self, parent_module: &Module, item: &Item) {
        match item {
            Item::Module(module) => self.create_module(parent_module, module),
            Item::Struct(struct_) => self.create_struct(parent_module, struct_),
            Item::Trait(trait_) => self.create_trait(parent_module, trait_),
            Item::TypeAlias(alias) => self.create_alias(parent_module, alias),
            Item::Function(function) => self.create_function(parent_module, function),
            Item::Global(global) => self.create_global(parent_module, global),
        }
    }

    fn create_module(&mut self, parent_module: &Module, module: &Module) {
        self.current_path.push(module.name.clone());

        self.html_start(&format!("Module {}", module.name));
        self.sidebar_start();
        self.render_module_sidebar(parent_module, module);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(false);
        self.h1(&format!("Module <span id=\"mod\" class=\"module\">{}</span>", module.name));
        self.render_comments(&module.comments, 1);
        self.render_items(&module.items, false, false);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(module, &module.items);

        self.current_path.pop();
    }

    fn render_module_sidebar(&mut self, parent_module: &Module, module: &Module) {
        self.h2(&format!("<a href=\"#mod\">Module {}</a>", module.name));
        self.render_module_items_sidebar(module);
        self.render_module_contents_sidebar(parent_module, true);
    }

    fn render_module_items_sidebar(&mut self, module: &Module) {
        if module.items.is_empty() {
            return;
        }
        self.h3("Module items");
        self.output.push_str("<ul class=\"sidebar-list\">");
        if !get_modules(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#modules\">Modules</a></li>");
        }
        if !get_structs(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#structs\">Structs</a></li>");
        }
        if !get_traits(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#traits\">Traits</a></li>");
        }
        if !get_type_aliases(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#type-aliases\">Type aliases</a></li>");
        }
        if !get_functions(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#functions\">Functions</a></li>");
        }
        if !get_globals(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#globals\">Globals</a></li>");
        }
        self.output.push_str("</ul>");
    }

    fn render_module_contents_sidebar(&mut self, module: &Module, reference_parent: bool) {
        if module.name.is_empty() {
            let crate_name = self.current_path.last().unwrap();
            self.h2(&format!("In crate {crate_name}"));
        } else {
            self.h2(&format!("In module {}", module.name));
        }
        self.render_items(&module.items, true, reference_parent);
    }

    fn create_struct(&mut self, parent_module: &Module, struct_: &Struct) {
        self.html_start(&format!("Struct {}", struct_.name));
        self.sidebar_start();
        self.render_struct_sidebar(struct_);
        self.render_module_contents_sidebar(parent_module, false);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(true);
        self.h1(&format!("Struct <span id=\"struct\" class=\"struct\">{}</span>", struct_.name));
        self.render_struct_code(struct_);
        self.render_comments(&struct_.comments, 1);
        self.render_struct_fields(&struct_.fields);
        self.render_impls(&struct_.impls);

        let mut trait_impls = struct_.trait_impls.clone();
        trait_impls.sort_by_key(trait_impl_trait_to_string);
        let show_methods = true;
        self.render_trait_impls(&trait_impls, "Trait implementations", show_methods);

        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(struct_.uri()));
    }

    fn render_struct_sidebar(&mut self, struct_: &Struct) {
        self.h2(&format!("<a href=\"#struct\">Struct {}</a>", struct_.name));

        if !struct_.fields.is_empty() {
            let mut fields = struct_.fields.clone();
            fields.sort_by_key(|field| field.name.clone());

            self.h3("Fields");
            self.output.push_str("<ul class=\"sidebar-list\">");
            for field in fields {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", field.name, field.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>");
        }

        let mut methods = struct_.impls.iter().flat_map(|iter| &iter.methods).collect::<Vec<_>>();
        methods.sort_by_key(|method| method.name.clone());

        if !methods.is_empty() {
            self.h3("Methods");
            self.output.push_str("<ul class=\"sidebar-list\">");
            for method in methods {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", method.name, method.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>");
        }

        self.render_sidebar_trait_impls("Trait implementations", true, &struct_.trait_impls);
    }

    fn create_trait(&mut self, parent_module: &Module, trait_: &Trait) {
        self.html_start(&format!("Trait {}", trait_.name));
        self.sidebar_start();
        self.render_trait_sidebar(trait_);
        self.render_module_contents_sidebar(parent_module, false);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(true);
        self.h1(&format!("Trait <span id=\"trait\" class=\"trait\">{}</span>", trait_.name));
        self.render_trait_code(trait_);
        self.render_comments(&trait_.comments, 1);
        self.render_trait_methods("Required methods", &trait_.required_methods);
        self.render_trait_methods("Provided methods", &trait_.provided_methods);

        let mut trait_impls = self.get_all_trait_impls(trait_);
        trait_impls.sort_by_key(|trait_impl| type_to_string(&trait_impl.r#type));
        let show_methods = false;
        self.render_trait_impls(&trait_impls, "Implementors", show_methods);

        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(trait_.uri()));
    }

    fn render_trait_sidebar(&mut self, trait_: &Trait) {
        self.h2(&format!("<a href=\"#trait\">Trait {}</a>", trait_.name));

        self.render_trait_sidebar_methods("Required methods", &trait_.required_methods);
        self.render_trait_sidebar_methods("Provided methods", &trait_.provided_methods);

        let trait_impls = self.get_all_trait_impls(trait_);
        self.render_sidebar_trait_impls("Implementors", false, &trait_impls);
    }

    fn render_trait_sidebar_methods(&mut self, title: &str, methods: &[Function]) {
        let mut methods = methods.iter().collect::<Vec<_>>();
        methods.sort_by_key(|method| method.name.clone());

        if !methods.is_empty() {
            self.h3(title);
            self.output.push_str("<ul class=\"sidebar-list\">");
            for method in methods {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", method.name, method.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>");
        }
    }

    fn render_sidebar_trait_impls(
        &mut self,
        title: &str,
        display_trait: bool,
        trait_impls: &[TraitImpl],
    ) {
        if trait_impls.is_empty() {
            return;
        }

        let mut trait_impls = trait_impls.to_vec();
        if display_trait {
            trait_impls.sort_by_key(trait_impl_trait_to_string);
        } else {
            trait_impls.sort_by_key(|trait_impl| type_to_string(&trait_impl.r#type));
        }

        self.h3(title);
        self.output.push_str("<ul class=\"sidebar-list\">");
        for trait_impl in trait_impls {
            self.output.push_str("<li>");
            self.output.push_str(&format!(
                "<a href=\"#{}\">{}</a>",
                trait_impl_anchor(&trait_impl),
                if display_trait {
                    trait_impl_trait_to_string(&trait_impl)
                } else {
                    type_to_string(&trait_impl.r#type)
                },
            ));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>");
    }

    fn create_alias(&mut self, parent_module: &Module, alias: &TypeAlias) {
        self.html_start(&format!("Type alias {}", alias.name));
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, false);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(true);
        self.h1(&format!("Type alias <span class=\"type\">{}</span>", alias.name));
        self.render_type_alias_code(alias);
        self.render_comments(&alias.comments, 1);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(alias.uri()));
    }

    fn create_function(&mut self, parent_module: &Module, function: &Function) {
        self.html_start(&format!("Function {}", function.name));
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, false);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(true);
        self.h1(&format!("Function <span class=\"fn\">{}</span>", function.name));
        let as_header = false;
        let output_id = false;
        self.render_function(function, 1, as_header, output_id);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(function.uri()));
    }

    fn create_global(&mut self, parent_module: &Module, global: &Global) {
        self.html_start(&format!("Global {}", global.name));
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, false);
        self.sidebar_end();
        self.main_start();
        self.render_breadcrumbs(true);
        self.h1(&format!("Global <span class=\"global\">{}</span>", global.name));
        self.render_global_code(global);
        self.render_comments(&global.comments, 1);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(global.uri()));
    }

    fn render_struct_code(&mut self, struct_: &Struct) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub struct {}", struct_.name));
        self.render_generics(&struct_.generics);
        if struct_.fields.is_empty() {
            if struct_.has_private_fields {
                self.output.push_str("\n{ <span class=\"comment\">/* private fields */</span> }\n");
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
                self.output.push_str("    <span class=\"comment\">/* private fields */</span>\n");
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
            self.output.push_str(&format!(
                "<div id=\"{}\" class=\"struct-field\"><code class=\"code-header\">",
                field.name
            ));
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.render_type(&field.r#type);
            self.output.push_str("</code></div>\n");
            self.output.push_str("<div class=\"padded-description\">");
            self.render_comments(&field.comments, 3);
            self.output.push_str("</div>");
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
        self.output.push_str("<h3><code class=\"code-header\">");
        self.output.push_str("impl");
        self.render_generics(&impl_.generics);
        self.output.push(' ');
        self.render_type(&impl_.r#type);
        self.output.push_str("</code></h3>\n\n");
        let output_id = true;
        self.render_methods(&impl_.methods, 3, output_id);
    }

    fn render_trait_impls(&mut self, trait_impls: &[TraitImpl], title: &str, show_methods: bool) {
        if trait_impls.is_empty() {
            return;
        }

        self.h2(title);

        for trait_impl in trait_impls {
            self.render_trait_impl(trait_impl, show_methods);
        }
    }

    fn render_trait_impl(&mut self, trait_impl: &TraitImpl, show_methods: bool) {
        let anchor = trait_impl_anchor(trait_impl);
        self.output.push_str(&format!("<h3 id=\"{anchor}\"><code class=\"code-header\">"));
        self.output.push_str("impl");
        self.render_generics(&trait_impl.generics);
        self.output.push(' ');
        self.render_id_reference(trait_impl.trait_id, &trait_impl.trait_name, ReferenceKind::Trait);
        self.render_generic_types(&trait_impl.trait_generics);
        self.output.push_str(" for ");
        self.render_type(&trait_impl.r#type);
        let indent = 0;
        self.render_where_clause(&trait_impl.where_clause, indent);
        self.output.push_str("</code></h3>\n\n");

        if show_methods {
            let output_id = false;
            self.render_methods(&trait_impl.methods, 3, output_id);
        }
    }

    fn render_trait_code(&mut self, trait_: &Trait) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub trait {}", trait_.name));
        self.render_generics(&trait_.generics);
        if !trait_.bounds.is_empty() {
            self.output.push(':');
            for (index, bound) in trait_.bounds.iter().enumerate() {
                self.output.push_str("\n    ");
                if index > 0 {
                    self.output.push_str("+ ");
                }
                self.render_trait_bound(bound);
            }
        }
        let indent = 0;
        self.render_where_clause(&trait_.where_clause, indent);
        self.output.push_str(" {\n");

        let color_name = true;
        let link = true;
        let indent = 1;
        let mut printed_member = false;

        if !trait_.associated_types.is_empty() {
            for associated_type in &trait_.associated_types {
                self.output.push_str("    type ");
                self.output.push_str(&associated_type.name);
                if !associated_type.bounds.is_empty() {
                    self.output.push_str(": ");
                    for (index, bound) in associated_type.bounds.iter().enumerate() {
                        if index > 0 {
                            self.output.push_str(" + ");
                        }
                        self.render_trait_bound(bound);
                    }
                }
                self.output.push_str(";\n");
            }

            printed_member = true;
        }

        if !trait_.associated_constants.is_empty() {
            for associated_constant in &trait_.associated_constants {
                self.output.push_str("    let ");
                self.output.push_str(&associated_constant.name);
                self.output.push_str(": ");
                self.render_type(&associated_constant.r#type);
                self.output.push_str(";\n");
            }

            printed_member = true;
        }

        if !trait_.required_methods.is_empty() {
            if printed_member {
                self.output.push('\n');
            }
            self.output.push_str("    <span class=\"comment\">// Required methods</span>\n");
            for method in &trait_.required_methods {
                self.output.push_str("    ");
                self.render_function_signature_inner(method, color_name, link, indent);
                self.output.push_str(";\n");
            }
            printed_member = true;
        }

        if !trait_.provided_methods.is_empty() {
            if printed_member {
                self.output.push('\n');
            }
            self.output.push_str("    <span class=\"comment\">// Provided methods</span>\n");
            for method in &trait_.provided_methods {
                self.output.push_str("    ");
                self.render_function_signature_inner(method, color_name, link, indent);
                self.output.push_str(" { ... }\n");
            }
        }
        self.output.push('}');
        self.output.push_str("</code></pre>\n\n");
    }

    fn render_trait_methods(&mut self, title: &str, methods: &[Function]) {
        if methods.is_empty() {
            return;
        }

        self.h2(title);
        let output_id = true;
        self.render_methods(methods, 2, output_id);
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

    fn render_methods(
        &mut self,
        methods: &[Function],
        current_heading_level: usize,
        output_id: bool,
    ) {
        self.output.push_str("<div class=\"padded-methods\">");
        for method in methods {
            self.render_function(method, current_heading_level, true, output_id);
        }
        self.output.push_str("</div>");
    }

    fn render_function(
        &mut self,
        function: &Function,
        current_heading_level: usize,
        as_header: bool,
        output_id: bool,
    ) {
        self.render_function_signature(function, as_header, output_id);
        if function.comments.is_some() {
            if as_header {
                self.output.push_str("<div class=\"padded-description\">");
            }
            self.render_comments(&function.comments, current_heading_level);
            if as_header {
                self.output.push_str("</div>");
            }
        }
    }

    fn render_function_signature(&mut self, function: &Function, as_header: bool, output_id: bool) {
        if as_header {
            if output_id {
                self.output
                    .push_str(&format!("<code id=\"{}\" class=\"code-header\">", function.name));
            } else {
                self.output.push_str("<code class=\"code-header\">");
            }
        } else {
            self.output.push_str("<pre>");
            self.output.push_str("<code>");
        }
        let color_name = as_header;
        let link = false;
        let indent = 0;
        self.render_function_signature_inner(function, color_name, link, indent);
        self.output.push_str("</code>");
        if !as_header {
            self.output.push_str("</pre>");
        }
        self.output.push_str("\n\n");
    }

    fn render_function_signature_inner(
        &mut self,
        function: &Function,
        color_name: bool,
        link: bool,
        indent: usize,
    ) {
        self.output.push_str("pub ");
        if function.unconstrained {
            self.output.push_str("unconstrained ");
        }
        if function.comptime {
            self.output.push_str("comptime ");
        }
        self.output.push_str("fn ");
        if link {
            self.output.push_str(&format!("<a href=\"#{}\">", function.name));
        }
        if color_name {
            self.output.push_str("<span class=\"fn\">");
            self.output.push_str(&function.name);
            self.output.push_str("</span>");
        } else {
            self.output.push_str(&function.name);
        }
        if link {
            self.output.push_str("</a>");
        }
        self.render_generics(&function.generics);
        self.output.push('(');
        let use_newlines = function.params.len() > 2;
        for (index, param) in function.params.iter().enumerate() {
            if index > 0 && !use_newlines {
                self.output.push_str(", ");
            }
            if use_newlines {
                self.output.push('\n');
                self.output.push_str(&" ".repeat(4 * (indent + 1)));
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
            self.output.push_str(&" ".repeat(4 * indent));
        }
        self.output.push(')');
        if !matches!(function.return_type, Type::Unit) {
            self.output.push_str(" -> ");
            self.render_type(&function.return_type);
        }
        self.render_where_clause(&function.where_clause, indent);
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

    fn render_where_clause(&mut self, where_clause: &[TraitConstraint], indent: usize) {
        if where_clause.is_empty() {
            return;
        }

        self.output.push('\n');
        self.output.push_str(&" ".repeat(4 * indent));
        self.output.push_str("where\n");
        for (index, constraint) in where_clause.iter().enumerate() {
            self.output.push_str(&" ".repeat(4 * (indent + 1)));
            self.render_type(&constraint.r#type);
            self.output.push_str(": ");
            self.render_trait_bound(&constraint.bound);
            if index != where_clause.len() - 1 {
                self.output.push_str(",\n");
            }
        }
    }

    fn render_trait_bound(&mut self, bound: &TraitBound) {
        self.render_id_reference(bound.trait_id, &bound.trait_name, ReferenceKind::Trait);
        self.render_trait_generics(&bound.ordered_generics, &bound.named_generics);
    }

    fn render_id_reference(&mut self, id: TypeId, name: &str, kind: ReferenceKind) {
        if let Some(uri) = self.id_to_uri.get(&id) {
            let class = match kind {
                ReferenceKind::Struct => "struct",
                ReferenceKind::Trait => "trait",
                ReferenceKind::TypeAlias => "type",
            };
            let nesting = self.current_path.len();
            self.output.push_str(&format!(
                "<a href=\"{}{uri}\" class=\"{class}\">{name}</a>",
                "../".repeat(nesting)
            ));
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
                self.output.push_str("str&lt;");
                self.render_type(length);
                self.output.push_str("&gt;");
            }
            Type::FmtString { length, element } => {
                self.output.push_str("fmtstr&lt;");
                self.render_type(length);
                self.output.push_str(", ");
                self.render_type(element);
                self.output.push_str("&gt;");
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
                self.render_id_reference(*id, name, ReferenceKind::Struct);
                self.render_generic_types(generics);
            }
            Type::TypeAlias { id, name, generics } => {
                self.render_id_reference(*id, name, ReferenceKind::TypeAlias);
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
                self.output.push_str(&escape_html(name));
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
                self.render_id_reference(*trait_id, trait_name, ReferenceKind::Trait);
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

        self.output.push_str("&lt;");
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
        self.output.push_str("&gt;");
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
        self.output.push_str("<div class=\"comments\">\n");
        self.output.push_str(&html);
        self.output.push_str("</div>\n");
    }

    fn render_breadcrumbs(&mut self, last_is_link: bool) {
        self.output.push_str("<div>");
        let mut nesting = self.current_path.len();
        self.output.push_str(
            format!("<a href=\"{}index.html\">{}</a>", "../".repeat(nesting), self.workspace_name)
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

    fn get_all_trait_impls(&self, trait_: &Trait) -> Vec<TraitImpl> {
        self.all_trait_impls
            .get(&trait_.id)
            .map(|impls| impls.iter().cloned().collect())
            .unwrap_or_else(|| trait_.trait_impls.clone())
    }

    fn html_start(&mut self, title: &str) {
        self.html_start_with_redirect(title, None);
    }

    fn html_start_with_redirect(&mut self, title: &str, redirect: Option<String>) {
        self.output.push_str("<!DOCTYPE html>\n");
        self.output.push_str("<html>\n");
        self.output.push_str("<head>\n");
        self.output.push_str("<meta charset=\"UTF-8\">\n");
        if let Some(redirect) = redirect {
            self.output.push_str(&format!(
                "<meta http-equiv=\"refresh\" content=\"0; url='{redirect}'\">\n",
            ));
        }

        let nesting = self.current_path.len();
        self.output.push_str(&format!(
            "<link rel=\"stylesheet\" href=\"{}styles.css\">\n",
            "../".repeat(nesting)
        ));
        self.output.push_str(&format!("<title>{title} documentation</title>\n"));
        self.output.push_str("</head>\n");
        self.output.push_str("<body>\n");
    }

    fn html_end(&mut self) {
        self.output.push_str("</body>\n");
        self.output.push_str("</html>\n");
    }

    fn main_start(&mut self) {
        self.output.push_str("<main>\n");
    }

    fn main_end(&mut self) {
        self.output.push_str("</main>\n");
    }

    fn sidebar_start(&mut self) {
        self.output.push_str("<nav class=\"sidebar\">\n");
        let nesting = self.current_path.len();
        let crates_name = self.workspace_name.clone();
        self.h1(&format!("<a href=\"{}index.html\">{}</a>", "../".repeat(nesting), crates_name));
        self.output.push_str(&format!(
            "<div><a href=\"{}all.html\">{}</a></div>\n",
            "../".repeat(nesting),
            "All items"
        ));
    }

    fn sidebar_end(&mut self) {
        self.output.push_str("</nav>\n");
    }

    fn h1(&mut self, text: &str) {
        self.output.push_str(&format!("<h1>{text}</h1>"));
    }

    fn h2(&mut self, text: &str) {
        self.output.push_str(&format!("<h2>{text}</h2>"));
    }

    fn h3(&mut self, text: &str) {
        self.output.push_str(&format!("<h3>{text}</h3>"));
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

fn get_modules(items: &[Item]) -> Vec<&Module> {
    items
        .iter()
        .filter_map(|item| if let Item::Module(module) = item { Some(module) } else { None })
        .collect()
}

fn get_structs(items: &[Item]) -> Vec<&Struct> {
    items
        .iter()
        .filter_map(|item| if let Item::Struct(struct_) = item { Some(struct_) } else { None })
        .collect()
}

fn get_traits(items: &[Item]) -> Vec<&Trait> {
    items
        .iter()
        .filter_map(|item| if let Item::Trait(trait_) = item { Some(trait_) } else { None })
        .collect()
}

fn get_type_aliases(items: &[Item]) -> Vec<&TypeAlias> {
    items
        .iter()
        .filter_map(|item| if let Item::TypeAlias(alias) = item { Some(alias) } else { None })
        .collect()
}

fn get_globals(items: &[Item]) -> Vec<&Global> {
    items
        .iter()
        .filter_map(|item| if let Item::Global(global) = item { Some(global) } else { None })
        .collect()
}

fn get_functions(items: &[Item]) -> Vec<&Function> {
    items
        .iter()
        .filter_map(|item| if let Item::Function(function) = item { Some(function) } else { None })
        .collect()
}

fn trait_impl_anchor(trait_impl: &TraitImpl) -> String {
    let mut string = String::new();
    string.push_str("impl-");
    string.push_str(&trait_impl_trait_to_string(trait_impl));
    string.push_str("-for-");
    string.push_str(&type_to_string(&trait_impl.r#type));
    string
}

fn trait_impl_trait_to_string(trait_impl: &TraitImpl) -> String {
    let mut string = String::new();
    string.push_str(&trait_impl.trait_name);
    if !trait_impl.trait_generics.is_empty() {
        string.push_str("&lt;");
        for (index, typ) in trait_impl.trait_generics.iter().enumerate() {
            if index > 0 {
                string.push_str(", ");
            }
            string.push_str(&type_to_string(typ));
        }
        string.push_str("&gt;");
    }
    string
}

fn type_to_string(typ: &Type) -> String {
    match typ {
        Type::Unit => "()".to_string(),
        Type::Primitive(primitive) => primitive.clone(),
        Type::Array { length, element } => {
            format!("[{}; {}]", type_to_string(element), type_to_string(length))
        }
        Type::Slice { element } => format!("[{}]", type_to_string(element)),
        Type::String { length } => format!("str&lt;{}&gt;", type_to_string(length)),
        Type::FmtString { length, element } => {
            format!("fmtstr&lt;{}, {}&gt;", type_to_string(length), type_to_string(element))
        }
        Type::Tuple(items) => {
            let items: Vec<String> = items.iter().map(type_to_string).collect();
            format!("({}{})", items.join(", "), if items.len() == 1 { "," } else { "" })
        }
        Type::Reference { r#type, mutable } => {
            if *mutable {
                format!("&mut {}", type_to_string(r#type))
            } else {
                format!("&{}", type_to_string(r#type))
            }
        }
        Type::Struct { name, .. } => name.clone(),
        Type::TypeAlias { name, .. } => name.clone(),
        Type::Function { .. } => "fn".to_string(),
        Type::Constant(value) => value.clone(),
        Type::Generic(name) => escape_html(name),
        Type::InfixExpr { lhs, operator, rhs } => {
            format!("{}{}{}", type_to_string(lhs), operator, type_to_string(rhs))
        }
        Type::TraitAsType { trait_name, .. } => format!("impl-{trait_name}"),
    }
}

fn escape_html(input: &str) -> String {
    input.replace('<', "&lt;").replace('>', "&gt;")
}

enum ReferenceKind {
    Struct,
    Trait,
    TypeAlias,
}
