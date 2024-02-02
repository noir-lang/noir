#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

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

/// Specifies the maximum width of the expressions which will be constrained.
///
/// Unbounded Expressions are useful if you are eventually going to pass the ACIR
/// into a proving system which supports R1CS.
///
/// Bounded Expressions are useful if you are eventually going to pass the ACIR
/// into a proving system which supports PLONK, where arithmetic expressions have a
/// finite fan-in.
#[derive(Debug, Clone, Copy)]
pub enum ExpressionWidth {
    Unbounded,
    Bounded { width: usize },
}

impl From<usize> for ExpressionWidth {
    fn from(width: usize) -> ExpressionWidth {
        if width == 0 {
            ExpressionWidth::Unbounded
        } else {
            ExpressionWidth::Bounded { width }
        }
    }
}
