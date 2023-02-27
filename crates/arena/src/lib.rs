#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

// For now we use a wrapper around generational-arena
pub use generational_arena::{Arena, Index};
