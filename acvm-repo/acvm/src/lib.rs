#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod compiler;
pub mod pwg;

// Suppress unused-dep warning: bn254_blackbox_solver is used by the generate_acvm_js_fixtures bin.
#[cfg(feature = "generate-test-fixtures")]
use bn254_blackbox_solver as _;

pub use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use pwg::OpcodeResolutionError;

// re-export acir
pub use acir;
pub use acir::{AcirField, FieldElement};
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use acvm_blackbox_solver as blackbox_solver;
