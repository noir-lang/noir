mod cli;

fn main() {
    if let Err(report) = cli::start_cli() {
        eprintln!("{report:?}");
        std::process::exit(1);
    }
}
