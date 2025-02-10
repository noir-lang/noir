//! The parser is the second pass of the noir compiler.
//! The parser's job is to take the output of the lexer (a stream of tokens)
//! and parse it into a valid Abstract Syntax Tree (Ast). During this, the parser
//! validates the grammar of the program and returns parsing errors for any syntactically
//! invalid constructs (such as `fn fn fn`).
//!
//! This file is mostly helper functions and types for the parser. For the parser itself,
//! see parser.rs. The definition of the abstract syntax tree can be found in the `ast` folder.
mod errors;
mod labels;
#[allow(clippy::module_inception)]
mod parser;

use crate::ast::{
    Documented, Ident, ImportStatement, ItemVisibility, LetStatement, ModuleDeclaration,
    NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, TypeImpl,
    UseTree,
};
use crate::token::SecondaryAttribute;

pub use errors::ParserError;
pub use errors::ParserErrorReason;
use noirc_errors::Span;
pub use parser::{parse_program, Parser, StatementOrExpressionOrLValue};

#[derive(Clone, Default)]
pub struct SortedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<Documented<NoirFunction>>,
    pub structs: Vec<Documented<NoirStruct>>,
    pub enums: Vec<Documented<NoirEnumeration>>,
    pub traits: Vec<Documented<NoirTrait>>,
    pub trait_impls: Vec<NoirTraitImpl>,
    pub impls: Vec<TypeImpl>,
    pub type_aliases: Vec<Documented<NoirTypeAlias>>,
    pub globals: Vec<(Documented<LetStatement>, ItemVisibility)>,

    /// Module declarations like `mod foo;`
    pub module_decls: Vec<Documented<ModuleDeclaration>>,

    /// Full submodules as in `mod foo { ... definitions ... }`
    pub submodules: Vec<Documented<SortedSubModule>>,

    pub inner_attributes: Vec<SecondaryAttribute>,
    pub inner_doc_comments: Vec<String>,
}

impl std::fmt::Display for SortedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for decl in &self.module_decls {
            writeln!(f, "{decl};")?;
        }

        for import in &self.imports {
            write!(f, "{import}")?;
        }

        for (global_const, _visibility) in &self.globals {
            write!(f, "{global_const}")?;
        }

        for type_ in &self.structs {
            write!(f, "{type_}")?;
        }

        for function in &self.functions {
            write!(f, "{function}")?;
        }

        for impl_ in &self.impls {
            write!(f, "{impl_}")?;
        }

        for type_alias in &self.type_aliases {
            write!(f, "{type_alias}")?;
        }

        for submodule in &self.submodules {
            write!(f, "{submodule}")?;
        }

        Ok(())
    }
}

/// A ParsedModule contains an entire Ast for one file.
#[derive(Clone, Debug, Default)]
pub struct ParsedModule {
    pub items: Vec<Item>,
    pub inner_doc_comments: Vec<String>,
}

impl ParsedModule {
    pub fn into_sorted(self) -> SortedModule {
        let mut module = SortedModule::default();

        for item in self.items {
            match item.kind {
                ItemKind::Import(import, visibility) => module.push_import(import, visibility),
                ItemKind::Function(func) => module.push_function(func, item.doc_comments),
                ItemKind::Struct(typ) => module.push_struct(typ, item.doc_comments),
                ItemKind::Enum(typ) => module.push_enum(typ, item.doc_comments),
                ItemKind::Trait(noir_trait) => module.push_trait(noir_trait, item.doc_comments),
                ItemKind::TraitImpl(trait_impl) => module.push_trait_impl(trait_impl),
                ItemKind::Impl(r#impl) => module.push_impl(r#impl),
                ItemKind::TypeAlias(type_alias) => {
                    module.push_type_alias(type_alias, item.doc_comments);
                }
                ItemKind::Global(global, visibility) => {
                    module.push_global(global, visibility, item.doc_comments);
                }
                ItemKind::ModuleDecl(mod_name) => {
                    module.push_module_decl(mod_name, item.doc_comments);
                }
                ItemKind::Submodules(submodule) => {
                    module.push_submodule(submodule.into_sorted(), item.doc_comments);
                }
                ItemKind::InnerAttribute(attribute) => module.inner_attributes.push(attribute),
            }
        }

        module.inner_doc_comments = self.inner_doc_comments;

        module
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub doc_comments: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Import(UseTree, ItemVisibility),
    Function(NoirFunction),
    Struct(NoirStruct),
    Enum(NoirEnumeration),
    Trait(NoirTrait),
    TraitImpl(NoirTraitImpl),
    Impl(TypeImpl),
    TypeAlias(NoirTypeAlias),
    Global(LetStatement, ItemVisibility),
    ModuleDecl(ModuleDeclaration),
    Submodules(ParsedSubModule),
    InnerAttribute(SecondaryAttribute),
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemKind::Enum(e) => e.fmt(f),
            ItemKind::Function(fun) => fun.fmt(f),
            ItemKind::ModuleDecl(m) => m.fmt(f),
            ItemKind::Import(tree, visibility) => {
                if visibility == &ItemVisibility::Private {
                    write!(f, "use {tree}")
                } else {
                    write!(f, "{visibility} use {tree}")
                }
            }
            ItemKind::Trait(t) => t.fmt(f),
            ItemKind::TraitImpl(i) => i.fmt(f),
            ItemKind::Struct(s) => s.fmt(f),
            ItemKind::Impl(i) => i.fmt(f),
            ItemKind::TypeAlias(t) => t.fmt(f),
            ItemKind::Submodules(s) => s.fmt(f),
            ItemKind::Global(c, visibility) => {
                if visibility != &ItemVisibility::Private {
                    write!(f, "{visibility} ")?;
                }
                c.fmt(f)
            }
            ItemKind::InnerAttribute(a) => write!(f, "#![{}]", a),
        }
    }
}

