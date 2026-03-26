//! This module defines security SSA passes detecting constraint problems leading to possible
//! soundness vulnerabilities.
//!
//! The compiler informs the developer of these as bugs.
use crate::ssa::ir::{function::Function, value::ValueId};

mod check_for_missing_brillig_constraints;
mod check_for_underconstrained_values;

/// Return `true` if a [ValueId] identifies a numeric constant in the DFG.
fn is_numeric_constant(func: &Function, value: ValueId) -> bool {
    func.dfg.get_numeric_constant(value).is_some()
}
