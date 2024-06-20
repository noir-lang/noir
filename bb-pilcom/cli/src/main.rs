use std::{io, path::Path};

use bb_pil_backend::vm_builder::analyzed_to_cpp;
use clap::Parser;
use powdr_ast::analyzed::{Analyzed, FunctionValueDefinition, Symbol};
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
}

fn extract_col_name(cols: Vec<&(Symbol, Option<FunctionValueDefinition>)>) -> Vec<String> {
    // Note that function val def should be none
    cols.iter()
        .map(|(sym, _def)| sym.absolute_name.replace(".", "_"))
        .collect()
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();

    let file_name = args.file;
    let name = args.name;

    let analyzed: Analyzed<Bn254Field> = analyze_file(Path::new(&file_name));

    let fixed = analyzed.constant_polys_in_source_order();
    let witness = analyzed.committed_polys_in_source_order();
    let public = analyzed.public_polys_in_source_order();

    analyzed_to_cpp(
        &analyzed,
        &extract_col_name(fixed),
        &extract_col_name(witness),
        &extract_col_name(public),
        name,
    );
    Ok(())
}
