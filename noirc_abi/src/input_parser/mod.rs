mod toml;

use std::{collections::BTreeMap, path::Path};

use noir_field::FieldElement;

/// Parses the initial Witness Values that are needed to seed the
/// Partial Witness generator
pub trait InitialWitnessParser {
    fn parse_initial_witness<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> (BTreeMap<String, FieldElement>, Vec<(String, usize)>);
}

/// The different formats that are supported when parsing
/// the initial witness values
pub enum Format {
    Toml,
}

impl Format {
    /// The name of the file that this Formatter can parse
    /// Eg. The Toml parser will look for an `input.toml`
    /// file in the directory.
    pub fn file_name(&self) -> &'static str {
        match self {
            Format::Toml => "input.toml",
        }
    }
}

impl Format {
    pub fn parse<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> (BTreeMap<String, FieldElement>, Vec<(String, usize)>) {
        match self {
            Format::Toml => {
                let mut dir_path = path.as_ref().to_path_buf();
                dir_path.push(self.file_name());
                toml::parse(dir_path)
            }
        }
    }
}

/// We allow users to input an array in the ABI
/// Each element must be mapped to a unique identifier
/// XXX: At the moment, the evaluator uses String, in particular the variable name
/// This function ensures that each element in the array is assigned a unique identifier
///
/// XXX: remove from noirc_evaluator in next refactor
pub fn mangle_array_element_name(array_name: &str, element_index: usize) -> String {
    use blake2::{Blake2s, Digest};

    let mut hasher = Blake2s::new();
    hasher.update(array_name);

    // use u128 so we do not get different hashes depending on the computer
    // architecture
    let index_u128 = element_index as u128;
    hasher.update(index_u128.to_be_bytes());

    let res = hasher.finalize();

    // If a variable is named array_0_1f4a
    // Then it will be certain, that the user
    // is trying to be malicious
    // For regular users, they will never encounter a name conflict
    // We could probably use MD5 here, as we do not need a crypto hash
    let checksum = &res[0..4];

    format!("{}__{}__{:x?}", array_name, element_index, checksum)
}
