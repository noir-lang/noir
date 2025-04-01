mod abi;
pub mod compare;
mod input;
mod program;

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
    /// Maximum nesting depth for complex expressions.
    pub max_depth: usize,
    /// Maximum number of fields for tuples.
    pub max_tuple_size: usize,
    /// Maximum size for arrays.
    pub max_array_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_globals: 3,
            max_functions: 5,
            max_function_args: 3,
            max_depth: 2,
            max_tuple_size: 4,
            max_array_size: 8,
        }
    }
}
