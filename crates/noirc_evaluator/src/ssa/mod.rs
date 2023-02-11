mod block;
mod builtin;
mod conditional;
mod context;
mod flatten;
mod function;
mod inline;
mod integer;
mod mem;
pub mod node;
mod optimizations;
mod ssa_form;

// Generate SSA
pub mod code_gen;

// Generate ACIR code
pub mod acir_gen;

// Optimizations
//
// CSE
mod anchor;
mod cse;
