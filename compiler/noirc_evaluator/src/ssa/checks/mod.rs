//! This module defines checks on the final SSA.
//!
//! The security checks detect constraint problems leading to possible soundness
//! vulnerabilities, and the compiler informs the developer of these as bugs. The runtime check
//! rejects calls to intrinsics that the calling function's runtime cannot lower.
use crate::ssa::ir::{function::Function, value::ValueId};

mod check_for_missing_brillig_constraints;
mod check_for_underconstrained_values;
mod check_runtime_only_intrinsics;

pub use check_for_missing_brillig_constraints::{
    DEFAULT_MAX_ANCESTOR_DISTANCE, DEFAULT_MAX_ARRAY_OUTPUT_LENGTH,
};

/// Return `true` if a [`ValueId`] identifies a numeric constant in the DFG.
fn is_numeric_constant(func: &Function, value: ValueId) -> bool {
    func.dfg.get_numeric_constant(value).is_some()
}
