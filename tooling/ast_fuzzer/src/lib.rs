#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

mod abi;
pub mod compare;
mod input;
mod program;

pub use abi::program_abi;
pub use compare::input_values_to_ssa;
pub use input::arb_inputs;
use program::freq::Freqs;
pub use program::{DisplayAstAsNoir, DisplayAstAsNoirComptime, arb_program, arb_program_comptime};
pub use program::{expr, rewrite, visitor};

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
    /// Only use comptime friendly expressions.
    pub comptime_friendly: bool,
}

impl Default for Config {
    fn default() -> Self {
        let expr_freqs = Freqs::new(&[
            ("unary", 10),
            ("binary", 20),
            ("if", 15),
            ("block", 30),
            ("vars", 25),
            ("literal", 5),
            ("call", 15),
        ]);
        let stmt_freqs_acir = Freqs::new(&[
            ("drop", 0), // The `ownership` module says it will insert `Drop` and `Clone`.
            ("assign", 30),
            ("if", 10),
            ("for", 18),
            ("let", 25),
            ("call", 5),
        ]);
        let stmt_freqs_brillig = Freqs::new(&[
            ("drop", 0),
            ("break", 20),
            ("continue", 20),
            ("assign", 30),
            ("if", 10),
            ("for", 15),
            ("loop", 15),
            ("while", 15),
            ("let", 20),
            ("call", 5),
            ("print", 15),
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
            expr_freqs,
            stmt_freqs_acir,
            stmt_freqs_brillig,
            force_brillig: false,
            avoid_overflow: false,
            avoid_err_by_zero: false,
            avoid_large_int_literals: false,
            avoid_negative_int_literals: false,
            avoid_loop_control: false,
            avoid_lambdas: false,
            comptime_friendly: false,
        }
    }
}
