use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use clap::ArgMatches;
use noir_field::FieldElement;
use noirc_abi::{input_parser::InputValue, Abi};
use pwg::Solver;

use crate::resolver::Resolver;

use super::{create_dir, write_to_file, PROOFS_DIR, PROOF_EXT};

pub(crate) fn run(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("prove")
        .unwrap()
        .value_of("proof_name")
        .unwrap();

    prove(proof_name);
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn prove(proof_name: &str) {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, backend_ptr) = Resolver::resolve_root_config(&curr_dir);
    let compiled_program = driver.into_compiled_program(backend_ptr);

    // Parse the initial witness values
    let mut path_to_src_dir = std::env::current_dir().unwrap();
    path_to_src_dir.push(std::path::PathBuf::from("src"));
    let witness_map = noirc_abi::input_parser::Format::Toml.parse(path_to_src_dir);

    // Check that enough witness values were supplied
    let num_params = compiled_program.abi.as_ref().unwrap().num_parameters();
    if num_params != witness_map.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            num_params,
            witness_map.len()
        )
    }

    let abi = compiled_program.abi.unwrap();
    let mut solved_witness = process_abi_with_input(abi, witness_map);

    Solver::solve(&mut solved_witness, compiled_program.circuit.gates.clone());

    let proof = backend_ptr
        .backend()
        .prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_proof_dir();
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path)
}

fn create_proof_dir() -> PathBuf {
    create_dir(PROOFS_DIR).expect("could not create the `contract` directory")
}

/// Ordering is important here, which is why we need the ABI to tell us what order to add the elements in
/// We then need the witness map to get the elements field values.
fn process_abi_with_input(
    abi: Abi,
    witness_map: BTreeMap<String, InputValue>,
) -> BTreeMap<Witness, FieldElement> {
    let mut solved_witness = BTreeMap::new();

    let param_names = abi.parameter_names();
    let mut index = 0;

    for param in param_names.into_iter() {
        // XXX: This is undesirable as we are eagerly allocating, but it avoids duplication
        let err_msg = &format!(
            "ABI expects the parameter `{}`, but this was not found",
            param
        );

        let value = witness_map.get(param).expect(err_msg).clone();
        match value {
            InputValue::Field(element) => {
                let old_value =
                    solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
                assert!(old_value.is_none());
                index += 1;
            }
            InputValue::Vec(arr) => {
                for element in arr {
                    let old_value =
                        solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
                    assert!(old_value.is_none());
                    index += 1;
                }
            }
        }
    }
    solved_witness
}
