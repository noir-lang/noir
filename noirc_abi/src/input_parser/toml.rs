use noir_field::FieldElement;
use serde_derive::Deserialize;
use std::{collections::BTreeMap, path::Path};

pub(crate) fn parse<P: AsRef<Path>>(
    path_to_toml: P,
) -> (BTreeMap<String, FieldElement>, Vec<(String, usize)>) {
    let path_to_toml = path_to_toml.as_ref();
    assert!(
        path_to_toml.exists(),
        "cannot find input file at located {}",
        path_to_toml.display()
    );

    // Get input.toml file as a string
    let input_as_string = std::fs::read_to_string(path_to_toml).unwrap();

    // Parse input.toml into a BTreeMap, converting the argument to field elements
    let data: BTreeMap<String, TomlTypes> =
        toml::from_str(&input_as_string).expect("input.toml file is badly formed, could not parse");

    toml_map_to_field(data)
}

/// Flattens the toml map and maps each parameter to a Witness
///
/// Arrays are flattened and each element is given a unique parameter name
/// We need to extract the collection types, since they are not in the FieldMap
///
/// Returns FieldMap and name of all collections
fn toml_map_to_field(
    toml_map: BTreeMap<String, TomlTypes>,
) -> (BTreeMap<String, FieldElement>, Vec<(String, usize)>) {
    let mut field_map = BTreeMap::new();

    let mut collections = Vec::new();

    for (parameter, value) in toml_map {
        match value {
            TomlTypes::String(string) => {
                let old_value = field_map.insert(parameter.clone(), parse_str(&string));
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
            TomlTypes::Integer(integer) => {
                let old_value =
                    field_map.insert(parameter.clone(), parse_str(&integer.to_string()));
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
            TomlTypes::ArrayNum(arr_num) => {
                collections.push((parameter.clone(), arr_num.len()));
                // We need the elements in the array to map to unique names
                // For arrays we postfix the index to the name
                // XXX: In the future, we can use the witness index to map these values
                // This is the only reason why we could have a duplicate name
                for (index, element) in arr_num.into_iter().enumerate() {
                    let unique_param_name = super::mangle_array_element_name(&parameter, index);
                    let old_value = field_map
                        .insert(unique_param_name.clone(), parse_str(&element.to_string()));
                    assert!(
                        old_value.is_none(),
                        "duplicate variable name {}",
                        unique_param_name
                    );
                }
            }
            TomlTypes::ArrayString(arr_str) => {
                collections.push((parameter.clone(), arr_str.len()));

                for (index, element) in arr_str.into_iter().enumerate() {
                    let unique_param_name = super::mangle_array_element_name(&parameter, index);
                    let old_value = field_map
                        .insert(unique_param_name.clone(), parse_str(&element.to_string()));
                    assert!(
                        old_value.is_none(),
                        "duplicate variable name {}",
                        unique_param_name
                    );
                }
            }
        }
    }

    (field_map, collections)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlTypes {
    // This is most likely going to be a hex string
    // But it is possible to support utf8
    String(String),
    // Just a regular integer, that can fit in 128 bits
    Integer(u64),
    // Array of regular integers
    ArrayNum(Vec<u64>),
    // Array of hexadecimal integers
    ArrayString(Vec<String>),
}

fn parse_str(value: &str) -> FieldElement {
    if value.starts_with("0x") {
        FieldElement::from_hex(value).expect(&format!("Could not parse hex value {}", value))
    } else {
        let val: i128 = value
            .parse()
            .expect("Expected witness values to be integers");
        FieldElement::from(val)
    }
}
