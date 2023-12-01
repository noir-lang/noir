#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod getters;
mod output;
mod pages_generation;
mod tests;

use clap::{Parser, Subcommand};
use getters::*;
use output::*;
use pages_generation::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Documentation error")]
pub enum DocError {
    #[error("Failed to get documentation from file")]
    GetDocError,

    #[error("Failed to extract filename")]
    ExtractFilenameError,

    #[error("Failed to generate module page")]
    GenerateModulePageError,

    #[error("Failed to create new file")]
    FileCreateError,

    #[error("Failed to edit file")]
    FileEditError,

    #[error("Failed to render an object")]
    RenderError,

    #[error("Failed to get information")]
    GetInfoError,

    #[error("Failed to get tokens")]
    GetTokensError,
}

/// Generates documentation from the source code in the specified input file.

/// The `generate_doc` function reads the source code from the given input file, processes it, and
/// generates documentation based on the code's structure and comments. The resulting documentation
/// is typically written to an output file or another destination. This function simplifies the
/// process of generating documentation from source code.
pub fn generate_doc(input_file: &str) -> Result<(), DocError> {
    let doc = get_doc(input_file)?;

    let tokens = Output::to_output(doc);

    let filename = extract_filename(input_file).ok_or(DocError::ExtractFilenameError)?.to_string();

    let out = AllOutput { all_output: tokens?.clone(), filename };

    generate_module_page(out)?;

    Ok(())
}

/// Represents a mapping of information to corresponding documentation.

/// The `Map` struct is used to create a mapping between information and its corresponding documentation.
/// This can be useful for organizing and retrieving documentation related to specific code elements
/// or information from the source code.
#[derive(Debug, PartialEq, Eq)]
pub struct Map {
    map: HashMap<Info, String>,
}

/// Retrieves and maps necessary information to documentation from a Noir code file.

/// The `get_map` function reads a Noir code file specified by the `input_file` path, processes it,
/// and maps the necessary information to its corresponding documentation. It returns a `Map` structure
/// that provides a convenient way to access the documentation associated with various code elements.
pub fn get_map(input_file: &str) -> Result<Map, DocError> {
    let mut map = HashMap::new();

    let doc = get_doc(input_file)?;

    let tokens = Output::to_output(doc)?;

    for token in tokens.iter() {
        map.insert(token.information.clone(), token.doc.clone());
    }

    Ok(Map { map })
}

/// Generates documentation pages based on `Noir` code
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Klee {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates documentation from the source code in the specified input file.
    GenerateDoc {
        #[clap(value_parser)]
        filename: String,
    },
    /// Retrieves and maps necessary information to documentation from a `Noir` code file.
    GetMap {
        #[clap(value_parser)]
        filename: String,
    },
}

fn main() -> Result<(), DocError> {
    let cli = Klee::parse();

    match &cli.command {
        Commands::GenerateDoc { filename } => {
            generate_doc(filename)?;
        }
        Commands::GetMap { filename } => {
            println!("{:#?}", get_map(filename)?);
        }
    }

    Ok(())
}
