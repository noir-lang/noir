use noirc_frontend::ast::ItemVisibility;
use serde::Serialize;

use crate::{
    html::{has_class::HasClass, has_uri::HasUri, markdown_utils::markdown_summary},
    items::{Comments, Impl, Item, ItemProperties, TraitImpl, Workspace},
};

/// A single searchable item in the generated documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct SearchEntry {
    /// The item's name, e.g. `BoundedVec`.
    pub(super) name: String,
    /// The fully qualified module path of the item, e.g. `std::collections::bounded_vec`.
    pub(super) path: String,
    /// The kind of the item, matching its CSS class, e.g. `struct`, `trait`, `fn`, `module`.
    pub(super) kind: String,
    /// The URL of the item's page, relative to the documentation root.
    pub(super) url: String,
    /// A short, single-line plain-text summary taken from the item's doc comments.
    #[serde(rename = "desc")]
    pub(super) description: String,
}

/// Gathers every searchable item across the workspace crates.
///
/// Items that are publicly re-exported from another crate are inlined into the re-exporting
/// module at conversion time (see `process_module_reexports`), so they are reachable here just
/// like items defined locally.
pub(super) fn compute_search_index(workspace: &Workspace) -> Vec<SearchEntry> {
    let mut entries = Vec::new();
    let mut current_path = Vec::new();

    for krate in &workspace.crates {
        if !krate.root_module.has_public_items() {
            continue;
        }

        current_path.push(krate.name.clone());
        for (visibility, item) in &krate.root_module.items {
            if visibility == &ItemVisibility::Public {
                gather_search_entries_in_item(item, &mut current_path, &mut entries);
            }
        }
        current_path.pop();
    }

    // Sort for a stable order. Members such as trait-impl methods are gathered in a
    // non-deterministic order, which would otherwise make the generated file differ between runs.
    // The order only affects how equally-ranked results tie-break at search time.
    entries.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.url.cmp(&b.url))
    });

    entries
}

fn gather_search_entries_in_item(
    item: &Item,
    current_path: &mut Vec<String>,
    entries: &mut Vec<SearchEntry>,
) {
    match item {
        Item::Module(module) => {
            push_entry(entries, current_path, &module.name, module.class(), &module.uri(), module);
            current_path.push(module.name.clone());
            for (visibility, item) in &module.items {
                if visibility == &ItemVisibility::Public {
                    gather_search_entries_in_item(item, current_path, entries);
                }
            }
            current_path.pop();
        }
        Item::Struct(struct_) => {
            push_entry(
                entries,
                current_path,
                &struct_.name,
                struct_.class(),
                &struct_.uri(),
                struct_,
            );

            let context = TypeContext::new(current_path, &struct_.name, &struct_.uri());
            for field in &struct_.fields {
                push_member_entry(
                    entries,
                    &context,
                    &field.name,
                    "field",
                    &format!("structfield.{}", field.name),
                    &field.comments,
                );
            }
            push_methods(entries, &context, &struct_.impls);
            push_trait_impl_methods(entries, &context, &struct_.trait_impls);
        }
        Item::Trait(trait_) => {
            push_entry(entries, current_path, &trait_.name, trait_.class(), &trait_.uri(), trait_);

            let context = TypeContext::new(current_path, &trait_.name, &trait_.uri());
            for method in trait_.required_methods.iter().chain(&trait_.provided_methods) {
                push_member_entry(
                    entries,
                    &context,
                    &method.name,
                    "method",
                    &method.name,
                    &method.comments,
                );
            }
        }
        Item::TypeAlias(type_alias) => {
            push_entry(
                entries,
                current_path,
                &type_alias.name,
                type_alias.class(),
                &type_alias.uri(),
                type_alias,
            );
        }
        Item::PrimitiveType(primitive_type) => {
            let name = primitive_type.kind.to_string();
            push_entry(
                entries,
                current_path,
                &name,
                primitive_type.class(),
                &primitive_type.uri(),
                primitive_type,
            );

            let context = TypeContext::new(current_path, &name, &primitive_type.uri());
            push_methods(entries, &context, &primitive_type.impls);
            push_trait_impl_methods(entries, &context, &primitive_type.trait_impls);
        }
        Item::Global(global) => {
            push_entry(entries, current_path, &global.name, global.class(), &global.uri(), global);
        }
        Item::Function(function) => {
            push_entry(
                entries,
                current_path,
                &function.name,
                function.class(),
                &function.uri(),
                function,
            );
        }
        Item::Reexport(_) => {}
    }
}

fn push_entry(
    entries: &mut Vec<SearchEntry>,
    current_path: &[String],
    name: &str,
    kind: &str,
    uri: &str,
    item: &dyn ItemProperties,
) {
    let path = current_path.join("::");
    let url = format!("{}/{uri}", current_path.join("/"));
    entries.push(SearchEntry {
        name: name.to_string(),
        path,
        kind: kind.to_string(),
        url,
        description: description(item.comments()),
    });
}

/// Identifies the type whose members are being indexed: its qualified path (e.g.
/// `std::collections::bounded_vec::BoundedVec`) and the URL of its page.
struct TypeContext {
    path: String,
    url: String,
}

