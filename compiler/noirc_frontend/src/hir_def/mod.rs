//! Noir's HIR (High-Level Intermediate Representation) is produced by the elaboration pass
//! and represents a fully resolved and type-checked program.
//! The HIR is the input to the monomorphization pass.
//!
//! Information about how the AST is transformed into an HIR can be found in the [elaborator][crate::elaborator] module.
//!
//! Monomorphization will take in the the HIR produced by elaboration and convert it to a monomorphized IR.
pub mod expr;
pub mod function;
pub mod stmt;
pub mod traits;
pub mod types;
