mod getters;
mod output;
mod pages_generation;
mod tests;

use getters::*;
use output::*;
use pages_generation::*;
use std::collections::HashMap;

/// Generates documentation from the source code in the specified input file.

/// The `generate_doc` function reads the source code from the given input file, processes it, and
/// generates documentation based on the code's structure and comments. The resulting documentation
/// is typically written to an output file or another destination. This function simplifies the
/// process of generating documentation from source code.
pub fn generate_doc(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let doc = get_doc(input_file).unwrap();

    let tokens = Output::to_output(doc);

    let filename = extract_filename(input_file).unwrap().to_string();

    let out = AllOutput { all_output: tokens.clone(), filename };

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
pub fn get_map(input_file: &str) -> Map {
    let mut map = HashMap::new();

    let doc = get_doc(input_file).unwrap();

    let tokens = Output::to_output(doc);

    for token in tokens.iter() {
        map.insert(token.information.clone(), token.doc.clone());
    }

    Map { map }
}

fn main() {
    generate_doc("input_files/prog.nr").unwrap();

    dbg!(get_map("input_files/struct_example.nr"));
}
