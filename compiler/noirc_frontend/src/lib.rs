//! The noir compiler is separated into the following passes which are listed
//! in order in square brackets. The inputs and outputs of each pass are also given:
//!
//! Source file -[Lexing]-> Tokens -[Parsing]-> Ast -[Name Resolution]-> Hir -[Type Checking]-> Hir -[Monomorphization]-> Monomorphized Ast
//!
//! After the monomorphized ast is created, it is passed to the noirc_evaluator crate to convert it to SSA form,
//! perform optimizations, convert to ACIR and eventually prove/verify the program.
#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod ast;
pub mod debug;
pub mod elaborator;
pub mod graph;
pub mod lexer;
pub mod locations;
pub mod monomorphization;
pub mod node_interner;
pub mod parser;
pub mod resolve_locations;

pub mod hir;
pub mod hir_def;

// Lexer API
pub use lexer::token;

// Parser API
pub use parser::{parse_program, ParsedModule};

// Type API
pub use hir_def::types::*;

// Unit tests that involve all modules
pub mod tests;

// API for experimental macros feature
pub mod macros_api {

    pub use acvm::FieldElement;
    pub use fm::FileId;
    pub use noirc_errors::Span;

    pub use crate::graph::CrateId;
    pub use crate::hir::def_collector::errors::MacroError;
    pub use crate::hir_def::expr::{HirExpression, HirLiteral};
    pub use crate::hir_def::stmt::HirStatement;
    pub use crate::node_interner::{NodeInterner, StructId};
    pub use crate::parser::{parse_program, SortedModule};
    pub use crate::token::SecondaryAttribute;

    pub use crate::ast::{
        BlockExpression, CallExpression, CastExpression, Expression, ExpressionKind,
        FunctionReturnType, Ident, IndexExpression, ItemVisibility, LetStatement, Literal,
        MemberAccessExpression, MethodCallExpression, NoirFunction, Path, PathKind, Pattern,
        Statement, UnresolvedType, UnresolvedTypeData, Visibility,
    };
    pub use crate::ast::{
        ForLoopStatement, ForRange, FunctionDefinition, ImportStatement, NoirStruct, Param,
        PrefixExpression, Signedness, StatementKind, TypeImpl, UnaryOp,
    };
    pub use crate::hir::{def_map::ModuleDefId, Context as HirContext};
    pub use crate::{StructType, Type};

    /// Methods to process the AST before and after type checking
    pub trait MacroProcessor {
        /// Function to manipulate the AST before type checking has been completed.
        fn process_untyped_ast(
            &self,
            ast: SortedModule,
            crate_id: &CrateId,
            file_id: FileId,
            context: &HirContext,
        ) -> Result<SortedModule, (MacroError, FileId)>;

        /// Function to manipulate the AST after type checking has been completed.
        /// The AST after type checking has been done is called the HIR.
        fn process_typed_ast(
            &self,
            crate_id: &CrateId,
            context: &mut HirContext,
        ) -> Result<(), (MacroError, FileId)>;
    }
}
