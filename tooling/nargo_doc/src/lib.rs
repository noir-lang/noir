use noirc_frontend::parser::SortedModule;

mod metadata;
use metadata::Metadata;

fn list_items(items: &[Metadata]) -> String {
    items.iter().map(|item| format!("<li><a href='#todo'>{}</a></li>", item.title)).collect()
}

pub struct ModuleDevDoc {
    pub crate_name: String,
    pub name: String,
    pub docs: String,
    pub functions: Vec<Metadata>,
    pub structs: Vec<Metadata>,
    pub traits: Vec<Metadata>,
    pub type_aliases: Vec<Metadata>,
    pub globals: Vec<Metadata>,
    pub submodules: Vec<Metadata>,
}

impl ModuleDevDoc {
    pub fn new(crate_name: String, module_name: String, module: SortedModule) -> Self {
        Self {
            crate_name,
            name: module_name,
            docs: module.inner_doc_comments.join("<br/>"),
            functions: module.functions.iter().map(Metadata::from).collect(),
            structs: module.types.iter().map(Metadata::from).collect(),
            traits: module.traits.iter().map(Metadata::from).collect(),
            type_aliases: module.type_aliases.iter().map(Metadata::from).collect(),
            globals: module.globals.iter().map(|g| Metadata::from(&g.0)).collect(),
            submodules: module.submodules.iter().map(Metadata::from).collect(),
        }
    }

    pub fn module_html_template(&self, crate_metadata: Metadata) -> String {
        let crate_name = &self.crate_name;
        let crate_short_description = crate_metadata.short_description;
        let module_name = &self.name;
        let module_docs = &self.docs;

        let functions = list_items(&self.functions);
        let structs = list_items(&self.structs);
        let traits = list_items(&self.traits);
        let type_aliases = list_items(&self.type_aliases);
        let globals = list_items(&self.globals);
        let submodules = list_items(&self.submodules);

        format!(
            r#"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content="{crate_short_description}" />
    <title>{crate_name} - Noir</title>
    <!-- <link rel="icon" type="image/svg+xml" href='#todo' /> -->
</head>
<body>
    <nav class="sidebar">
        <a class="logo-container" href='#todo'>
            {NOIR_LOGO}
            <h2>{crate_name}</h2>
        </a>
        <div>
            <h3>{module_name}</h3>
            <h4>Functions</h4>
            <ul>{functions}</ul>
            <h4>Structs</h4>
            <ul>{structs}</ul>
            <h4>Traits</h4>
            <ul>{traits}</ul>
            <h4>Type Aliases</h4>
            <ul>{type_aliases}</ul>
            <h4>Globals</h4>
            <ul>{globals}</ul>
            <h4>Submodules</h4>
            <ul>{submodules}</ul>
        </div>
    </nav>
    <main>{module_docs}</main>
</body>
<style>{STYLES}</style>
</html>
<!--pui-70
#321e4c
pui-60
#514167
pui-50
#706383
pui-40
#8f869e
pui-30
#afa8ba
pui-20
#cecbd5
pui-10
#eeedf1 -->
"#
        )
    }
}

const STYLES: &'static str = r#"
* { padding: 0px; margin: 0px; }

body {
    color: #eeedf1;
    background-color: #321e4c;
    display: flex;
    font-family: monospace;
}

main {
    padding: 1rem;
}

.logo-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
}

.sidebar {
    background-color: #514167;
    min-height: 100vh;
    padding: 0 0.5rem;
}

.sidebar h4 {
    padding: 0.5rem 0;
}

.sidebar li {
    list-style-type: none;
    padding: 0.5rem 0;
}

.sidebar a {
    color: #eeedf1;
    text-decoration: none;
}

.sidebar a:hover {
    color: #afa8ba;
}

.sidebar a:active {
    color: #8f869e;
}
"#;

pub const SAUCE: &'static str = r#"
//! module docs
//!
//! this is a module for things
//!
//! ## This is a section
//!
//! ## This is another section

/// This is a function that does things
fn f0() {
    // -- snip --
}

/// This is another function that does other things
fn f1() {
    // -- snip --
}

/// This is a struct that has things
pub struct Thing {
    /// This is a thing
    thing: u64,
}

/// This is a trait that does things
pub trait DoThings {
    /// This is a function that does things
    fn do_things(self);
}

/// This is a type alias for a thing
pub type ThingAlias = Thing;

/// This is a global that does things
global thing: Field = 1;

/// This is a submodule that does things
mod submod {
    //! This is a submodule, but does it do things?
    // -- snip --
}

comptime mut global foo: Field = 1 + 1;
"#;

const NOIR_LOGO: &'static str = r#"<img src="TODO" />"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmp() {
        let (sorted_module, errs) = noirc_frontend::parse_program(&SAUCE);

        println!("{:#?}", errs);

        let sorted_module = sorted_module.into_sorted();

        let crate_metadata = Metadata {
            title: "noirc".to_string(),
            short_description: "noirc is a compiler for the Noir programming language".to_string(),
        };

        let module_dev_doc =
            ModuleDevDoc::new("my_crate".to_string(), "my_module".to_string(), sorted_module);

        let html = module_dev_doc.module_html_template(crate_metadata);

        std::fs::write("test.html", html).unwrap();
    }
}
