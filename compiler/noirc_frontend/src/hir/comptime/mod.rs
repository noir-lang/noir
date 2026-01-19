//! The comptime interpreter is used to execute Noir code which is marked to
//! be executed at compilation time. The main uses for this are:
//!     - Metaprogramming such as deriving trait implementations
//!     - Interpreting comptime blocks, attribute functions, and macros
//!     - Evaluation of globals
//!
//! This interpreter is run on the HIR while the program is being resolved and type-checked, and before it is monomorphized. Code written to be run at comptime may
//! then require additional type hints compared to equivalent non-comptime Noir code.
//!
//! For more information on Noir's comptime execution and metaprogramming in general, see the linked page in
//! the [Noir docs](<https://noir-lang.org/docs/noir/concepts/comptime>)

mod display;
mod errors;
mod hir_to_display_ast;
mod interpreter;
mod tests;
mod value;

pub use display::{tokens_to_string, tokens_to_string_with_indent};
pub use errors::{ComptimeError, InterpreterError};
pub use interpreter::Interpreter;
pub use value::{FormatStringFragment, Value};
