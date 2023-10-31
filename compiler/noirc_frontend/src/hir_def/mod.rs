//! Noir's Hir is the result of the name resolution step (defined in the
//! hir module) and is essentially identical to the Ast with some small transformations.
//! The HIR is the input to the name resolution pass, the type checking pass, and the
//! monomorphization pass.
//!
//! Name Resolution takes the AST as input and produces the initial Hir which strongly
//! resembles the Ast except:
//! - Variables now have DefinitionIDs that can be used to reference the definition that defines them.
//! - Modules & imports are removed in favor of these direct links to definitions
//! - Resolves names in UnresolvedTypes to produce resolved Types.
//!
//! Type checking takes the Hir and:
//! - Tags each DefinitionId with its type
//! - Replaces MethodCall nodes with FunctionCalls
//!
//! Finally, monomorphization takes the Hir and converts it to a monomorphized Ir
//! (monomorphization::ast::Expression).
pub mod expr;
pub mod function;
pub mod stmt;
pub mod traits;
pub mod types;
