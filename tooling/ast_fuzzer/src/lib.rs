mod abi;
pub mod compare;
mod input;
mod program;

use std::ops::Index;

pub use abi::program_abi;
pub use input::arb_inputs;
pub use program::arb_program;

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
    /// Maximum nesting depth for complex expressions.
    pub max_depth: usize,
    /// Maximum number of fields for tuples.
    pub max_tuple_size: usize,
    /// Maximum size for arrays.
    pub max_array_size: usize,
    /// Frequency of expressions that produce a value.
    pub expr_freqs: Freqs,
    /// Frequency of statements that don't produce a value.
    pub stmt_freqs: Freqs,
}

impl Default for Config {
    fn default() -> Self {
        let expr_freqs = Freqs::new(&[
            ("unary", 5),
            ("binary", 15),
            ("if_then", 15),
            ("block", 20),
            ("vars", 50),
            ("literal", 5),
        ]);
        let stmt_freqs = Freqs::new(&[("drop", 5), ("assign", 20), ("if_then", 20), ("let", 30)]);
        Self {
            max_globals: 3,
            max_functions: 5,
            max_function_args: 3,
            max_function_size: 50,
            max_depth: 2,
            max_tuple_size: 5,
            max_array_size: 4,
            expr_freqs,
            stmt_freqs,
        }
    }
}

/// Frequency distribution of generators.
#[derive(Debug, Clone)]
pub struct Freqs {
    items: im::HashMap<&'static str, usize>,
    total: usize,
}

impl Freqs {
    pub fn new(items: &[(&'static str, usize)]) -> Self {
        let total = items.iter().map(|i| i.1).sum();
        Self { items: items.iter().cloned().collect(), total }
    }
    pub fn total(&self) -> usize {
        self.total
    }
}

impl Index<&str> for Freqs {
    type Output = usize;

    fn index(&self, index: &str) -> &Self::Output {
        self.items.get(index).unwrap_or_else(|| panic!("unknown freq: {index}"))
    }
}
