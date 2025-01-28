use acvm::{AcirField, FieldElement};

use array::{mutate_vector_structure, splice_array_structure};
use configurations::{
    TestCaseSpliceType, BASIC_TESTCASE_SPLICE_CONFIGURATION,
    BASIC_TOP_LEVEL_MUTATION_CONFIGURATION, BASIC_UNBALANCED_ARRAY_SPLICE_MUTATION_CONFIGURATION,
};
use dictionary::FullDictionary;
use field::mutate_field_input_value;
use int::mutate_int_input_value;
use noirc_abi::{input_parser::InputValue, Abi, AbiType, InputMap};
use rand::Rng;
use rand_xorshift::XorShiftRng;
use std::{
    collections::{BTreeMap, HashSet},
    iter::zip,
};
use string::{mutate_string_input_value, splice_string_input_value};

mod array;
mod configurations;
mod dictionary;
mod field;
mod int;
mod string;
use num_traits::Zero;
pub struct InputMutator {
    abi: Abi,
    weight_tree: NodeWeight,
    full_dictionary: FullDictionary,
}

const MUTATION_LOG_MIN: u32 = 0;
const MUTATION_LOG_MAX: u32 = 5;
/// NodeWeight determines the probability of mutating a particular object
/// It represent the relative weight of this InputValue and its children
#[derive(Clone, Debug)]
pub struct NodeWeight {
    start: u32,
    end: u32,
    subnodes: Option<Vec<NodeWeight>>,
}
impl NodeWeight {
    pub fn get_weight(&self) -> u32 {
        self.end - self.start
    }
    pub fn calculate_offsets(&mut self, start_offset: u32) {
        assert!(self.start == 0, "Only calculate offsets after initial computation");
        self.start += start_offset;
        self.end += start_offset;
        let mut current_update = start_offset;
        if self.subnodes.is_some() {
            let subnode_weights = self.subnodes.as_mut().unwrap();
            for subnode_weight in subnode_weights.iter_mut() {
                let weight_update = subnode_weight.get_weight();
                subnode_weight.calculate_offsets(current_update);
                current_update += weight_update;
            }
        }
    }
}

enum UnbalancedSplice {
    FirstTestCase,
    SecondTestCase,
}

struct UnbalancedSpliceConfiguration {
    first_testcase_weight: usize,
    #[allow(unused)]
    second_testcase_weight: usize,
    total_weight: usize,
}

