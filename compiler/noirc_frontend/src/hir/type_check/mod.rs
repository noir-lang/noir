mod errors;
pub mod generics;

pub use self::errors::Source;
pub use errors::{MAX_MISSING_CASES, NoMatchingImplFoundError, TypeCheckError};
