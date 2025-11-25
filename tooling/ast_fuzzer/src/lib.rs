#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use rand as _;

mod abi;
pub mod compare;
mod input;
mod program;

pub use abi::program_abi;
pub use compare::{input_value_to_ssa, input_values_to_ssa};
pub use input::arb_inputs;
use program::freq::Freqs;
pub use program::{
    DisplayAstAsNoir, DisplayAstAsNoirComptime, arb_program, arb_program_comptime,
    program_wrap_expression,
};
pub use program::{expr, rewrite, scope, types};

/// AST generation configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of global definitions.
    pub max_globals: usize,
    /// Minimum number of functions (other than main) to generate.
    pub min_functions: usize,
    /// Maximum number of functions (other than main) to generate.
    pub max_functions: usize,
    /// Maximum number of arguments a function can have.
    pub max_function_args: usize,
    /// Maximum number of statements to aim for in a function body.
    pub max_function_size: usize,
    /// Maximum number of statements in a block.
    pub max_block_size: usize,
    /// Maximum nesting depth for complex expressions.
    pub max_depth: usize,
    /// Maximum number of fields for tuples.
    pub max_tuple_size: usize,
    /// Maximum size for arrays.
    pub max_array_size: usize,
    /// Maximum size of for loop ranges, which affects unrolling in ACIR.
    pub max_loop_size: usize,
    /// Whether to choose the backstop for `loop` and `while` randomly.
    pub vary_loop_size: bool,
    /// Maximum number of recursive calls to make at runtime.
    pub max_recursive_calls: usize,
    /// Maximum number of match cases.
    pub max_match_cases: usize,
    /// Frequency of expressions, which produce a value.
    pub expr_freqs: Freqs,
    /// Frequency of statements in ACIR functions.
    pub stmt_freqs_acir: Freqs,
    /// Frequency of statements in Brillig functions.
    pub stmt_freqs_brillig: Freqs,
    /// Whether to force all functions to be unconstrained.
    pub force_brillig: bool,
    /// Try to avoid overflowing operations. Useful when testing the minimal pipeline,
    /// to avoid trivial failures due to multiplying or adding constants.
    pub avoid_overflow: bool,
    /// Try to avoid "Index out of bounds" by using modulo to limit indexing to the
    /// range that an array or slice is expected to contain.
    ///
    /// This is easy to trigger (a random `u32` will most certainly be out of the range
    /// of the arrays we generate), so by default it is "on". When it's "off", a random
    /// decision is taken for each index operation whether to apply the modulo or not.
    pub avoid_index_out_of_bounds: bool,
    /// Try to avoid operations that can result in error when zero is on the RHS.
    pub avoid_err_by_zero: bool,
    /// Avoid using negative integer literals where the frontend expects unsigned types.
    pub avoid_negative_int_literals: bool,
    /// Avoid using large integer literals where the frontend expects 32 bits.
    pub avoid_large_int_literals: bool,
    /// Avoid using loop control (break/continue).
    pub avoid_loop_control: bool,
    /// Avoid using function pointers in parameters.
    pub avoid_lambdas: bool,
    /// Avoid print statements.
    pub avoid_print: bool,
    /// Avoid using constrain statements.
    pub avoid_constrain: bool,
    /// Avoid match statements and expressions.
    pub avoid_match: bool,
    /// Avoid using the slice type.
    pub avoid_slices: bool,
    /// Only use comptime friendly expressions.
    pub comptime_friendly: bool,
}

impl Default for Config {
    fn default() -> Self {
        let expr_freqs = Freqs::new(&[
            ("unary", 10),
            ("binary", 20),
            ("if", 15),
            ("match", 30),
            ("block", 30),
            ("vars", 25),
            ("literal", 5),
            ("call", 15),
        ]);
        let stmt_freqs_acir = Freqs::new(&[
            ("assign", 30),
            ("if", 10),
            ("match", 10),
            ("for", 37),
            ("let", 25),
            ("call", 5),
            ("constrain", 4),
        ]);
        let stmt_freqs_brillig = Freqs::new(&[
            ("break", 45),
            ("continue", 25),
            ("assign", 30),
            ("if", 10),
            ("match", 15),
            ("for", 40),
            ("loop", 40),
            ("while", 40),
            ("let", 20),
            ("call", 5),
            ("print", 15),
            ("constrain", 15),
        ]);
        Self {
            max_globals: 3,
            min_functions: 0,
            max_functions: 5,
            max_function_args: 3,
            max_function_size: 25,
            max_block_size: 5,
            max_depth: 2,
            max_tuple_size: 5,
            max_array_size: 4,
            max_loop_size: 10,
            vary_loop_size: true,
            max_recursive_calls: 25,
            max_match_cases: 3,
            expr_freqs,
            stmt_freqs_acir,
            stmt_freqs_brillig,
            force_brillig: false,
            avoid_overflow: false,
            avoid_index_out_of_bounds: true,
            avoid_err_by_zero: false,
            avoid_large_int_literals: false,
            avoid_negative_int_literals: false,
            avoid_loop_control: false,
            avoid_lambdas: false,
            avoid_print: false,
            avoid_constrain: false,
            avoid_match: false,
            avoid_slices: false,
            comptime_friendly: false,
        }
    }
}
