use std::{io, path::Path};

use bb_pil_backend::vm_builder::analyzed_to_cpp;
use clap::Parser;
use powdr_ast::analyzed::Analyzed;
use powdr_number::Bn254Field;
use powdr_pil_analyzer::analyze_file;

#[derive(Parser)]
#[command(name = "bb-pil-cli", author, version, about, long_about = None)]
struct Cli {
    /// Input file
    file: String,

    /// Output directory for the PIL file, json file and fixed and witness column data.
    #[arg(short, long)]
    #[arg(default_value_t = String::from("."))]
    output_directory: String,

    /// BBerg: Name of the output file for bberg
    #[arg(long)]
    name: Option<String>,

    /// Delete the output directory if it already exists
    #[arg(short, long)]
    #[arg(default_value_t = false)]
    yes: bool,
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();

    let file_name = args.file;
    let name = args.name.unwrap();
    let analyzed: Analyzed<Bn254Field> = analyze_file(Path::new(&file_name));

    analyzed_to_cpp(&analyzed, &name, args.yes);

    Ok(())
}
