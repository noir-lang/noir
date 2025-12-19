//! This folder contains each optimization pass for the SSA IR.
//!
//! Each pass is generally expected to mutate the SSA IR into a gradually
//! simpler form until the IR only has a single function remaining with 1 block within it.
//! Generally, these passes are also expected to minimize the final amount of instructions.

mod array_set;
mod as_slice_length;
mod basic_conditional;
mod brillig_array_get_and_set;
pub(crate) mod brillig_entry_points;
mod check_u128_mul_overflow;
mod checked_to_unchecked;
mod constant_folding;
mod defunctionalize;
mod die;
mod evaluate_static_assert_and_assert_constant;
mod expand_signed_checks;
mod expand_signed_math;
pub(crate) mod flatten_cfg;
mod hint;
mod inline_simple_functions;
mod inlining;
mod loop_invariant;
mod make_constrain_not_equal;
mod mem2reg;
mod normalize_value_ids;
mod preprocess_fns;
pub(crate) mod pure;
mod rc;
mod remove_bit_shifts;
mod remove_enable_side_effects;
mod remove_if_else;
mod remove_truncate_after_range_check;
mod remove_unreachable_functions;
mod remove_unreachable_instructions;
mod remove_unused_instructions;
mod simple_optimization;
mod simplify_cfg;
mod unrolling;

pub use constant_folding::DEFAULT_MAX_ITER as CONSTANT_FOLDING_MAX_ITER;
pub use inlining::MAX_INSTRUCTIONS as INLINING_MAX_INSTRUCTIONS;
pub(crate) use unrolling::Loops;

#[cfg(test)]
use crate::ssa::{
    interpreter::{errors::InterpreterError, value::Value},
    ssa_gen::Ssa,
};

/// Asserts that the given SSA, after normalizing its IDs and printing it,
/// is equal to the expected string. Normalization is done so the IDs don't
/// shift depending on whether temporary intermediate values were created.
#[cfg(test)]
pub(crate) fn assert_normalized_ssa_equals(mut ssa: Ssa, expected: &str) {
    use crate::{ssa::Ssa, trim_comments_from_lines, trim_leading_whitespace_from_lines};

    // Clean up the expected SSA a bit
    let expected = trim_leading_whitespace_from_lines(expected);
    let expected = trim_comments_from_lines(&expected);

    // First check if `expected` is valid SSA by parsing it, otherwise
    // the comparison will always fail but it won't be clear that it's because
    // expected is not valid.
    let mut expected_ssa = match Ssa::from_str(&expected) {
        Ok(ssa) => ssa,
        Err(err) => {
            panic!("`expected` argument of `assert_ssa_equals` is not valid SSA:\n{err:?}")
        }
    };

    // We won't exactly compare `expected` against `ssa`:
    // we parse it, normalize it and turn it back into a string.
    // This allows us to use any names and not just `b0`, `b1`, `v0`, `v1`, etc.
    // which is what the SSA printer produces.
    expected_ssa.normalize_ids();

    ssa.normalize_ids();

    let ssa = ssa.print_without_locations().to_string();
    let ssa = ssa.trim_end();

    let expected_ssa = expected_ssa.print_without_locations().to_string();
    let expected_ssa = expected_ssa.trim_end();

    if ssa == expected_ssa {
        return;
    }

    if expected != expected_ssa {
        println!("Expected (before ID normalization):\n~~~\n{expected}\n~~~\n");
    }

    println!("Expected (after ID normalization):\n~~~\n{expected_ssa}\n~~~\n");
    println!("Got:\n~~~\n{ssa}\n~~~");
    similar_asserts::assert_eq!(expected_ssa, ssa);
}

/// Compare the textural representation of the SSA after normalizing its IDs to a snapshot.
///
/// # Example:
///
/// ```ignore
/// let ssa = todo!();
/// assert_ssa_snapshot!(ssa, @r"
///   acir(inline) fn main f0 {
///       b0(v0: Field):
///         return v0
///     }
/// ");
/// ```
/// Or without taking ownership:
/// ```ignore
/// let mut ssa = todo!();
/// assert_ssa_snapshot!(&mut ssa, @r"
///   acir(inline) fn main f0 {
///       b0(v0: Field):
///         return v0
///     }
/// ");
/// ```
#[macro_export]
macro_rules! assert_ssa_snapshot {
    ($ssa:expr, $($arg:tt)*) => {
        #[allow(unused_mut)]
        let mut mut_ssa = $ssa;
        mut_ssa.normalize_ids();
        let ssa_string = mut_ssa.print_without_locations().to_string();
        insta::assert_snapshot!(ssa_string, $($arg)*)
    };
}

/// Assert that running a certain pass on the SSA does nothing.
#[cfg(test)]
pub(crate) fn assert_ssa_does_not_change(src: &str, pass: impl FnOnce(Ssa) -> Ssa) {
    let ssa = Ssa::from_str(src).unwrap();
    let ssa = pass(ssa);
    assert_normalized_ssa_equals(ssa, src);
}

/// Assert that running a certain pass on the SSA does not change the execution result.
#[cfg(test)]
fn assert_pass_does_not_affect_execution(
    ssa: Ssa,
    inputs: Vec<Value>,
    ssa_pass: impl FnOnce(Ssa) -> Ssa,
) -> (Ssa, Result<Vec<Value>, InterpreterError>) {
    let before = ssa.interpret(inputs.clone());

    let new_ssa = ssa_pass(ssa);

    let after = new_ssa.interpret(inputs);
    assert_eq!(before, after, "SSA pass has resulted in a different execution result");
    (new_ssa, after)
}
