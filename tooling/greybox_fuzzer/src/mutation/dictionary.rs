use std::{collections::HashSet, iter::zip};

use acvm::{AcirField, FieldElement};
use noirc_abi::{input_parser::InputValue, Abi, AbiType, InputMap};
/// This file contains the collection of objects for providing program-specific values to the mutator

/// A dictionary for integer values. Separated by width
#[derive(Default)]
pub struct IntDictionary {
    width_dictionaries: [Vec<FieldElement>; 4],
}

pub type FieldDictionary = Vec<FieldElement>;

impl IntDictionary {
    pub fn new(original_dictionary: &[FieldElement]) -> Self {
        Self { width_dictionaries: Self::filter_dictionary_by_width(original_dictionary) }
    }
    pub fn get_dictionary_by_width(&self, width: u32) -> &Vec<FieldElement> {
        match width {
            8 => &self.width_dictionaries[0],
            16 => &self.width_dictionaries[1],
            32 => &self.width_dictionaries[2],
            64 => &self.width_dictionaries[3],
            _ => panic!("Only widths 8, 16, 32, 64 are supported"),
        }
    }
    /// Filter values in the original dictionary collected from the program into 4 categories, separated by width of integers into which those elements can fit
    fn filter_dictionary_by_width(original_dictionary: &[FieldElement]) -> [Vec<FieldElement>; 4] {
        let mut width8_dict = Vec::new();
        let mut width16_dict = Vec::new();
        let mut width32_dict = Vec::new();
        let mut width64_dict = Vec::new();
        const MAX_U8: i128 = u8::MAX as i128;
        const MAX_U16: i128 = u16::MAX as i128;
        const MAX_U32: i128 = u32::MAX as i128;
        const MAX_U64: i128 = u64::MAX as i128;
        for element in original_dictionary.iter().copied() {
            let el_i128 = element.to_i128();
            if el_i128 <= 0 {
                continue;
            }
            if el_i128 < MAX_U64 {
                width64_dict.push(element);
            }
            if el_i128 < MAX_U32 {
                width32_dict.push(element);
            }
            if el_i128 < MAX_U16 {
                width16_dict.push(element);
            }
            if el_i128 < MAX_U8 {
                width8_dict.push(element);
            }
        }
        [width8_dict, width16_dict, width32_dict, width64_dict]
    }
}

/// An object with values from the program used for mutating inputs
/// Contains an int dictionary, where elements are grouped into appropriate widths
pub struct FullDictionary {
    field_dictionary: Vec<FieldElement>,
    int_dictionary: IntDictionary,
}

impl FullDictionary {
    /// Parse input value and collect value(s) for the dictionary from it
    fn collect_dictionary_from_input_value(
        abi_type: &AbiType,
        input: &InputValue,
        full_dictionary: &mut HashSet<FieldElement>,
    ) {
        match abi_type {
            // Boolean only has 2 values, there is no point in getting the value
            AbiType::Boolean => (),
            AbiType::Field | AbiType::Integer { .. } => {
                let initial_field_value = match input {
                    InputValue::Field(inner_field) => inner_field,
                    _ => panic!("Shouldn't be used with other input value types"),
                };
                full_dictionary.insert(*initial_field_value);
            }
            AbiType::String { length: _ } => {
                let initial_string = match input {
                    InputValue::String(inner_string) => inner_string,
                    _ => panic!("Shouldn't be used with other input value types"),
                };
                for character in initial_string.as_bytes().iter() {
                    full_dictionary.insert(FieldElement::from(*character as i128));
                }
            }
            AbiType::Array { length: _, typ } => {
                let input_vector = match input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                for element in input_vector.iter() {
                    Self::collect_dictionary_from_input_value(typ, element, full_dictionary);
                }
            }

            AbiType::Struct { fields, .. } => {
                let input_struct = match input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                for (name, typ) in fields.iter() {
                    Self::collect_dictionary_from_input_value(
                        typ,
                        &input_struct[name],
                        full_dictionary,
                    );
                }
            }

            AbiType::Tuple { fields } => {
                let input_vector = match input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                for (typ, previous_tuple_input) in zip(fields, input_vector) {
                    Self::collect_dictionary_from_input_value(
                        typ,
                        previous_tuple_input,
                        full_dictionary,
                    );
                }
            }
        }
    }

    /// Update the dictionary of field elements from a given testcase
    fn collect_dictionary_from_input(
        abi: &Abi,
        input: &InputMap,
        full_dictionary: &mut HashSet<FieldElement>,
    ) {
        for param in abi.parameters.iter() {
            Self::collect_dictionary_from_input_value(
                &param.typ,
                &input[&param.name],
                full_dictionary,
            );
        }
    }

    /// Create a full dictionary from a set of field elements
    /// Creates a dictionary including the original elements and the same set separated by integer widths for quicker access during mutations
    pub fn new(original_dictionary: &HashSet<FieldElement>) -> Self {
        let dictionary_vector: Vec<_> = original_dictionary.iter().copied().collect();
        let int_dict = IntDictionary::new(&dictionary_vector);
        Self { field_dictionary: dictionary_vector, int_dictionary: int_dict }
    }

    /// Update the dictionary with values from a given testcase
    pub fn update(&mut self, abi: &Abi, testcase: &InputMap) {
        let mut testcase_full_dictionary: HashSet<_> =
            self.field_dictionary.iter().copied().collect();
        Self::collect_dictionary_from_input(abi, testcase, &mut testcase_full_dictionary);
        self.field_dictionary = testcase_full_dictionary.iter().copied().collect();
        self.int_dictionary = IntDictionary::new(&self.field_dictionary);
    }

    /// Get a reference to the int dictionary
    pub fn get_int_dictionary(&self) -> &IntDictionary {
        &self.int_dictionary
    }

    /// Get a reference to the field dictionary
    pub fn get_field_dictionary(&self) -> &FieldDictionary {
        &self.field_dictionary
    }
}
