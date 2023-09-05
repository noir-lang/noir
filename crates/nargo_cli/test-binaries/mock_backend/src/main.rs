#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::io::Write;

fn main() {
    std::io::stdout().write_all(&0u64.to_be_bytes()).unwrap();
}