impl UnbalancedSpliceConfiguration {
    #[allow(unused)]
    pub fn new(first_testcase_weight: usize, second_testcase_weight: usize) -> Self {
        let total_weight = first_testcase_weight + second_testcase_weight;
        Self { first_testcase_weight, second_testcase_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> UnbalancedSplice {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.first_testcase_weight {
            return UnbalancedSplice::FirstTestCase;
        }
        return UnbalancedSplice::SecondTestCase;
    }
}

const BASIC_UNBALANCED_SLICE_CONFIGURATION: UnbalancedSpliceConfiguration =
    UnbalancedSpliceConfiguration {
        first_testcase_weight: 8,
        second_testcase_weight: 2,
        total_weight: 8 + 2,
    };
const BOOL_WEIGHT: u32 = 1u32;
const FIELD_WEIGHT: u32 = 8u32;
const STRING_WEIGHT_MULTIPLIER: u32 = 4;
impl InputMutator {
    /// Create a new input mutator given the ABI and a hashset of field elements used in the circuit (for the dictionary)
    pub fn new(abi: &Abi, original_dictionary: &HashSet<FieldElement>) -> Self {
        let mut weight_tree = Self::count_all_input_weights(abi);
        weight_tree.calculate_offsets(0);
        Self {
            abi: abi.clone(),
            weight_tree,
            full_dictionary: FullDictionary::new(original_dictionary),
        }
    }

    /// Fill the dictionary with values from an interesting input
    pub fn update_dictionary(&mut self, testcase: &InputMap) {
        self.full_dictionary.update(&self.abi, testcase);
    }

    /// Count weights of each element recursively (complex structures return a vector of weights of their most basic elements)
    fn count_single_input_weight(abi_type: &AbiType) -> NodeWeight {
        match abi_type {
            AbiType::Boolean => NodeWeight { start: 0u32, end: BOOL_WEIGHT, subnodes: None },
            AbiType::Field | AbiType::Integer { .. } => {
                NodeWeight { start: 0u32, end: FIELD_WEIGHT, subnodes: None }
            }
            AbiType::String { length } => {
                NodeWeight { start: 0u32, end: *length * STRING_WEIGHT_MULTIPLIER, subnodes: None }
            }
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let subnode_weight = Self::count_single_input_weight(typ);
                let node_weight = subnode_weight.get_weight() * length as u32;

                NodeWeight {
                    start: 0,
                    end: node_weight,
                    subnodes: Some(vec![subnode_weight; length]),
                }
            }

            AbiType::Struct { fields, .. } => {
                let mut weights = Vec::new();

                let mut total_node_weight = 0u32;
                for single in fields.iter().map(|(_, typ)| Self::count_single_input_weight(typ)) {
                    total_node_weight += &single.get_weight();
                    weights.push(single);
                }

                NodeWeight { start: 0, end: total_node_weight, subnodes: Some(weights) }
            }

            AbiType::Tuple { fields } => {
                let mut weights = Vec::new();

                let mut total_node_weight = 0u32;
                for single in fields.iter().map(Self::count_single_input_weight) {
                    total_node_weight += &single.get_weight();
                    weights.push(single);
                }

                NodeWeight { start: 0, end: total_node_weight, subnodes: Some(weights) }
            }
        }
    }

    /// Recurse through all the inputs in the ABI and collect weights of every input so we don't get affected by the depth
    fn count_all_input_weights(abi: &Abi) -> NodeWeight {
        assert!(!abi.parameters.is_empty());
        let mut weights = Vec::new();

        let mut total_node_weight = 0u32;
        for single in
            abi.parameters.iter().map(|parameter| Self::count_single_input_weight(&parameter.typ))
        {
            total_node_weight += &single.get_weight();
            weights.push(single);
        }

        NodeWeight { start: 0, end: total_node_weight, subnodes: Some(weights) }
    }

    /// Perform a single update on an input value
    /// In case the value is a basic type (boolean, field, integer), perform value mutations according to the type
    /// In case it is a string, either perform value mutations on the characters or perform structural mutation on the string
    /// In case it is a complex type (array, struct, tuple), select a child and recurse
    /// If we hit any arrays along the way, we enable the structural mutations for selection. If in the end structural mutation is chosen, the probability at each level is the same
    /// The result is the input value generated by the mutation and an optional directive to mutate an array structurally
    pub fn mutate_input_value(
        &self,
        abi_type: &AbiType,
        previous_input: &InputValue,
        prng: &mut XorShiftRng,
        weight_tree_node: &NodeWeight,
        mutation_weight: u32,
        arrays_hit: usize,
    ) -> (InputValue, Option<usize>) {
        // Check if this is a leaf type. If it is and the element is in an array,
        // then we can trigger structural mutations for one of the arrays this element is in by randomly selecting the level
        match abi_type {
            AbiType::Boolean
            | AbiType::Field
            | AbiType::Integer { .. }
            | AbiType::String { .. } => match BASIC_TOP_LEVEL_MUTATION_CONFIGURATION.select(prng) {
                configurations::TopLevelMutation::Value => (),
                configurations::TopLevelMutation::Structure => {
                    if !arrays_hit.is_zero() {
                        return (previous_input.clone(), Some(prng.gen_range(1..=arrays_hit)));
                    }
                }
            },
            AbiType::Array { .. } | AbiType::Struct { .. } | AbiType::Tuple { .. } => (),
        }
        match abi_type {
            // Boolean only has 2 values, there is no point in performing complex logic
            AbiType::Boolean => {
                (InputValue::Field(FieldElement::from(prng.gen_range(0u32..=1u32))), None)
            }
            AbiType::Field => (
                mutate_field_input_value(
                    previous_input,
                    &self.full_dictionary.get_field_dictionary(),
                    prng,
                ),
                None,
            ),
            AbiType::Integer { sign, width } => (
                mutate_int_input_value(
                    previous_input,
                    sign,
                    *width,
                    &self.full_dictionary.get_int_dictionary(),
                    prng,
                ),
                None,
            ),
            AbiType::String { length: _ } => (
                mutate_string_input_value(
                    previous_input,
                    prng,
                    &self.full_dictionary.get_int_dictionary(),
                ),
                None,
            ),
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let input_vector = match previous_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                // This is an array and can be structurally mutated if the number of elements is more than one
                let arrays_hit = arrays_hit + ((length > 1) as usize);
                let mut structural_mutation_directive = None;
                let mut element_vector_with_value_mutation = (0..length)
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|(idx, weight_node)| {
                        if mutation_weight >= weight_node.start && mutation_weight < weight_node.end
                        {
                            let (mutation_result, directive) = self.mutate_input_value(
                                typ,
                                &input_vector[idx],
                                prng,
                                weight_node,
                                mutation_weight,
                                arrays_hit,
                            );
                            structural_mutation_directive = directive;
                            mutation_result
                        } else {
                            input_vector[idx].clone()
                        }
                    })
                    .collect();

                // If there is a command to structurally mutate an array
                if let Some(array_level_to_mutate) = structural_mutation_directive {
                    // And it's this array that should be mutated
                    if length > 1 && arrays_hit == array_level_to_mutate {
                        // mutate array
                        element_vector_with_value_mutation =
                            mutate_vector_structure(&element_vector_with_value_mutation, prng);
                        // disable directive
                        structural_mutation_directive = None;
                    }
                }
                (InputValue::Vec(element_vector_with_value_mutation), structural_mutation_directive)
            }

            AbiType::Struct { fields, .. } => {
                let input_struct = match previous_input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let mut structural_mutation_directive = None;
                let fields: Vec<(String, InputValue)> = fields
                    .iter()
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|((name, typ), weight_node)| {
                        (
                            name.clone(),
                            if mutation_weight >= weight_node.start
                                && mutation_weight < weight_node.end
                            {
                                let (element_result, new_directive) = self.mutate_input_value(
                                    typ,
                                    &input_struct[name],
                                    prng,
                                    weight_node,
                                    mutation_weight,
                                    arrays_hit,
                                );
                                structural_mutation_directive = new_directive;
                                element_result
                            } else {
                                input_struct[name].clone()
                            },
                        )
                    })
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                (InputValue::Struct(fields), structural_mutation_directive)
            }

