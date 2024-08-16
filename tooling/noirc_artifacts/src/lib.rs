#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.

pub mod contract;
pub mod debug;
mod debug_vars;
pub mod program;
