mod getters;
mod output;
mod pages_generation;
mod tests;

use std::collections::HashMap;
use getters::*;
use output::*;
use pages_generation::*;

/// the main function of the program
/// generates all documentation files
/// the input file is a file with a Noir code
pub fn generate_doc(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let doc = get_doc(input_file).unwrap();

    let tokens = Output::to_output(doc);

    let filename = extract_filename(input_file).unwrap().to_string();

    let out = AllOutput{ all_output: tokens.clone(), filename };

    generate_module_page(out)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct Map {
    map: HashMap<Info, String>,
}

/// returns all necessary information for generating documentation
/// the input file is a file with a Noir code
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
