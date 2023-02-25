#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

// For now we use a wrapper around generational-arena
pub use generational_arena::{Arena, Index};
