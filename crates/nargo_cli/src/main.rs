#![forbid(unsafe_code)]

mod panic;

fn main() {
    panic::set_hook();

    if let Err(report) = nargo_cli::cli::start_cli() {
        println!("{report}");
    }
}
