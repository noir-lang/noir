mod abi;
pub mod compare;
mod input;
mod program;

pub use abi::program_abi;
pub use input::arb_inputs;
use program::freq::Freqs;
pub use program::{DisplayAstAsNoir, arb_program};

/// AST generation configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of global definitions.
    pub max_globals: usize,
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
    pub max_range_size: usize,
    /// Frequency of expressions that produce a value.
    pub expr_freqs: Freqs,
    /// Frequency of statements that don't produce a value.
    pub stmt_freqs: Freqs,
}

impl Default for Config {
    fn default() -> Self {
        let expr_freqs = Freqs::new(&[
            ("unary", 5),
            ("binary", 20),
            ("if", 15),
            ("block", 30),
            ("vars", 20),
            ("literal", 5),
        ]);
        let stmt_freqs =
            Freqs::new(&[("drop", 5), ("assign", 30), ("if", 10), ("for", 20), ("let", 20)]);
        Self {
            max_globals: 3,
            max_functions: 5,
            max_function_args: 3,
            max_function_size: 25,
            max_block_size: 5,
            max_depth: 2,
            max_tuple_size: 5,
            max_array_size: 4,
            max_range_size: 10,
            expr_freqs,
            stmt_freqs,
        }
    }
}
