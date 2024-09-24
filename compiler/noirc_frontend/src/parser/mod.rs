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
    NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, TypeImpl, UseTree,
};
use crate::token::{SecondaryAttribute, Token};

pub use errors::ParserError;
pub use errors::ParserErrorReason;
use noirc_errors::Span;
pub use parser::{parse_program, parse_result, Parser};

pub trait NoirParser<T> {}

#[derive(Debug, Clone)]
pub struct TopLevelStatement {
    pub kind: TopLevelStatementKind,
    pub doc_comments: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TopLevelStatementKind {
    Function(NoirFunction),
    Module(ModuleDeclaration),
    Import(UseTree, ItemVisibility),
    Struct(NoirStruct),
    Trait(NoirTrait),
    TraitImpl(NoirTraitImpl),
    Impl(TypeImpl),
    TypeAlias(NoirTypeAlias),
    SubModule(ParsedSubModule),
    Global(LetStatement),
    InnerAttribute(SecondaryAttribute),
    Error,
}

impl TopLevelStatementKind {
    pub fn into_item_kind(self) -> Option<ItemKind> {
        match self {
            TopLevelStatementKind::Function(f) => Some(ItemKind::Function(f)),
            TopLevelStatementKind::Module(m) => Some(ItemKind::ModuleDecl(m)),
            TopLevelStatementKind::Import(i, visibility) => Some(ItemKind::Import(i, visibility)),
            TopLevelStatementKind::Struct(s) => Some(ItemKind::Struct(s)),
            TopLevelStatementKind::Trait(t) => Some(ItemKind::Trait(t)),
            TopLevelStatementKind::TraitImpl(t) => Some(ItemKind::TraitImpl(t)),
            TopLevelStatementKind::Impl(i) => Some(ItemKind::Impl(i)),
            TopLevelStatementKind::TypeAlias(t) => Some(ItemKind::TypeAlias(t)),
            TopLevelStatementKind::SubModule(s) => Some(ItemKind::Submodules(s)),
            TopLevelStatementKind::Global(c) => Some(ItemKind::Global(c)),
            TopLevelStatementKind::InnerAttribute(a) => Some(ItemKind::InnerAttribute(a)),
            TopLevelStatementKind::Error => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct SortedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<Documented<NoirFunction>>,
    pub types: Vec<Documented<NoirStruct>>,
    pub traits: Vec<Documented<NoirTrait>>,
    pub trait_impls: Vec<NoirTraitImpl>,
    pub impls: Vec<TypeImpl>,
    pub type_aliases: Vec<Documented<NoirTypeAlias>>,
    pub globals: Vec<Documented<LetStatement>>,

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

        for global_const in &self.globals {
            write!(f, "{global_const}")?;
        }

        for type_ in &self.types {
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
                ItemKind::Struct(typ) => module.push_type(typ, item.doc_comments),
                ItemKind::Trait(noir_trait) => module.push_trait(noir_trait, item.doc_comments),
                ItemKind::TraitImpl(trait_impl) => module.push_trait_impl(trait_impl),
                ItemKind::Impl(r#impl) => module.push_impl(r#impl),
                ItemKind::TypeAlias(type_alias) => {
                    module.push_type_alias(type_alias, item.doc_comments);
                }
                ItemKind::Global(global) => module.push_global(global, item.doc_comments),
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
    Trait(NoirTrait),
    TraitImpl(NoirTraitImpl),
    Impl(TypeImpl),
    TypeAlias(NoirTypeAlias),
    Global(LetStatement),
    ModuleDecl(ModuleDeclaration),
    Submodules(ParsedSubModule),
    InnerAttribute(SecondaryAttribute),
}

/// A submodule defined via `mod name { contents }` in some larger file.
/// These submodules always share the same file as some larger ParsedModule
#[derive(Clone, Debug)]
pub struct ParsedSubModule {
    pub name: Ident,
    pub contents: ParsedModule,
    pub outer_attributes: Vec<SecondaryAttribute>,
    pub is_contract: bool,
}

impl ParsedSubModule {
    pub fn into_sorted(self) -> SortedSubModule {
        SortedSubModule {
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
    pub contents: SortedModule,
    pub outer_attributes: Vec<SecondaryAttribute>,
    pub is_contract: bool,
}

impl SortedModule {
    fn push_function(&mut self, func: NoirFunction, doc_comments: Vec<String>) {
        self.functions.push(Documented::new(func, doc_comments));
    }

    fn push_type(&mut self, typ: NoirStruct, doc_comments: Vec<String>) {
        self.types.push(Documented::new(typ, doc_comments));
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

    fn push_global(&mut self, global: LetStatement, doc_comments: Vec<String>) {
        self.globals.push(Documented::new(global, doc_comments));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Or,
    And,
    Xor,
    LessGreater,
    Shift,
    Sum,
    Product,
    Highest,
}

impl Precedence {
    // Higher the number, the higher(more priority) the precedence
    // XXX: Check the precedence is correct for operators
    fn token_precedence(tok: &Token) -> Option<Precedence> {
        let precedence = match tok {
            Token::Equal => Precedence::Lowest,
            Token::NotEqual => Precedence::Lowest,
            Token::Pipe => Precedence::Or,
            Token::Ampersand => Precedence::And,
            Token::Caret => Precedence::Xor,
            Token::Less => Precedence::LessGreater,
            Token::LessEqual => Precedence::LessGreater,
            Token::Greater => Precedence::LessGreater,
            Token::GreaterEqual => Precedence::LessGreater,
            Token::ShiftLeft => Precedence::Shift,
            Token::ShiftRight => Precedence::Shift,
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::Slash => Precedence::Product,
            Token::Star => Precedence::Product,
            Token::Percent => Precedence::Product,
            _ => return None,
        };

        assert_ne!(precedence, Precedence::Highest, "expression_with_precedence in the parser currently relies on the highest precedence level being uninhabited");
        Some(precedence)
    }

    /// Return the next higher precedence. E.g. `Sum.next() == Product`
    fn next(self) -> Self {
        use Precedence::*;
        match self {
            Lowest => Or,
            Or => Xor,
            Xor => And,
            And => LessGreater,
            LessGreater => Shift,
            Shift => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
        }
    }

    /// TypeExpressions only contain basic arithmetic operators and
    /// notably exclude `>` due to parsing conflicts with generic type brackets.
    fn next_type_precedence(self) -> Self {
        use Precedence::*;
        match self {
            Lowest => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
            other => unreachable!("Unexpected precedence level in type expression: {:?}", other),
        }
    }

    /// The operators with the lowest precedence still useable in type expressions
    /// are '+' and '-' with precedence Sum.
    fn lowest_type_precedence() -> Self {
        Precedence::Sum
    }
}

impl std::fmt::Display for TopLevelStatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelStatementKind::Function(fun) => fun.fmt(f),
            TopLevelStatementKind::Module(m) => m.fmt(f),
            TopLevelStatementKind::Import(tree, visibility) => {
                if visibility == &ItemVisibility::Private {
                    write!(f, "use {tree}")
                } else {
                    write!(f, "{visibility} use {tree}")
                }
            }
            TopLevelStatementKind::Trait(t) => t.fmt(f),
            TopLevelStatementKind::TraitImpl(i) => i.fmt(f),
            TopLevelStatementKind::Struct(s) => s.fmt(f),
            TopLevelStatementKind::Impl(i) => i.fmt(f),
            TopLevelStatementKind::TypeAlias(t) => t.fmt(f),
            TopLevelStatementKind::SubModule(s) => s.fmt(f),
            TopLevelStatementKind::Global(c) => c.fmt(f),
            TopLevelStatementKind::InnerAttribute(a) => write!(f, "#![{}]", a),
            TopLevelStatementKind::Error => write!(f, "error"),
        }
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
