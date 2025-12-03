use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use noirc_frontend::ast::ItemVisibility;

use crate::{
    html::{
        all_items::AllItems,
        colorize::colorize_markdown_code_blocks,
        has_class::HasClass,
        has_uri::HasUri,
        id_to_info::{ItemInfo, compute_id_to_info},
        markdown_utils::{fix_markdown, markdown_summary},
        trait_impls::gather_all_trait_impls,
    },
    items::{
        Comments, Crate, Function, FunctionParam, Generic, Global, Impl, Item, ItemId,
        ItemProperties, LinkTarget, Links, Module, PrimitiveType, PrimitiveTypeKind, Reexport,
        Struct, StructField, Trait, TraitBound, TraitConstraint, TraitImpl, Type, TypeAlias,
        Workspace,
    },
};

mod all_items;
mod colorize;
mod has_class;
mod has_uri;
mod id_to_info;
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
    current_crate_version: Option<String>,
    workspace_name: String,
    id_to_info: HashMap<ItemId, ItemInfo>,
    /// Maps a trait ID to all its implementations across all crates.
    all_trait_impls: HashMap<ItemId, HashSet<TraitImpl>>,
    self_type: Option<Type>,
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
        let id_to_info = compute_id_to_info(workspace);

        // Each trait in each create will have trait impls that are found in that crate.
        // When showing a trait we want to show all impls across all crates, so we gather
        // them now.
        let all_trait_impls = gather_all_trait_impls(workspace);
        Self {
            output,
            files,
            current_path,
            current_crate_version: None,
            workspace_name,
            id_to_info,
            all_trait_impls,
            self_type: None,
        }
    }

    fn process_workspace(&mut self, workspace: &Workspace) {
        self.create_styles();
        self.create_js();
        self.create_all_items(workspace);
        self.create_index(workspace);

        for krate in workspace.all_crates() {
            if !krate.root_module.has_public_items() {
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

    fn create_js(&mut self) {
        let contents = include_str!("nargo_doc.js");
        self.output.push_str(contents);
        self.push_file(PathBuf::from("nargo_doc.js"));
    }

    fn create_all_items(&mut self, workspace: &Workspace) {
        let all_items = all_items::compute_all_items(workspace);
        self.html_start(&format!("All items in {}", workspace.name), &workspace.name);
        self.sidebar_start();
        self.render_all_items_sidebar(&all_items);
        self.sidebar_end();
        self.main_start(false);
        self.h1(&format!("All items in {}", workspace.name));
        self.render_all_items_list("Structs", "struct", &all_items.structs);
        self.render_all_items_list("Traits", "trait", &all_items.traits);
        self.render_all_items_list("Type aliases", "type", &all_items.type_aliases);
        self.render_all_items_list("Primitive types", "primitive", &all_items.primitive_types);
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
        self.output.push_str("<ul class=\"item-list\">\n");
        for (path, name) in items {
            let url_path = path.join("/");
            let module = path.join("::");
            self.output.push_str("<li>");
            self.output.push_str(&format!(
                "<a href=\"{url_path}/{class}.{name}.html\">{module}::{name}</a>",
            ));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn render_all_items_sidebar(&mut self, all_items: &AllItems) {
        self.h2("Workspace items");
        self.output.push_str("<ul class=\"sidebar-list\">\n");
        if !all_items.structs.is_empty() {
            self.output.push_str("<li><a href=\"#struct\">Structs</a></li>\n");
        }
        if !all_items.traits.is_empty() {
            self.output.push_str("<li><a href=\"#trait\">Traits</a></li>\n");
        }
        if !all_items.type_aliases.is_empty() {
            self.output.push_str("<li><a href=\"#type\">Type aliases</a></li>\n");
        }
        if !all_items.primitive_types.is_empty() {
            self.output.push_str("<li><a href=\"#primitive\">Primitive types</a></li>\n");
        }
        if !all_items.functions.is_empty() {
            self.output.push_str("<li><a href=\"#fn\">Functions</a></li>\n");
        }
        if !all_items.globals.is_empty() {
            self.output.push_str("<li><a href=\"#global\">Globals</a></li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn create_index(&mut self, workspace: &Workspace) {
        let crates = workspace
            .crates
            .iter()
            .filter(|krate| krate.root_module.has_public_items())
            .collect::<Vec<_>>();
        let redirect =
            if crates.len() == 1 { Some(format!("{}/index.html", crates[0].name)) } else { None };
        self.html_start_with_redirect(
            &format!("{} documentation", workspace.name),
            &workspace.name,
            redirect,
        );

        // This sidebar is empty because there's not much we can list here.
        // It's here so that every page has a sidebar.
        self.sidebar_start();
        self.sidebar_end();

        self.main_start(false);
        self.h1(&format!("{} documentation", workspace.name));
        self.render_list("Crates", "crates", false, 0, &crates);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));
    }

    fn create_crate(&mut self, workspace: &Workspace, krate: &Crate) {
        self.current_path.push(krate.name.clone());
        self.current_crate_version = krate.version.clone();

        self.html_start(&format!("Crate {}", krate.name), &krate.name);
        self.sidebar_start();
        self.render_crate_sidebar(workspace, krate);
        self.sidebar_end();
        self.main_start(false);
        self.h1(&format!("Crate <span class=\"crate\">{}</span>", krate.name));
        self.render_comments(krate.root_module.comments.as_ref(), 1);
        self.render_items(&krate.root_module.items, false, 0);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(&krate.root_module, &krate.root_module.items);

        self.current_path.pop();
        self.current_crate_version = None;
    }

    fn render_crate_sidebar(&mut self, workspace: &Workspace, krate: &Crate) {
        self.render_module_items_sidebar("Crate items", &krate.root_module);

        self.h3("Crates");
        self.output.push_str("<ul class=\"sidebar-list\">\n");
        for krate in &workspace.crates {
            if !krate.root_module.has_public_items() {
                continue;
            }

            self.output.push_str("<li>");
            self.output.push_str(&format!("<a href=\"../{}\">{}</a>", krate.uri(), krate.name(),));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn render_items(&mut self, items: &[(ItemVisibility, Item)], sidebar: bool, nesting: usize) {
        if !sidebar {
            self.render_reexports(items);
        }
        self.render_contracts(items, sidebar, nesting);
        self.render_modules(items, sidebar, nesting);
        self.render_structs(items, sidebar, nesting);
        self.render_traits(items, sidebar, nesting);
        self.render_type_aliases(items, sidebar, nesting);
        self.render_primitive_types(items, sidebar, nesting);
        self.render_functions(items, sidebar, nesting);
        self.render_globals(items, sidebar, nesting);
    }

    fn render_reexports(&mut self, items: &[(ItemVisibility, Item)]) {
        let reexports = get_reexports(items);
        if !reexports.is_empty() {
            self.render_reexports_list("Re-exports", "re-exports", &reexports);
        }
    }

    fn render_contracts(
        &mut self,
        items: &[(ItemVisibility, Item)],
        sidebar: bool,
        nesting: usize,
    ) {
        let modules = get_contracts(items);
        if !modules.is_empty() {
            self.render_list("Contracts", "contracts", sidebar, nesting, &modules);
        }
    }

    fn render_modules(&mut self, items: &[(ItemVisibility, Item)], sidebar: bool, nesting: usize) {
        let modules = get_modules(items);
        if !modules.is_empty() {
            self.render_list("Modules", "modules", sidebar, nesting, &modules);
        }
    }

    fn render_structs(&mut self, items: &[(ItemVisibility, Item)], sidebar: bool, nesting: usize) {
        let structs = get_structs(items);
        if !structs.is_empty() {
            self.render_list("Structs", "structs", sidebar, nesting, &structs);
        }
    }

    fn render_traits(&mut self, items: &[(ItemVisibility, Item)], sidebar: bool, nesting: usize) {
        let traits = get_traits(items);
        if !traits.is_empty() {
            self.render_list("Traits", "traits", sidebar, nesting, &traits);
        }
    }

    fn render_type_aliases(
        &mut self,
        items: &[(ItemVisibility, Item)],
        sidebar: bool,
        nesting: usize,
    ) {
        let type_aliases = get_type_aliases(items);
        if !type_aliases.is_empty() {
            self.render_list("Type aliases", "type-aliases", sidebar, nesting, &type_aliases);
        }
    }

    fn render_primitive_types(
        &mut self,
        items: &[(ItemVisibility, Item)],
        sidebar: bool,
        nesting: usize,
    ) {
        let primitive_types = get_primitive_types(items);
        if !primitive_types.is_empty() {
            self.render_list(
                "Primitive types",
                "primitive-types",
                sidebar,
                nesting,
                &primitive_types,
            );
        }
    }

    fn render_globals(&mut self, items: &[(ItemVisibility, Item)], sidebar: bool, nesting: usize) {
        let globals = get_globals(items);
        if !globals.is_empty() {
            self.render_list("Globals", "globals", sidebar, nesting, &globals);
        }
    }

    fn render_functions(
        &mut self,
        items: &[(ItemVisibility, Item)],
        sidebar: bool,
        nesting: usize,
    ) {
        let functions = get_functions(items);
        if !functions.is_empty() {
            self.render_list("Functions", "functions", sidebar, nesting, &functions);
        }
    }

    fn render_list<T: ItemProperties + HasUri + HasClass>(
        &mut self,
        title: &str,
        anchor: &str,
        sidebar: bool,
        nesting: usize,
        items: &[&T],
    ) {
        if sidebar {
            self.output.push_str(&format!("<h3>{title}</h3>"));
        } else {
            self.output.push_str(&format!("<h2 id=\"{anchor}\">{title}</h2>"));
        }
        self.output.push_str("<ul class=\"item-list\">\n");

        let mut items = items.to_vec();
        items.sort_by_key(|item| item.name().to_lowercase());

        for item in items {
            self.output.push_str("<li>");
            if !sidebar {
                self.output.push_str("<div class=\"item-name\">");
            }
            let class =
                if sidebar { String::new() } else { format!(" class=\"{}\"", item.class()) };
            self.output.push_str(&format!(
                "<a href=\"{}{}\"{}>{}</a>",
                "../".repeat(nesting),
                item.uri(),
                class,
                item.name(),
            ));
            if !sidebar {
                if item.is_deprecated() {
                    self.output.push_str("\n<span class=\"deprecated\">Deprecated</span>\n");
                }
                self.output.push_str("</div>");
                self.output.push_str("<div class=\"item-description\">");
                if let Some((comments, links)) = item.comments() {
                    let comments = self.process_comments_links(links, comments.clone());
                    let comments = colorize_markdown_code_blocks(comments);

                    let summary = markdown_summary(&comments);

                    let markdown = markdown_utils::to_html(&summary);
                    let markdown = markdown.trim_start_matches("<p>");
                    let summary = markdown.trim().trim_end_matches("</p>").trim();

                    self.output.push_str(summary);
                }
            }
            self.output.push_str("</div>");
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn render_reexports_list(&mut self, title: &str, anchor: &str, reexports: &[&Reexport]) {
        let nesting = self.current_path.len();

        self.output.push_str(&format!("<h2 id=\"{anchor}\">{title}</h2>"));
        self.output.push_str("<ul class=\"item-list\">\n");

        for reexport in reexports {
            let info = &self.id_to_info[&reexport.id];
            let path = info.path.join("::");
            let path = if path.is_empty() { String::new() } else { format!("{path}::") };
            let uri = &info.uri;
            let class = info.class;
            self.output.push_str("<li>");
            self.output.push_str("<div class=\"item-name\">");
            self.output.push_str("<code>");
            self.output.push_str(&format!(
                "pub use {}<a href=\"{}{}\" class=\"{}\">{}</a>{};",
                path,
                "../".repeat(nesting),
                uri,
                class,
                reexport.item_name,
                if reexport.name != reexport.item_name {
                    format!(" as {}", reexport.name)
                } else {
                    String::new()
                },
            ));
            self.output.push_str("</code>");
            self.output.push_str("</div>");
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn create_items(&mut self, parent_module: &Module, items: &[(ItemVisibility, Item)]) {
        for (visibility, item) in items {
            if visibility == &ItemVisibility::Public {
                self.create_item(parent_module, item);
            }
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
            Item::PrimitiveType(primitive) => self.create_primitive_type(parent_module, primitive),
            Item::Reexport(_) => { /* Re-exports don't have their own pages */ }
        }
    }

    fn create_module(&mut self, parent_module: &Module, module: &Module) {
        self.current_path.push(module.name.clone());

        let kind = if module.is_contract { "Contract" } else { "Module" };

        self.html_start(&format!("{kind} {}", module.name), &module.name);
        self.sidebar_start();
        self.render_module_sidebar(parent_module, module);
        self.sidebar_end();
        self.main_start(false);
        self.h1(&format!("{kind} <span id=\"mod\" class=\"module\">{}</span>", module.name));
        self.render_comments(module.comments.as_ref(), 1);
        self.render_items(&module.items, false, 0);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from("index.html"));

        self.create_items(module, &module.items);

        self.current_path.pop();
    }

    fn render_module_sidebar(&mut self, parent_module: &Module, module: &Module) {
        let kind = if module.is_contract { "Contract" } else { "Module" };
        self.h2(&format!("<a href=\"#mod\">{kind} {}</a>", module.name));
        self.render_module_items_sidebar(&format!("{kind} items"), module);
        self.render_module_contents_sidebar(parent_module, 1);
    }

    fn render_module_items_sidebar(&mut self, title: &str, module: &Module) {
        if !module.has_public_items() {
            return;
        }
        self.h3(title);
        self.output.push_str("<ul class=\"sidebar-list\">\n");
        if !get_reexports(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#re-exports\">Re-exports</a></li>\n");
        }
        if !get_contracts(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#contracts\">Contracts</a></li>\n");
        }
        if !get_modules(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#modules\">Modules</a></li>\n");
        }
        if !get_structs(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#structs\">Structs</a></li>\n");
        }
        if !get_traits(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#traits\">Traits</a></li>\n");
        }
        if !get_type_aliases(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#type-aliases\">Type aliases</a></li>\n");
        }
        if !get_functions(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#functions\">Functions</a></li>\n");
        }
        if !get_globals(&module.items).is_empty() {
            self.output.push_str("<li><a href=\"#globals\">Globals</a></li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn render_module_contents_sidebar(&mut self, module: &Module, nesting: usize) {
        if module.name.is_empty() {
            let crate_name = self.current_path.last().unwrap();
            self.h2(&format!("In crate {crate_name}"));
        } else {
            self.h2(&format!("In module {}", module.name));
        }
        self.render_items(&module.items, true, nesting);
    }

    fn create_struct(&mut self, parent_module: &Module, struct_: &Struct) {
        self.html_start(&format!("Struct {}", struct_.name), &struct_.name);
        self.sidebar_start();
        self.render_struct_sidebar(struct_);
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!("Struct <span id=\"struct\" class=\"struct\">{}</span>", struct_.name));
        self.render_struct_code(struct_);
        self.render_comments(struct_.comments.as_ref(), 1);
        self.render_struct_fields(&struct_.fields);
        self.render_impls(&struct_.impls);

        let mut trait_impls = struct_.trait_impls.clone();
        trait_impls.sort_by_key(|trait_impl| {
            (trait_impl_trait_to_string(trait_impl), trait_impl_anchor(trait_impl))
        });
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
            self.output.push_str("<ul class=\"sidebar-list\">\n");
            for field in fields {
                self.output.push_str("<li>");
                self.output.push_str(&format!(
                    "<a href=\"#structfield.{}\">{}</a>", // cSpell:disable-line
                    field.name, field.name
                ));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>\n");
        }

        let mut methods = struct_.impls.iter().flat_map(|iter| &iter.methods).collect::<Vec<_>>();
        methods.sort_by_key(|method| method.name.clone());

        if !methods.is_empty() {
            self.h3("Methods");
            self.output.push_str("<ul class=\"sidebar-list\">\n");
            for method in methods {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", method.name, method.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>\n");
        }

        self.render_sidebar_trait_impls("Trait implementations", true, &struct_.trait_impls);
    }

    fn create_trait(&mut self, parent_module: &Module, trait_: &Trait) {
        self.html_start(&format!("Trait {}", trait_.name), &trait_.name);
        self.sidebar_start();
        self.render_trait_sidebar(trait_);
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!("Trait <span id=\"trait\" class=\"trait\">{}</span>", trait_.name));
        self.render_trait_code(trait_);
        self.render_comments(trait_.comments.as_ref(), 1);
        self.render_trait_methods("Required methods", &trait_.required_methods);
        self.render_trait_methods("Provided methods", &trait_.provided_methods);

        let mut trait_impls = self.get_all_trait_impls(trait_);
        trait_impls.sort_by_key(|trait_impl| {
            (self.type_to_string(&trait_impl.r#type), trait_impl_anchor(trait_impl))
        });
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
            self.output.push_str("<ul class=\"sidebar-list\">\n");
            for method in methods {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", method.name, method.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>\n");
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
            trait_impls.sort_by_key(|trait_impl| {
                (trait_impl_trait_to_string(trait_impl), trait_impl_anchor(trait_impl))
            });
        } else {
            trait_impls.sort_by_key(|trait_impl| {
                (self.type_to_string(&trait_impl.r#type), trait_impl_anchor(trait_impl))
            });
        }

        self.h3(title);
        self.output.push_str("<ul class=\"sidebar-list\">\n");
        for trait_impl in trait_impls {
            self.output.push_str("<li>");
            self.output.push_str(&format!(
                "<a href=\"#{}\">{}</a>",
                trait_impl_anchor(&trait_impl),
                if display_trait {
                    trait_impl_trait_to_string(&trait_impl)
                } else {
                    self.type_to_string(&trait_impl.r#type)
                },
            ));
            self.output.push_str("</li>\n");
        }
        self.output.push_str("</ul>\n");
    }

    fn create_alias(&mut self, parent_module: &Module, alias: &TypeAlias) {
        self.html_start(&format!("Type alias {}", alias.name), &alias.name);
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!("Type alias <span class=\"type\">{}</span>", alias.name));
        self.render_type_alias_code(alias);
        self.render_comments(alias.comments.as_ref(), 1);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(alias.uri()));
    }

    fn create_function(&mut self, parent_module: &Module, function: &Function) {
        self.html_start(&format!("Function {}", function.name), &function.name);
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!("Function <span class=\"fn\">{}</span>", function.name));
        let as_header = false;
        let link = false;
        let output_id = false;
        self.render_function(function, 1, as_header, link, output_id);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(function.uri()));
    }

    fn create_global(&mut self, parent_module: &Module, global: &Global) {
        self.html_start(&format!("Global {}", global.name), &global.name);
        self.sidebar_start();
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!("Global <span class=\"global\">{}</span>", global.name));
        self.render_global_code(global);
        self.render_comments(global.comments.as_ref(), 1);
        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(global.uri()));
    }

    fn create_primitive_type(&mut self, parent_module: &Module, primitive: &PrimitiveType) {
        self.html_start(&format!("Primitive type {}", primitive.kind), &primitive.kind.to_string());
        self.sidebar_start();
        self.render_primitive_sidebar(primitive);
        self.render_module_contents_sidebar(parent_module, 0);
        self.sidebar_end();
        self.main_start(true);
        self.h1(&format!(
            "Primitive type <span id=\"primitive\" class=\"primitive\">{}</span>",
            primitive.kind
        ));
        self.render_comments(primitive.comments.as_ref(), 1);
        self.render_impls(&primitive.impls);

        let mut trait_impls = primitive.trait_impls.clone();
        trait_impls.sort_by_key(|trait_impl| {
            (trait_impl_trait_to_string(trait_impl), trait_impl_anchor(trait_impl))
        });
        let show_methods = true;
        self.render_trait_impls(&trait_impls, "Trait implementations", show_methods);

        self.main_end();
        self.html_end();
        self.push_file(PathBuf::from(primitive.kind.uri()));
    }

    fn render_primitive_sidebar(&mut self, primitive: &PrimitiveType) {
        self.h2(&format!("<a href=\"#primitive\">Primitive type {}</a>", primitive.kind));

        let mut methods = primitive.impls.iter().flat_map(|iter| &iter.methods).collect::<Vec<_>>();
        methods.sort_by_key(|method| method.name.clone());

        if !methods.is_empty() {
            self.h3("Methods");
            self.output.push_str("<ul class=\"sidebar-list\">\n");
            for method in methods {
                self.output.push_str("<li>");
                self.output.push_str(&format!("<a href=\"#{}\">{}</a>", method.name, method.name));
                self.output.push_str("</li>\n");
            }
            self.output.push_str("</ul>\n");
        }

        self.render_sidebar_trait_impls("Trait implementations", true, &primitive.trait_impls);
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
                "<div id=\"structfield.{}\" class=\"struct-field\"><code class=\"code-header\">", // cSpell:disable-line
                field.name
            ));
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.render_type(&field.r#type);
            self.output.push_str("</code></div>\n");
            self.output.push_str("<div class=\"padded-description\">");
            self.render_comments(field.comments.as_ref(), 3);
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

        self.self_type = Some(impl_.r#type.clone());
        let link = true;
        self.render_methods(&impl_.methods, 3, output_id, link);
        self.self_type = None;
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
        self.render_id_reference(&trait_impl.trait_id, &trait_impl.trait_name);
        self.render_generic_types(&trait_impl.trait_generics);
        self.output.push_str(" for ");
        self.render_type(&trait_impl.r#type);
        let indent = 0;
        self.render_where_clause(&trait_impl.where_clause, indent);
        self.output.push_str("</code></h3>\n\n");

        if show_methods {
            self.self_type = Some(trait_impl.r#type.clone());
            let output_id = false;
            let link = false;
            self.render_methods(&trait_impl.methods, 3, output_id, link);
            self.self_type = None;
        }
    }

    fn render_trait_code(&mut self, trait_: &Trait) {
        self.output.push_str("<pre><code>");
        self.output.push_str(&format!("pub trait {}", trait_.name));
        self.render_generics(&trait_.generics);
        if !trait_.bounds.is_empty() {
            self.output.push(':');
            for (index, bound) in trait_.bounds.iter().enumerate() {
                if trait_.bounds.len() == 1 {
                    self.output.push(' ');
                } else {
                    self.output.push_str("\n    ");
                }
                if index > 0 {
                    self.output.push_str("+ ");
                }
                self.render_trait_bound(bound);
            }
        }
        let indent = 0;
        self.render_where_clause(&trait_.where_clause, indent);
        if trait_.where_clause.is_empty() {
            if trait_.bounds.len() <= 1 {
                self.output.push_str(" {\n");
            } else {
                self.output.push_str("\n{\n");
            }
        } else {
            self.output.push_str("{\n");
        }

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
        let link = true;
        self.render_methods(methods, 2, output_id, link);
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
        link: bool,
    ) {
        self.output.push_str("<div class=\"padded-methods\">");
        for method in methods {
            self.render_function(method, current_heading_level, true, link, output_id);
        }
        self.output.push_str("</div>");
    }

    fn render_function(
        &mut self,
        function: &Function,
        current_heading_level: usize,
        as_header: bool,
        link: bool,
        output_id: bool,
    ) {
        self.render_function_signature(function, as_header, link, output_id);

        let pad = as_header && (function.is_deprecated() || function.comments.is_some());
        if pad {
            self.output.push_str("<div class=\"padded-description\">");
        }

        if let Some(deprecated) = &function.deprecated {
            self.output.push_str("<div class=\"deprecated\">\n");
            self.output.push_str("<span class=\"emoji\">👎</span>\nDeprecated");
            if let Some(msg) = deprecated {
                self.output.push_str(": ");
                self.output.push_str(msg);
            }
            self.output.push_str("\n</div>\n");
        }

        if function.comments.is_some() {
            self.render_comments(function.comments.as_ref(), current_heading_level);
        }

        if pad {
            self.output.push_str("</div>");
        }
    }

    fn render_function_signature(
        &mut self,
        function: &Function,
        as_header: bool,
        link: bool,
        output_id: bool,
    ) {
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

        // Split params into multiple lines if the signature will likely be too long
        // to fit in one line.
        let use_newlines =
            function_signature_to_string(function, self.self_type.as_ref()).len() >= 90;
        for (index, param) in function.params.iter().enumerate() {
            if index > 0 && !use_newlines {
                self.output.push_str(", ");
            }
            if use_newlines {
                self.output.push('\n');
                self.output.push_str(&" ".repeat(4 * (indent + 1)));
            }
            if param.mut_ref {
                self.output.push_str("&mut ");
            }
            self.output.push_str(&param.name);

            if !is_self_param(param, self.self_type.as_ref()) {
                self.output.push_str(": ");
                self.render_type(&param.r#type);
            }

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
                self.output.push_str("let ");
                self.output.push_str(&generic.name);
                self.output.push_str(": ");
                self.render_type(numeric);
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
        self.output.push_str("<div class=\"where-clause\">");
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
        self.output.push_str("</div>");
    }

    fn render_trait_bound(&mut self, bound: &TraitBound) {
        self.render_id_reference(&bound.trait_id, &bound.trait_name);
        self.render_trait_generics(&bound.ordered_generics, &bound.named_generics);
    }

    fn render_id_reference(&mut self, id: &ItemId, name: &str) {
        if let Some(ItemInfo { path: _, uri, class, visibility: ItemVisibility::Public }) =
            self.id_to_info.get(id)
        {
            let nesting = self.current_path.len();
            self.output.push_str(&format!(
                "<a href=\"{}{}\" class=\"{class}\">{name}</a>",
                "../".repeat(nesting),
                uri,
            ));
        } else {
            self.output.push_str(name);
        }
    }

    fn render_type(&mut self, typ: &Type) {
        if let Some(self_type) = &self.self_type {
            if self_type == typ {
                self.output.push_str("Self");
                return;
            }
        }

        match typ {
            Type::Unit => self.output.push_str("()"),
            Type::Primitive(primitive) => {
                let nesting = self.current_path.len();
                self.output.push_str(&format!(
                    "<a href=\"{}std/{}\" class=\"primitive\">{}</a>",
                    "../".repeat(nesting),
                    primitive.uri(),
                    primitive
                ));
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
                let nesting = self.current_path.len();
                let primitive = PrimitiveTypeKind::Str;
                self.output.push_str(&format!(
                    "<a href=\"{}std/{}\" class=\"primitive\">{}</a>",
                    "../".repeat(nesting),
                    primitive.uri(),
                    primitive
                ));
                self.output.push_str("&lt;");
                self.render_type(length);
                self.output.push_str("&gt;");
            }
            Type::FmtString { length, element } => {
                let nesting = self.current_path.len();
                let primitive = PrimitiveTypeKind::Fmtstr;
                self.output.push_str(&format!(
                    "<a href=\"{}std/{}\" class=\"primitive\">{}</a>",
                    "../".repeat(nesting),
                    primitive.uri(),
                    primitive
                ));
                self.output.push_str("&lt;");
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
                self.render_id_reference(id, name);
                self.render_generic_types(generics);
            }
            Type::TypeAlias { id, name, generics } => {
                self.render_id_reference(id, name);
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
                self.render_id_reference(trait_id, trait_name);
                self.render_trait_generics(ordered_generics, named_generics);
            }
        }
    }

    fn type_to_string(&self, typ: &Type) -> String {
        type_to_string(typ, self.self_type.as_ref())
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
    fn render_comments(&mut self, comments: Option<&Comments>, current_heading_level: usize) {
        let Some((comments, links)) = comments else {
            return;
        };

        let comments = fix_markdown(comments, current_heading_level);
        let comments = self.process_comments_links(links, comments);
        let comments = colorize_markdown_code_blocks(comments);

        let html = markdown_utils::to_html(&comments);
        self.output.push_str("<div class=\"comments\">\n");
        self.output.push_str(&html);
        self.output.push_str("</div>\n");
    }

    /// Replace links in the given comments with proper HTML links to pages and anchors
    /// related to target items.
    fn process_comments_links(&self, links: &Links, comments: String) -> String {
        if links.is_empty() {
            return comments;
        }

        let mut lines = comments.lines().map(|line| line.to_string()).collect::<Vec<_>>();
        for link in links.iter().rev() {
            let target = &link.target;
            let name = &link.name;
            let id = target.id();
            let anchor = target
                .name()
                .map(|name| {
                    if matches!(target, LinkTarget::StructMember(..)) {
                        format!("#structfield.{name}") // cSpell:disable
                    } else {
                        format!("#{name}")
                    }
                })
                .unwrap_or_default();
            let mut line = lines[link.line].to_string();
            if let Some(id) = id {
                if let Some(ItemInfo {
                    path: _,
                    uri,
                    class: _,
                    visibility: ItemVisibility::Public,
                }) = self.id_to_info.get(id)
                {
                    let nesting = "../".repeat(self.current_path.len());
                    let replacement = format!("[{name}]({nesting}{uri}{anchor})");
                    line.replace_range(link.start..link.end, &replacement);
                }
            }
            if let Some(primitive_type) = target.primitive_type() {
                let nesting = "../".repeat(self.current_path.len());
                let uri = primitive_type.uri();
                let replacement = format!("[{name}]({nesting}std/{uri}{anchor})");
                line.replace_range(link.start..link.end, &replacement);
            }

            lines[link.line] = line;
        }

        lines.join("\n")
    }

    fn render_breadcrumbs(&mut self, last_is_link: bool) {
        self.output.push_str("<div>");
        let mut nesting = self.current_path.len();
        self.output.push_str(
            format!("<a href=\"{}index.html\">{}</a>", "../".repeat(nesting), self.workspace_name)
                .as_str(),
        );
        if !self.current_path.is_empty() {
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
        }
        self.output.push_str("</div>\n");
    }

    fn get_all_trait_impls(&self, trait_: &Trait) -> Vec<TraitImpl> {
        self.all_trait_impls
            .get(&trait_.id)
            .map(|impls| impls.iter().cloned().collect())
            .unwrap_or_else(|| trait_.trait_impls.clone())
    }

    fn html_start(&mut self, title: &str, short_title: &str) {
        self.html_start_with_redirect(title, short_title, None);
    }

    fn html_start_with_redirect(
        &mut self,
        title: &str,
        short_title: &str,
        redirect: Option<String>,
    ) {
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
        self.output.push_str(&format!(
            "<script defer src=\"{}nargo_doc.js\"></script>\n",
            "../".repeat(nesting)
        ));
        self.output.push_str(&format!("<title>{title} documentation</title>\n"));
        self.output.push_str("</head>\n");
        self.output.push_str("<body>\n");
        self.output.push_str("<div id=\"sidebar-toggle\">\n");
        self.output.push_str("<button id=\"sidebar-toggle-button\"></button>\n");
        self.output.push_str(&format!("<div id=\"sidebar-toggle-title\">{short_title}</div>\n"));
        self.output.push_str("</div>\n");
        self.output.push_str("<span id=\"main-contents\">\n");
    }

    fn html_end(&mut self) {
        self.output.push_str("</span>\n");
        self.output.push_str("</body>\n");
        self.output.push_str("</html>\n");
    }

    fn main_start(&mut self, last_breadcrumb_is_link: bool) {
        self.output.push_str("<main>\n");
        self.render_breadcrumbs(last_breadcrumb_is_link);
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
        if let Some(crate_name) = self.current_path.first().cloned() {
            self.output.push_str(&format!(
                "<h2 id=\"crate-name\"><a href=\"{}index.html\">{crate_name}</a></h2>\n",
                "../".repeat(nesting - 1)
            ));
            // If there's no version, use 0.0.0 by default to make it really obvious it's missing
            // (this is what rustdoc does too)
            let version = self.current_crate_version.clone().unwrap_or_else(|| "0.0.0".to_string());
            self.output
                .push_str(&format!("<div id=\"crate-version\">{}</div>\n", escape_html(&version)));
        }
    }

    fn sidebar_end(&mut self) {
        self.output.push_str("</nav>\n");
    }

    fn h1(&mut self, text: &str) {
        self.output.push_str(&format!("<h1>{text}</h1>\n"));
    }

    fn h2(&mut self, text: &str) {
        self.output.push_str(&format!("<h2>{text}</h2>\n"));
    }

    fn h3(&mut self, text: &str) {
        self.output.push_str(&format!("<h3>{text}</h3>\n"));
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

fn get_reexports(items: &[(ItemVisibility, Item)]) -> Vec<&Reexport> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Reexport(reexport) = item {
                    return Some(reexport);
                }
            }
            None
        })
        .collect()
}

fn get_modules(items: &[(ItemVisibility, Item)]) -> Vec<&Module> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Module(module) = item {
                    if !module.is_contract && module.has_public_items() {
                        return Some(module);
                    }
                }
            }
            None
        })
        .collect()
}

fn get_contracts(items: &[(ItemVisibility, Item)]) -> Vec<&Module> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Module(module) = item {
                    if module.is_contract && module.has_public_items() {
                        return Some(module);
                    }
                }
            }
            None
        })
        .collect()
}

fn get_structs(items: &[(ItemVisibility, Item)]) -> Vec<&Struct> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Struct(struct_) = item {
                    return Some(struct_);
                }
            }
            None
        })
        .collect()
}