impl TypeContext {
    fn new(current_path: &[String], type_name: &str, type_uri: &str) -> Self {
        Self {
            path: format!("{}::{type_name}", current_path.join("::")),
            url: format!("{}/{type_uri}", current_path.join("/")),
        }
    }
}

/// Indexes the methods of a type's inherent `impl` blocks. Each method's page is its type's
/// page, anchored at the method name.
fn push_methods(entries: &mut Vec<SearchEntry>, context: &TypeContext, impls: &[Impl]) {
    for impl_ in impls {
        for method in &impl_.methods {
            push_member_entry(
                entries,
                context,
                &method.name,
                "method",
                &method.name,
                &method.comments,
            );
        }
    }
}

/// Indexes the methods of a type's trait `impl` blocks. Trait-impl methods don't get their own
/// anchor, so they link to the surrounding `impl` block on the type's page.
fn push_trait_impl_methods(
    entries: &mut Vec<SearchEntry>,
    context: &TypeContext,
    trait_impls: &[TraitImpl],
) {
    for trait_impl in trait_impls {
        let anchor = super::trait_impl_anchor(trait_impl);
        for method in &trait_impl.methods {
            push_member_entry(entries, context, &method.name, "method", &anchor, &method.comments);
        }
    }
}

/// Adds an entry for a member (a field or method) of a type. Its qualified path includes the
/// type name, and its URL is the type's page anchored at `anchor`.
fn push_member_entry(
    entries: &mut Vec<SearchEntry>,
    context: &TypeContext,
    name: &str,
    kind: &str,
    anchor: &str,
    comments: &Option<Comments>,
) {
    entries.push(SearchEntry {
        name: name.to_string(),
        path: context.path.clone(),
        kind: kind.to_string(),
        url: format!("{}#{anchor}", context.url),
        description: description(comments.as_ref()),
    });
}

/// Returns a short, single-line plain-text description from an item's doc comments.
fn description(comments: Option<&Comments>) -> String {
    let Some((markdown, _links)) = comments else {
        return String::new();
    };

    markdown_summary(markdown).lines().next().unwrap_or("").trim().to_string()
}

/// Renders the search index as the contents of `search-index.js`.
///
/// The index is assigned to a global variable (rather than fetched) so that search works when
/// the documentation is opened directly from the filesystem, where `fetch` is blocked by the
/// browser's same-origin policy.
pub(super) fn render_search_index_js(entries: &[SearchEntry]) -> String {
    let mut output = String::from("window.searchIndex = [\n");
    for entry in entries {
        output.push_str("  ");
        output.push_str(
            &serde_json::to_string(entry).expect("a search entry is always serializable"),
        );
        output.push_str(",\n");
    }
    output.push_str("];\n");
    output
}

#[cfg(test)]
mod tests {
    use noirc_arena::Arena;
    use noirc_errors::Location;
    use noirc_frontend::ast::ItemVisibility;
    use noirc_frontend::graph::CrateId;
    use noirc_frontend::hir::def_map::{LocalModuleId, ModuleId};

    use crate::items::{
        Comments, Crate, Function, Impl, Item, ItemId, ItemKind, Module, Struct, StructField,
        Trait, Type, Workspace,
    };

    use super::*;

    fn dummy_item_id(name: &str, kind: ItemKind) -> ItemId {
        ItemId { location: Location::dummy(), kind, name: name.to_string() }
    }

    fn dummy_module_id() -> ModuleId {
        let mut arena = Arena::<()>::default();
        ModuleId { krate: CrateId::Root(0), local_id: LocalModuleId::new(arena.insert(())) }
    }

    fn struct_item(name: &str, comments: Option<Comments>) -> Item {
        Item::Struct(Struct {
            id: dummy_item_id(name, ItemKind::Struct),
            name: name.to_string(),
            generics: Vec::new(),
            fields: Vec::new(),
            has_private_fields: false,
            comptime: false,
            impls: Vec::new(),
            trait_impls: Vec::new(),
            comments,
        })
    }

    fn function(name: &str) -> Function {
        Function {
            id: dummy_item_id(name, ItemKind::Function),
            unconstrained: false,
            comptime: false,
            name: name.to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Type::Unit,
            where_clause: Vec::new(),
            comments: None,
            deprecated: None,
        }
    }

    fn function_item(name: &str) -> Item {
        Item::Function(function(name))
    }

