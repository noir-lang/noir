use acvm::FieldElement;

use field::mutate_field_input_value;
use int::{mutate_int_input_value, IntDictionary};
use noirc_abi::{input_parser::InputValue, Abi, AbiType, InputMap};
use rand::Rng;
use rand_xorshift::XorShiftRng;
use std::{
    collections::{BTreeMap, HashSet},
    iter::zip,
};

mod field;
mod int;
mod uint;
pub struct InputMutator {
    abi: Abi,
    weight_tree: NodeWeight,
    full_dictionary: FullDictionary,
}
pub struct FullDictionary {
    original_dictionary: Vec<FieldElement>,
    int_dictionary: IntDictionary,
}

impl FullDictionary {
    pub fn new(original_dictionary: &HashSet<FieldElement>) -> Self {
        let dictionary_vector: Vec<_> = original_dictionary.iter().copied().collect();
        let int_dict = IntDictionary::new(&dictionary_vector);
        Self { original_dictionary: dictionary_vector, int_dictionary: int_dict }
    }
}
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
impl InputMutator {
    pub fn new(abi: &Abi, original_dictionary: &HashSet<FieldElement>) -> Self {
        let mut weight_tree = Self::count_all_input_weights(abi);
        weight_tree.calculate_offsets(0);
        Self {
            abi: abi.clone(),
            weight_tree,
            full_dictionary: FullDictionary::new(original_dictionary),
        }
    }

    /// Count weights of each element recursively (complex structures return a vector of weights of their most basic elements)
    fn count_single_input_weight(abi_type: &AbiType) -> NodeWeight {
        match abi_type {
            AbiType::Boolean => NodeWeight { start: 0u32, end: 1u32, subnodes: None },
            AbiType::Field | AbiType::Integer { .. } => {
                NodeWeight { start: 0u32, end: 8u32, subnodes: None }
            }
            AbiType::String { length } => NodeWeight { start: 0u32, end: *length, subnodes: None },
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let subnode_weight = Self::count_single_input_weight(&typ);
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
                for single in fields.iter().map(|(_, typ)| Self::count_single_input_weight(&typ)) {
                    total_node_weight += &single.get_weight();
                    weights.push(single);
                }

                NodeWeight { start: 0, end: total_node_weight, subnodes: Some(weights) }
            }

            AbiType::Tuple { fields } => {
                let mut weights = Vec::new();

                let mut total_node_weight = 0u32;
                for single in fields.iter().map(|typ| Self::count_single_input_weight(&typ)) {
                    total_node_weight += &single.get_weight();
                    weights.push(single);
                }

                NodeWeight { start: 0, end: total_node_weight, subnodes: Some(weights) }
            }
        }
    }

    /// Recurse through all the inputs in the ABI and collect weights of every input so we don't get affected by the depth
    fn count_all_input_weights(abi: &Abi) -> NodeWeight {
        assert!(abi.parameters.len() > 0);
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

    pub fn mutate_input_value(
        &self,
        abi_type: &AbiType,
        previous_input: &InputValue,
        prng: &mut XorShiftRng,
        weight_tree_node: &NodeWeight,
        mutation_weight: u32,
    ) -> InputValue {
        match abi_type {
            // Boolean only has 2 values, there is no point in performing complex logic
            AbiType::Boolean => InputValue::Field(FieldElement::from(prng.gen_range(0u32..=1u32))),
            // Mutate fields in a smart way
            AbiType::Field => mutate_field_input_value(
                previous_input,
                &self.full_dictionary.original_dictionary,
                prng,
            ),
            // TODO: IMPLEMENT THESE
            AbiType::Integer { sign, width } => mutate_int_input_value(
                previous_input,
                sign,
                *width,
                &self.full_dictionary.int_dictionary,
                prng,
            ),
            AbiType::String { length } => {
                InputValue::String(String::from_utf8(vec![0x41u8; *length as usize]).unwrap())
            }
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                let input_vector = match previous_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                InputValue::Vec(
                    (0..length)
                        .zip(weight_tree_node.subnodes.as_ref().unwrap())
                        .map(|(idx, weight_node)| {
                            if mutation_weight >= weight_node.start
                                && mutation_weight < weight_node.end
                            {
                                self.mutate_input_value(
                                    typ,
                                    &input_vector[idx],
                                    prng,
                                    weight_node,
                                    mutation_weight,
                                )
                            } else {
                                input_vector[idx].clone()
                            }
                        })
                        .collect(),
                )
            }

            AbiType::Struct { fields, .. } => {
                let input_struct = match previous_input {
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
                                self.mutate_input_value(
                                    typ,
                                    &input_struct[name],
                                    prng,
                                    weight_node,
                                    mutation_weight,
                                )
                            } else {
                                input_struct[name].clone()
                            },
                        )
                    })
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                InputValue::Struct(fields)
            }

            AbiType::Tuple { fields } => {
                let input_vector = match previous_input {
                    InputValue::Vec(previous_input_vector) => previous_input_vector,
                    _ => panic!("Mismatch of AbiType and InputValue should not happen"),
                };
                let fields: Vec<_> = zip(fields, input_vector)
                    .zip(weight_tree_node.subnodes.as_ref().unwrap())
                    .map(|((typ, previous_tuple_input), weight_node)| {
                        if mutation_weight >= weight_node.start && mutation_weight < weight_node.end
                        {
                            self.mutate_input_value(
                                typ,
                                previous_tuple_input,
                                prng,
                                weight_node,
                                mutation_weight,
                            )
                        } else {
                            previous_tuple_input.clone()
                        }
                    })
                    .collect();
                InputValue::Vec(fields)
            }
        }
    }

    pub fn mutate_input_map(
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
                        )
                    } else {
                        previous_input_map[&param.name].clone()
                    },
                )
            })
            .collect()
    }

    /// Generate the default input value for a given type
    /// false for boolean, 0 for integers and field elements and recursively defined through the first three for others
    pub fn generate_default_input_value(abi_type: &AbiType) -> InputValue {
        match abi_type {
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => {
                InputValue::Field(0i128.into())
            }

            AbiType::String { length } => {
                InputValue::String(String::from_utf8(vec![0x0u8; *length as usize]).unwrap())
            }
            AbiType::Array { length, typ } => {
                let length = *length as usize;
                InputValue::Vec(
                    (0..=length).map(|_x| Self::generate_default_input_value(typ)).collect(),
                )
            }

            AbiType::Struct { fields, .. } => {
                let fields: Vec<(String, InputValue)> = fields
                    .iter()
                    .map(|(name, typ)| (name.clone(), Self::generate_default_input_value(typ)))
                    .collect();

                let fields: BTreeMap<_, _> = fields.into_iter().collect();
                InputValue::Struct(fields)
            }

            AbiType::Tuple { fields } => {
                let fields: Vec<_> =
                    fields.iter().map(|typ| Self::generate_default_input_value(typ)).collect();
                InputValue::Vec(fields)
            }
        }
    }

    /// Generate an input map consisting of default values (0 for field, false for boolean, etc)
    /// Used to initialize the corpus for the fuzzer, since the input can't be empty as usually in fuzzers
    pub fn generate_default_input_map(&self) -> InputMap {
        self.abi
            .parameters
            .iter()
            .map(|param| (param.name.clone(), Self::generate_default_input_value(&param.typ)))
            .collect()
    }
}
