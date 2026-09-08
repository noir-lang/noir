#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

mod cli;

fn main() {
    if let Err(report) = cli::start_cli() {
        eprintln!("{report:#}");
        std::process::exit(1);
    }
}