    fn field(name: &str) -> StructField {
        StructField { name: name.to_string(), r#type: Type::Unit, comments: None }
    }

    fn impl_with_method(method_name: &str) -> Impl {
        Impl {
            generics: Vec::new(),
            r#type: Type::Unit,
            where_clause: Vec::new(),
            methods: vec![function(method_name)],
            comments: None,
        }
    }

    fn module(name: &str, items: Vec<(ItemVisibility, Item)>) -> Module {
        Module {
            id: dummy_item_id(name, ItemKind::Module),
            module_id: dummy_module_id(),
            name: name.to_string(),
            items,
            comments: None,
            is_contract: false,
        }
    }

    fn workspace(crates: Vec<Crate>) -> Workspace {
        Workspace { name: "my_workspace".to_string(), crates, dependencies: Vec::new() }
    }

    fn krate(name: &str, items: Vec<(ItemVisibility, Item)>) -> Crate {
        Crate {
            name: name.to_string(),
            version: None,
            root_module: module(name, items),
            root_file: String::new(),
        }
    }

    #[test]
    fn collects_items_with_qualified_path_and_url() {
        let comments: Comments = ("The Foo struct.\n\nMore details.".to_string(), Vec::new());
        let submodule =
            Item::Module(module("submod", vec![(ItemVisibility::Public, function_item("bar"))]));
        let root = vec![
            (ItemVisibility::Public, struct_item("Foo", Some(comments))),
            (ItemVisibility::Public, submodule),
        ];

        let entries = compute_search_index(&workspace(vec![krate("mylib", root)]));

        let foo = entries.iter().find(|e| e.name == "Foo").expect("Foo should be indexed");
        assert_eq!(foo.path, "mylib");
        assert_eq!(foo.kind, "struct");
        assert_eq!(foo.url, "mylib/struct.Foo.html");
        assert_eq!(foo.description, "The Foo struct.");

        let submod = entries.iter().find(|e| e.name == "submod").expect("submod should be indexed");
        assert_eq!(submod.path, "mylib");
        assert_eq!(submod.kind, "module");
        assert_eq!(submod.url, "mylib/submod/index.html");

        let bar = entries.iter().find(|e| e.name == "bar").expect("bar should be indexed");
        assert_eq!(bar.path, "mylib::submod");
        assert_eq!(bar.kind, "fn");
        assert_eq!(bar.url, "mylib/submod/fn.bar.html");
    }

    #[test]
    fn entries_are_sorted() {
        let root = vec![
            (ItemVisibility::Public, function_item("zebra")),
            (ItemVisibility::Public, function_item("alpha")),
            (ItemVisibility::Public, function_item("mango")),
        ];

        let entries = compute_search_index(&workspace(vec![krate("mylib", root)]));

        let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["alpha", "mango", "zebra"]);
    }

    #[test]
    fn excludes_private_items() {
        let root = vec![
            (ItemVisibility::Private, struct_item("Hidden", None)),
            (ItemVisibility::Public, struct_item("Shown", None)),
        ];

        let entries = compute_search_index(&workspace(vec![krate("mylib", root)]));

        assert!(entries.iter().any(|e| e.name == "Shown"));
        assert!(!entries.iter().any(|e| e.name == "Hidden"));
    }

    #[test]
    fn collects_fields_and_methods() {
        let foo = Struct {
            id: dummy_item_id("Foo", ItemKind::Struct),
            name: "Foo".to_string(),
            generics: Vec::new(),
            fields: vec![field("value")],
            has_private_fields: false,
            comptime: false,
            impls: vec![impl_with_method("increment")],
            trait_impls: Vec::new(),
            comments: None,
        };
        let doubler = Trait {
            id: dummy_item_id("Doubler", ItemKind::Trait),
            name: "Doubler".to_string(),
            generics: Vec::new(),
            bounds: Vec::new(),
            where_clause: Vec::new(),
            associated_types: Vec::new(),
            associated_constants: Vec::new(),
            required_methods: vec![function("double")],
            provided_methods: Vec::new(),
            trait_impls: Vec::new(),
            comments: None,
        };
        let root = vec![
            (ItemVisibility::Public, Item::Struct(foo)),
            (ItemVisibility::Public, Item::Trait(doubler)),
        ];

        let entries = compute_search_index(&workspace(vec![krate("mylib", root)]));

        let value = entries.iter().find(|e| e.name == "value").expect("field should be indexed");
        assert_eq!(value.path, "mylib::Foo");
        assert_eq!(value.kind, "field");
        assert_eq!(value.url, "mylib/struct.Foo.html#structfield.value");

        let increment =
            entries.iter().find(|e| e.name == "increment").expect("method should be indexed");
        assert_eq!(increment.path, "mylib::Foo");
        assert_eq!(increment.kind, "method");
        assert_eq!(increment.url, "mylib/struct.Foo.html#increment");

        let double =
            entries.iter().find(|e| e.name == "double").expect("trait method should be indexed");
        assert_eq!(double.path, "mylib::Doubler");
        assert_eq!(double.kind, "method");
        assert_eq!(double.url, "mylib/trait.Doubler.html#double");
    }

    #[test]
    fn renders_index_as_escaped_json() {
        let entries = vec![SearchEntry {
            name: "Foo".to_string(),
            path: "mylib".to_string(),
            kind: "struct".to_string(),
            url: "mylib/struct.Foo.html".to_string(),
            description: "Says \"hi\".\nLine two\\path".to_string(),
        }];

        let js = render_search_index_js(&entries);

        assert!(js.starts_with("window.searchIndex = [\n"));
        assert!(js.trim_end().ends_with("];"));
        assert!(js.contains(r#""name":"Foo""#));
        assert!(js.contains(r#""url":"mylib/struct.Foo.html""#));
        assert!(js.contains(r#""desc":"Says \"hi\".\nLine two\\path""#));
    }
}
