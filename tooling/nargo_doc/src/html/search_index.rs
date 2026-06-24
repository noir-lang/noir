use noirc_frontend::ast::ItemVisibility;

use crate::{
    html::{has_class::HasClass, has_uri::HasUri, markdown_utils::markdown_summary},
    items::{Comments, Item, ItemProperties, Workspace},
};

/// A single searchable item in the generated documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        }
        Item::Trait(trait_) => {
            push_entry(entries, current_path, &trait_.name, trait_.class(), &trait_.uri(), trait_);
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
        output.push_str("  {\"name\":");
        push_json_string(&mut output, &entry.name);
        output.push_str(",\"path\":");
        push_json_string(&mut output, &entry.path);
        output.push_str(",\"kind\":");
        push_json_string(&mut output, &entry.kind);
        output.push_str(",\"url\":");
        push_json_string(&mut output, &entry.url);
        output.push_str(",\"desc\":");
        push_json_string(&mut output, &entry.description);
        output.push_str("},\n");
    }
    output.push_str("];\n");
    output
}

/// Appends `string` to `output` as a JSON string literal, including the surrounding quotes.
fn push_json_string(output: &mut String, string: &str) {
    output.push('"');
    for c in string.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if (c as u32) < 0x20 => output.push_str(&format!("\\u{:04x}", c as u32)),
            c => output.push(c),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use noirc_arena::Arena;
    use noirc_errors::Location;
    use noirc_frontend::ast::ItemVisibility;
    use noirc_frontend::graph::CrateId;
    use noirc_frontend::hir::def_map::{LocalModuleId, ModuleId};

    use crate::items::{
        Comments, Crate, Function, Item, ItemId, ItemKind, Module, Struct, Type, Workspace,
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

    fn function_item(name: &str) -> Item {
        Item::Function(Function {
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
        })
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