/// A submodule defined via `mod name { contents }` in some larger file.
/// These submodules always share the same file as some larger ParsedModule
#[derive(Clone, Debug)]
pub struct ParsedSubModule {
    pub visibility: ItemVisibility,
    pub name: Ident,
    pub contents: ParsedModule,
    pub outer_attributes: Vec<SecondaryAttribute>,
    pub is_contract: bool,
}

impl ParsedSubModule {
    pub fn into_sorted(self) -> SortedSubModule {
        SortedSubModule {
            visibility: self.visibility,
            name: self.name,
            contents: self.contents.into_sorted(),
            outer_attributes: self.outer_attributes,
            is_contract: self.is_contract,
        }
    }
}

impl std::fmt::Display for SortedSubModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mod {} {{", self.name)?;

        for line in self.contents.to_string().lines() {
            write!(f, "\n    {line}")?;
        }

        write!(f, "\n}}")
    }
}

#[derive(Clone)]
pub struct SortedSubModule {
    pub name: Ident,
    pub visibility: ItemVisibility,
    pub contents: SortedModule,
    pub outer_attributes: Vec<SecondaryAttribute>,
    pub is_contract: bool,
}

impl SortedModule {
    fn push_function(&mut self, func: NoirFunction, doc_comments: Vec<String>) {
        self.functions.push(Documented::new(func, doc_comments));
    }

    fn push_struct(&mut self, typ: NoirStruct, doc_comments: Vec<String>) {
        self.structs.push(Documented::new(typ, doc_comments));
    }

    fn push_enum(&mut self, typ: NoirEnumeration, doc_comments: Vec<String>) {
        self.enums.push(Documented::new(typ, doc_comments));
    }

    fn push_trait(&mut self, noir_trait: NoirTrait, doc_comments: Vec<String>) {
        self.traits.push(Documented::new(noir_trait, doc_comments));
    }

    fn push_trait_impl(&mut self, trait_impl: NoirTraitImpl) {
        self.trait_impls.push(trait_impl);
    }

    fn push_impl(&mut self, r#impl: TypeImpl) {
        self.impls.push(r#impl);
    }

    fn push_type_alias(&mut self, type_alias: NoirTypeAlias, doc_comments: Vec<String>) {
        self.type_aliases.push(Documented::new(type_alias, doc_comments));
    }

    fn push_import(&mut self, import_stmt: UseTree, visibility: ItemVisibility) {
        self.imports.extend(import_stmt.desugar(None, visibility));
    }

    fn push_module_decl(&mut self, mod_decl: ModuleDeclaration, doc_comments: Vec<String>) {
        self.module_decls.push(Documented::new(mod_decl, doc_comments));
    }

    fn push_submodule(&mut self, submodule: SortedSubModule, doc_comments: Vec<String>) {
        self.submodules.push(Documented::new(submodule, doc_comments));
    }

    fn push_global(
        &mut self,
        global: LetStatement,
        visibility: ItemVisibility,
        doc_comments: Vec<String>,
    ) {
        self.globals.push((Documented::new(global, doc_comments), visibility));
    }
}

impl std::fmt::Display for ParsedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone().into_sorted().fmt(f)
    }
}

impl std::fmt::Display for ParsedSubModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone().into_sorted().fmt(f)
    }
}
