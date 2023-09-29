#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

pub mod compiler;
pub mod pwg;

pub use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use core::fmt::Debug;
use pwg::OpcodeResolutionError;

// re-export acir
pub use acir;
pub use acir::FieldElement;
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use acvm_blackbox_solver as blackbox_solver;

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone, Copy)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}