fn get_traits(items: &[(ItemVisibility, Item)]) -> Vec<&Trait> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Trait(trait_) = item {
                    return Some(trait_);
                }
            }
            None
        })
        .collect()
}

fn get_type_aliases(items: &[(ItemVisibility, Item)]) -> Vec<&TypeAlias> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::TypeAlias(alias) = item {
                    return Some(alias);
                }
            }
            None
        })
        .collect()
}

fn get_primitive_types(items: &[(ItemVisibility, Item)]) -> Vec<&PrimitiveType> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::PrimitiveType(primitive_type) = item {
                    return Some(primitive_type);
                }
            }
            None
        })
        .collect()
}

fn get_globals(items: &[(ItemVisibility, Item)]) -> Vec<&Global> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Global(global) = item {
                    return Some(global);
                }
            }
            None
        })
        .collect()
}

fn get_functions(items: &[(ItemVisibility, Item)]) -> Vec<&Function> {
    items
        .iter()
        .filter_map(|(visibility, item)| {
            if visibility == &ItemVisibility::Public {
                if let Item::Function(function) = item {
                    return Some(function);
                }
            }
            None
        })
        .collect()
}

fn trait_impl_anchor(trait_impl: &TraitImpl) -> String {
    let mut string = String::new();
    string.push_str("impl-");
    string.push_str(&trait_impl_trait_to_string(trait_impl));
    string.push_str("-for-");
    string.push_str(&type_to_string(&trait_impl.r#type, None));
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
            string.push_str(&type_to_string(typ, None));
        }
        string.push_str("&gt;");
    }
    string
}

