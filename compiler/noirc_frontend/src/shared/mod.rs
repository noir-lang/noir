//! The `shared` module contains simple types which are using in multiple of Noir's IRs.
//!
//! This is done to avoid each IR from needing to have its own definition of elementary types
//! while avoiding one IR being embedded within another.

mod builtin;
mod foreign_calls;
mod signedness;
mod visibility;

pub use builtin::Builtin;
pub use foreign_calls::ForeignCall;
pub use signedness::Signedness;
pub use visibility::Visibility;
