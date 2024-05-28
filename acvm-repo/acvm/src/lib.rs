#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod compiler;
pub mod pwg;

pub use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use pwg::OpcodeResolutionError;

// re-export acir
pub use acir;
pub use acir::{AcirField, FieldElement};
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use acvm_blackbox_solver as blackbox_solver;
