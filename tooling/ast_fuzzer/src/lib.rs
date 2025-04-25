mod abi;
pub mod compare;
mod input;
mod program;

pub use abi::program_abi;
pub use input::arb_inputs;
use program::freq::Freqs;
pub use program::visitor::{visit_expr, visit_expr_mut};
pub use program::{DisplayAstAsNoir, DisplayAstAsNoirComptime, arb_program, arb_program_comptime};

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
            ("drop", 3),
            ("assign", 30),
            ("if", 10),
            ("for", 18),
            ("let", 25),
            ("call", 5),
        ]);
        let stmt_freqs_brillig = Freqs::new(&[
            ("drop", 5),
            ("break", 20),
            ("continue", 20),
            ("assign", 30),
            ("if", 10),
            ("for", 15),
            ("loop", 15),
            ("while", 15),
            ("let", 20),
            ("call", 5),
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
        }
    }
}
