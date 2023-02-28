//! This set of modules implements the name resolution pass which converts the AST into
//! the HIR. In comparison to the AST, the HIR is interned in the NodeInterner and attaches
//! DefinitionIds to each Variable AST node to link them up to their definition. In doing so,
//! this pass validates scoping requirements and is responsible for issuing name/definition
//! related errors including missing definitions and duplicate definitions. One aspect of
//! handling definition links is handling and tracking imports. Another result of this pass
//! is that all imports will be removed from the program, and the AST of each file will
//! be combined into a larger Hir stored in the NodeInterner and linked together via DefinitionIds.
//!
//! The pass is comprised of two parts. The first part - definition collection - is implemented
//! in dc_mod::collect_defs and is responsible for collecting all the public symbols used by
//! the program so that we can resolve them later without worrying about cyclic references or
//! variables that aren't defined yet.
//!
//! The second part of the pass is the actual name resolution. Name resolution is handled by
//! super::resolution::Resolvers which traverse the entirety of a definition, ensure all names
//! are defined and linked, and convert the definition into Hir.
//!
//! These passes are performed sequentially (along with type checking afterward) in dc_crate.
pub mod dc_crate;
pub mod dc_mod;
mod errors;
