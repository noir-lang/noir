mod cli;
mod fs;

fn main() {
    if let Err(e) = cli::start_cli() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
