use noirc_frontend::ast::Documented;
use noirc_frontend::ast::LetStatement;
use noirc_frontend::ast::ModuleDeclaration;
use noirc_frontend::ast::NoirFunction;
use noirc_frontend::ast::NoirStruct;
use noirc_frontend::ast::NoirTrait;
use noirc_frontend::ast::NoirTypeAlias;
use noirc_frontend::parser::SortedModule;
use noirc_frontend::parser::SortedSubModule;

fn short_description(doc_comments: &[String]) -> String {
    doc_comments
        .iter()
        .take_while(|line| !line.is_empty())
        .fold(String::new(), |acc, line| format!("{acc} {line}"))
}

fn list_items(items: &[Metadata]) -> String {
    items.iter()
        .map(|item| format!("<li>{}</li>", item.title))
        .collect()
}

pub struct Metadata {
    pub title: String,
    pub short_description: String,
}

impl From<&Documented<NoirFunction>> for Metadata {
    fn from(value: &Documented<NoirFunction>) -> Self {
        Self {
            title: value.item.name().to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirStruct>> for Metadata {
    fn from(value: &Documented<NoirStruct>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirTrait>> for Metadata {
    fn from(value: &Documented<NoirTrait>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirTypeAlias>> for Metadata {
    fn from(value: &Documented<NoirTypeAlias>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<LetStatement>> for Metadata {
    fn from(value: &Documented<LetStatement>) -> Self {
        Self {
            title: value.item.pattern.name_ident().0.contents.clone(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<ModuleDeclaration>> for Metadata {
    fn from(value: &Documented<ModuleDeclaration>) -> Self {
        Self {
            title: value.item.ident.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<SortedSubModule>> for Metadata {
    fn from(value: &Documented<SortedSubModule>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
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
            docs: short_description(&module.inner_doc_comments),
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
        <!-- <link rel="icon" type="image/svg+xml" href="TODO" /> -->
    </head>
    <body>
        <nav class="sidebar">
            <a class="logo-container" href="TODO">
                {NOIR_LOGO}
                <h2>{crate_name}</h2>
            </a>
            <div>
                <h3>Module {module_name}</h3>
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
        <div>

        </div>
    </body>
    <style>
        * {{ padding: 0px; margin: 0px; }}
        body {{ color: #eeedf1; background-color: #321e4c; }}
    </style>
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

const NOIR_LOGO: &'static str = r#"<img src="TODO" alt="Noir Logo" />
"#;

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

        let module_dev_doc = ModuleDevDoc::new("noirc".to_string(), "module".to_string(), sorted_module);

        let html = module_dev_doc.module_html_template(crate_metadata);

        std::fs::write("test.html", html).unwrap();
    }
}
