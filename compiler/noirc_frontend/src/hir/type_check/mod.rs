mod errors;
mod generics;

pub use self::errors::Source;
pub use errors::{NoMatchingImplFoundError, TypeCheckError};
pub use generics::Generic;
