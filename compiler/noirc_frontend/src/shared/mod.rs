//! The `shared` module contains simple types which are using in multiple of Noir's IRs.
//!
//! This is done to avoid each IR from needing to have its own definition of elementary types
//! while avoiding one IR being embedded within another.

mod signedness;
mod visibility;

pub use signedness::Signedness;
pub use visibility::Visibility;