fn function_signature_to_string(function: &Function, self_type: Option<&Type>) -> String {
    let mut string = String::new();
    string.push_str("pub ");
    if function.unconstrained {
        string.push_str("unconstrained ");
    }
    if function.comptime {
        string.push_str("comptime ");
    }
    string.push_str("fn ");
    string.push_str(&function.name);
    string.push_str(&generics_to_string(&function.generics));
    string.push('(');
    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }
        if param.mut_ref {
            string.push_str("&mut ");
        }
        string.push_str(&param.name);
        if !is_self_param(param, self_type) {
            string.push_str(": ");
            string.push_str(&type_to_string(&param.r#type, self_type));
        }
    }
    string.push(')');
    if !matches!(function.return_type, Type::Unit) {
        string.push_str(" -> ");
        string.push_str(&type_to_string(&function.return_type, self_type));
    }
    string
}

fn generics_to_string(generics: &[Generic]) -> String {
    if generics.is_empty() {
        return String::new();
    }

    let mut string = String::new();
    string.push('<');
    for (index, generic) in generics.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }
        if let Some(numeric) = &generic.numeric {
            string.push_str(&format!("let {}: {}", generic.name, &type_to_string(numeric, None)));
        } else {
            string.push_str(&generic.name);
        }
    }
    string.push('>');
    string
}