            AbiType::Tuple { fields } => {
                let input_vector = match previous_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let mut structural_mutation_directive = None;
                let fields: Vec<_> = zip(fields, input_vector)
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|((typ, previous_tuple_input), weight_node)| {
                        if mutation_weight >= weight_node.start && mutation_weight < weight_node.end
                        {
                            let (element_result, new_directive) = self.mutate_input_value(
                                typ,
                                previous_tuple_input,
                                prng,
                                weight_node,
                                mutation_weight,
                                arrays_hit,
                            );
                            structural_mutation_directive = new_directive;
                            element_result
                        } else {
                            previous_tuple_input.clone()
                        }
                    })
                    .collect();
                (InputValue::Vec(fields), structural_mutation_directive)
            }
        }
    }

    pub fn mutate_input_map_single(
        &self,
        previous_input_map: &InputMap,
        prng: &mut XorShiftRng,
    ) -> InputMap {
        let chosen_weight = prng.gen_range(0..self.weight_tree.get_weight());
        let current_level_weight_tree = self.weight_tree.subnodes.as_ref().unwrap();
        self.abi
            .parameters
            .iter()
            .enumerate()
            .map(|(idx, param)| {
                (
                    param.name.clone(),
                    if chosen_weight >= current_level_weight_tree[idx].start
                        && chosen_weight < current_level_weight_tree[idx].end
                    {
                        self.mutate_input_value(
                            &param.typ,
                            &previous_input_map[&param.name],
                            prng,
                            &current_level_weight_tree[idx],
                            chosen_weight,
                            0,
                        )
                        .0
                    } else {
                        previous_input_map[&param.name].clone()
                    },
                )
            })
            .collect()
    }

    /// Recurse over elements and pick them according to the unbalanced configuration (80/20 first to second in the basic case)
    /// Arrays and string also sometimes use complex splicing which picks out chunks
    pub fn splice_unbalanced(
        abi_type: &AbiType,
        first_input: &InputValue,
        second_input: &InputValue,
        prng: &mut XorShiftRng,
    ) -> InputValue {
        match abi_type {
            // For a single-element type pick one based on the unbalanced schedule
            AbiType::Boolean | AbiType::Field | AbiType::Integer { .. } => {
                match BASIC_UNBALANCED_SLICE_CONFIGURATION.select(prng) {
                    UnbalancedSplice::FirstTestCase => first_input.clone(),
                    UnbalancedSplice::SecondTestCase => second_input.clone(),
                }
            }

            // For string, with a 50% chance pick one based on the unbalanced schedule, with 50% splice with string splicing methods
            AbiType::String { length: _ } => match prng.gen_range(0..2) {
                0 => match BASIC_UNBALANCED_SLICE_CONFIGURATION.select(prng) {
                    UnbalancedSplice::FirstTestCase => first_input.clone(),
                    UnbalancedSplice::SecondTestCase => second_input.clone(),
                },
                _ => splice_string_input_value(first_input, second_input, prng),
            },
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let first_input_vector = match first_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_vector = match second_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                assert!(!length.is_zero());
                // If array is a single element, recurse
                if length == 1 {
                    return InputValue::Vec(
                        [Self::splice_unbalanced(
                            &typ,
                            first_input_vector.first().unwrap(),
                            second_input_vector.first().unwrap(),
                            prng,
                        )]
                        .to_vec(),
                    );
                }
                match BASIC_UNBALANCED_ARRAY_SPLICE_MUTATION_CONFIGURATION.select(prng) {
                    configurations::UnbalancedArraySpliceType::ArraySpecific => {
                        splice_array_structure(first_input, second_input, prng)
                    }
                    configurations::UnbalancedArraySpliceType::Recurse => InputValue::Vec(
                        (0..length)
                            .map(|idx| {
                                Self::splice_unbalanced(
                                    typ,
                                    &first_input_vector[idx],
                                    &second_input_vector[idx],
                                    prng,
                                )
                            })
                            .collect(),
                    ),
                }
            }

            // Go over each structure member and pick according to unbalanced schedule
            AbiType::Struct { fields, .. } => {
                let first_input_struct = match first_input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_struct = match second_input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let fields: Vec<(String, InputValue)> = fields
                    .iter()
                    .map(|(name, typ)| {
                        (
                            name.clone(),
                            Self::splice_unbalanced(
                                typ,
                                &first_input_struct[name],
                                &second_input_struct[name],
                                prng,
                            ),
                        )
                    })
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                InputValue::Struct(fields)
            }

            // In case of tuple just go over each element and pick according to unbalanced schedule
            AbiType::Tuple { fields } => {
                let first_input_vector = match first_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_vector = match second_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let fields: Vec<_> = zip(fields, first_input_vector)
                    .zip(second_input_vector)
                    .map(|((typ, first_tuple_input), second_tuple_input)| {
                        Self::splice_unbalanced(typ, first_tuple_input, second_tuple_input, prng)
                    })
                    .collect();
                InputValue::Vec(fields)
            }
        }
    }

    /// Get a single value from the second input in the new testcase
    pub fn splice_import_single_input_value(
        abi_type: &AbiType,
        first_input: &InputValue,
        second_input: &InputValue,
        prng: &mut XorShiftRng,
        weight_tree_node: &NodeWeight,
        mutation_weight: u32,
    ) -> InputValue {
        match abi_type {
            // If we run into a leaf type, we need to get the second one
            AbiType::Boolean
            | AbiType::Field
            | AbiType::Integer { .. }
            | AbiType::String { .. } => second_input.clone(),
            // For array, struct and tuple we copy all the elements from the first vector apart from the element with weight selected by mutation
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let first_input_vector = match first_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_vector = match second_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                assert!(!length.is_zero());
                InputValue::Vec(
                    (0..length)
                        .zip(weight_tree_node.subnodes.as_ref().unwrap())
                        .map(|(idx, weight_node)| {
                            if mutation_weight >= weight_node.start
                                && mutation_weight < weight_node.end
                            {
                                Self::splice_import_single_input_value(
                                    typ,
                                    &first_input_vector[idx],
                                    &second_input_vector[idx],
                                    prng,
                                    weight_node,
                                    mutation_weight,
                                )
                            } else {
                                first_input_vector[idx].clone()
                            }
                        })
                        .collect(),
                )
            }

            AbiType::Struct { fields, .. } => {
                let first_input_struct = match first_input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_struct = match second_input {
                    InputValue::Struct(previous_input_struct) => previous_input_struct,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let fields: Vec<(String, InputValue)> = fields
                    .iter()
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|((name, typ), weight_node)| {
                        (
                            name.clone(),
                            if mutation_weight >= weight_node.start
                                && mutation_weight < weight_node.end
                            {
                                Self::splice_import_single_input_value(
                                    typ,
                                    &first_input_struct[name],
                                    &second_input_struct[name],
                                    prng,
                                    weight_node,
                                    mutation_weight,
                                )
                            } else {
                                first_input_struct[name].clone()
                            },
                        )
                    })
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                InputValue::Struct(fields)
            }

            AbiType::Tuple { fields } => {
                let first_input_vector = match first_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let second_input_vector = match second_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let fields: Vec<_> = zip(fields, first_input_vector)
                    .zip(second_input_vector)
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|(((typ, first_tuple_input), second_tuple_input), weight_node)| {
                        if mutation_weight >= weight_node.start && mutation_weight < weight_node.end
                        {
                            Self::splice_import_single_input_value(
                                typ,
                                first_tuple_input,
                                second_tuple_input,
                                prng,
                                weight_node,
                                mutation_weight,
                            )
                        } else {
                            first_tuple_input.clone()
                        }
                    })
                    .collect();
                InputValue::Vec(fields)
            }
        }
    }

    /// Use one of splicing mechanisms to produce a new testcase generated by merging two previous ones
    pub fn splice_two_maps(
        &self,
        first_input_map: &InputMap,
        second_input_map: &InputMap,
        prng: &mut XorShiftRng,
    ) -> InputMap {
        // TODO: add the most basic splice where the whole map is converted into a witness map and then we splice those
        match BASIC_TESTCASE_SPLICE_CONFIGURATION.select(prng) {
            TestCaseSpliceType::BalancedTopLevel => {
                self // Randomly pick top-level objects with 50/50 probability
                    .abi
                    .parameters
                    .iter()
                    .map(|param| {
                        (
                            param.name.clone(),
                            if prng.gen_bool(0.5) {
                                first_input_map[&param.name].clone()
                            } else {
                                second_input_map[&param.name].clone()
                            },
                        )
                    })
                    .collect()
            }
            TestCaseSpliceType::UnbalancedFull => {
                self // Randomly pick low-level elements with a distribution slightly gravitating towards the first testcase
                    .abi
                    .parameters
                    .iter()
                    .map(|param| {
                        (
                            param.name.clone(),
                            Self::splice_unbalanced(
                                &param.typ,
                                &first_input_map[&param.name],
                                &second_input_map[&param.name],
                                prng,
                            ),
                        )
                    })
                    .collect()
            }
            TestCaseSpliceType::SingleElementImport => {
                // Import one low-level element from the second testcase into the first one
                // Pick an element to import in the whole input
                let chosen_weight = prng.gen_range(0..self.weight_tree.get_weight());
                let current_level_weight_tree = self.weight_tree.subnodes.as_ref().unwrap();
                self.abi
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(idx, param)| {
                        (
                            param.name.clone(),
                            if chosen_weight >= current_level_weight_tree[idx].start
                                && chosen_weight < current_level_weight_tree[idx].end
                            {
                                Self::splice_import_single_input_value(
                                    &param.typ,
                                    &first_input_map[&param.name],
                                    &second_input_map[&param.name],
                                    prng,
                                    &current_level_weight_tree[idx],
                                    chosen_weight,
                                )
                            } else {
                                first_input_map[&param.name].clone()
                            },
                        )
                    })
                    .collect()
            }
        }
    }

    /// Create a mutated input for use in fuzzing
    /// Picks a number of mutations ({1,2,4,8,16,32}) and applies random mutations to the inputs
    pub fn generate_mutated_input(
        &self,
        previous_input_map: InputMap,
        additional_input_map: Option<InputMap>,
        prng: &mut XorShiftRng,
    ) -> InputMap {
        let mut starting_input_value = previous_input_map.clone();

        if additional_input_map.is_some() && prng.gen_range(0..4).is_zero() {
            starting_input_value =
                self.splice_two_maps(&previous_input_map, &additional_input_map.unwrap(), prng);
        }
        for _ in 0..(1 << prng.gen_range(MUTATION_LOG_MIN..=MUTATION_LOG_MAX)) {
            starting_input_value = self.mutate_input_map_single(&starting_input_value, prng);
        }
        starting_input_value
    }

    /// Generate the default input value for a given type
    /// false for boolean, 0 for integers and field elements and recursively defined through the first three for others
    pub fn generate_default_input_value(abi_type: &AbiType) -> InputValue {
        match abi_type {
            // Field integer and boolean are 0
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => {
                InputValue::Field(FieldElement::zero())
            }

            // Default string is zero-filled
            AbiType::String { length } => {
                InputValue::String(String::from_utf8(vec![0x0u8; *length as usize]).unwrap())
            }

            // Array uses default values of its type
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                InputValue::Vec(
                    (0..length).map(|_x| Self::generate_default_input_value(typ)).collect(),
                )
            }

            // Structure is recursively filled out with default values of its members
            AbiType::Struct { fields, .. } => {
                let fields: Vec<(String, InputValue)> = fields
                    .iter()
                    .map(|(name, typ)| (name.clone(), Self::generate_default_input_value(typ)))
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                InputValue::Struct(fields)
            }

            // Tuple is recursively filled out with default values of its members
            AbiType::Tuple { fields } => {
                let fields: Vec<_> =
                    fields.iter().map(Self::generate_default_input_value).collect();
                InputValue::Vec(fields)
            }
        }
    }

    /// Generate an input map consisting of default values (0 for field, false for boolean, etc)
    /// Used to initialize the corpus for the fuzzer, since the input can't be empty as usually in binary fuzzers
    pub fn generate_default_input_map(&self) -> InputMap {
        self.abi
            .parameters
            .iter()
            .map(|param| (param.name.clone(), Self::generate_default_input_value(&param.typ)))
            .collect()
    }
}
