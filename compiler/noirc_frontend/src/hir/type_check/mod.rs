//! This file contains type_check_func, the entry point to the type checking pass (for each function).
//!
//! The pass structure of type checking is relatively straightforward. It is a single pass through
//! the HIR of each function and outputs the inferred type of each HIR node into the NodeInterner,
//! keyed by the ID of the node.
//!
//! Although this algorithm features inference via TypeVariables, there is no generalization step
//! as all functions are required to give their full signatures. Closures are inferred but are
//! never generalized and thus cannot be used polymorphically.
mod errors;
pub use self::errors::Source;
pub use errors::{NoMatchingImplFoundError, TypeCheckError};