fn type_to_string(typ: &Type, self_type: Option<&Type>) -> String {
    if let Some(self_type) = self_type {
        if self_type == typ {
            return "Self".to_string();
        }
    }

    match typ {
        Type::Unit => "()".to_string(),
        Type::Primitive(primitive) => primitive.to_string(),
        Type::Array { length, element } => {
            format!(
                "[{}; {}]",
                type_to_string(element, self_type),
                type_to_string(length, self_type)
            )
        }
        Type::Slice { element } => format!("[{}]", type_to_string(element, self_type)),
        Type::String { length } => format!("str&lt;{}&gt;", type_to_string(length, self_type)),
        Type::FmtString { length, element } => {
            format!(
                "fmtstr&lt;{}, {}&gt;",
                type_to_string(length, self_type),
                type_to_string(element, self_type)
            )
        }
        Type::Tuple(items) => {
            let items: Vec<String> =
                items.iter().map(|typ| type_to_string(typ, self_type)).collect();
            format!("({}{})", items.join(", "), if items.len() == 1 { "," } else { "" })
        }
        Type::Reference { r#type, mutable } => {
            if *mutable {
                format!("&mut {}", type_to_string(r#type, self_type))
            } else {
                format!("&{}", type_to_string(r#type, self_type))
            }
        }
        Type::Struct { name, generics, id: _ } | Type::TypeAlias { name, generics, id: _ } => {
            let mut string = String::new();
            string.push_str(name);
            if !generics.is_empty() {
                let generic_strings: Vec<String> =
                    generics.iter().map(|typ| type_to_string(typ, self_type)).collect();
                string.push_str(&format!("<{}>", generic_strings.join(", ")));
            }
            string
        }
        Type::Function { params, return_type, env, unconstrained } => {
            let mut string = String::new();
            if *unconstrained {
                string.push_str("unconstrained ");
            }
            string.push_str("fn");
            if !matches!(env.as_ref(), &Type::Unit) {
                string.push_str(&format!("[{}]", type_to_string(env, self_type)));
            }
            string.push('(');
            let param_strings: Vec<String> =
                params.iter().map(|typ| type_to_string(typ, self_type)).collect();
            string.push_str(&param_strings.join(", "));
            string.push(')');
            if !matches!(return_type.as_ref(), &Type::Unit) {
                string.push_str(" -> ");
                string.push_str(&type_to_string(return_type, self_type));
            }
            string
        }
        Type::Constant(value) => value.clone(),
        Type::Generic(name) => escape_html(name),
        Type::InfixExpr { lhs, operator, rhs } => {
            format!(
                "{}{}{}",
                type_to_string(lhs, self_type),
                operator,
                type_to_string(rhs, self_type)
            )
        }
        Type::TraitAsType { trait_name, ordered_generics, named_generics, trait_id: _ } => {
            let mut string = String::new();
            string.push_str("impl ");
            string.push_str(trait_name);
            if !ordered_generics.is_empty() || !named_generics.is_empty() {
                string.push('<');
                let mut first = true;
                for generic in ordered_generics {
                    if !first {
                        string.push_str(", ");
                    }
                    first = false;
                    string.push_str(&type_to_string(generic, self_type));
                }
                for (name, typ) in named_generics {
                    if !first {
                        string.push_str(", ");
                    }
                    first = false;
                    string.push_str(name);
                    string.push_str(" = ");
                    string.push_str(&type_to_string(typ, self_type));
                }
                string.push('>');
            }
            string
        }
    }
}

fn is_self_param(param: &FunctionParam, self_type: Option<&Type>) -> bool {
    if param.name != "self" {
        return false;
    }

    let Some(self_type) = self_type else {
        if let Type::Generic(generic) = &param.r#type {
            return generic == "Self";
        }
        return false;
    };

    if param.mut_ref {
        let Type::Reference { r#type, mutable: true } = &param.r#type else {
            return false;
        };
        r#type.as_ref() == self_type
    } else {
        &param.r#type == self_type
    }
}

pub(super) fn escape_html(input: &str) -> String {
    input.replace('<', "&lt;").replace('>', "&gt;")
}
