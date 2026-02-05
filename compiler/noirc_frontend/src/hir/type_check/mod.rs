mod errors;
pub mod generics;

pub use self::errors::Source;
pub use errors::{
    ExpectingOtherError, MAX_MISSING_CASES, NoMatchingImplFoundError, TypeCheckError,
};
