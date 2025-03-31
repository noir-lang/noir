mod abi;
mod input;
mod program;

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
}

impl Default for Config {
    fn default() -> Self {
        Self { max_globals: 3, max_functions: 3, max_function_args: 3 }
    }
}
