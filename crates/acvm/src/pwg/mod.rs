use acir::{circuit::gate::GadgetInput, native_types::Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

// Re-usable methods that backends can use to implement their PWG
// XXX: This can possible be refactored to be default trait methods

pub mod arithmetic;
pub mod hash;
pub mod logic;
pub mod signature;

pub fn input_to_value<'a, F: FieldElement>(
    witness_map: &'a BTreeMap<Witness, F>,
    input: &GadgetInput,
) -> &'a F {
    match witness_map.get(&input.witness) {
        None => panic!("Cannot find witness assignment for {:?}", input),
        Some(assignment) => assignment,
    }
}
