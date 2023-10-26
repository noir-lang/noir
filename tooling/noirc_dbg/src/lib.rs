#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod app;
pub(crate) mod compile;
pub mod dap_server;
pub mod error;
pub(crate) mod vm;
