use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use clap::ArgMatches;
use pwg::Solver;

use crate::resolver::Resolver;

use super::{create_dir, write_to_file, PROOFS_DIR};

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
    let mut path_to_toml = std::env::current_dir().unwrap();
    path_to_toml.push(std::path::PathBuf::from("src"));
    path_to_toml.push(std::path::PathBuf::from("input.toml"));
    let (witness_map, collection_names) = noirc_abi::input_parser::Format::Toml.parse(path_to_toml);

    // Check that enough witness values were supplied
    if compiled_program.abi.as_ref().unwrap().len() != witness_map.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            compiled_program.abi.as_ref().unwrap().len(),
            witness_map.len()
        )
    }

    let mut solved_witness = BTreeMap::new();

    let abi = compiled_program.abi.unwrap();
    let param_names = abi.parameter_names();
    let mut index = 0;

    for param in param_names.into_iter() {
        // XXX: This is undesirable as we are eagerly allocating, but it avoids duplication
        let err_msg = &format!(
            "ABI expects the parameter `{}`, but this was not found in input.toml",
            param
        );

        // Note: the collection name will not be in the witness_map
        // only mangled_names for it's elements
        if let Some(collection) = collection_names.iter().find(|(name, _)| name == param) {
            for i in 0..collection.1 {
                let mangled_element_name =
                    noirc_abi::input_parser::mangle_array_element_name(&collection.0, i);
                let value = witness_map.get(&mangled_element_name).expect(err_msg);

                let old_value =
                    solved_witness.insert(Witness::new(index + WITNESS_OFFSET), value.clone());
                assert!(old_value.is_none());

                index += 1
            }
        } else {
            let value = witness_map.get(param).expect(err_msg);

            let old_value =
                solved_witness.insert(Witness::new(index + WITNESS_OFFSET), value.clone());
            assert!(old_value.is_none());

            index += 1;
        }
    }

    Solver::solve(&mut solved_witness, compiled_program.circuit.gates.clone());

    let proof = backend_ptr
        .backend()
        .prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_proof_dir();
    proof_path.push(proof_name);
    proof_path.set_extension("proof");

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path)
}

fn create_proof_dir() -> PathBuf {
    create_dir(PROOFS_DIR).expect("could not create the `contract` directory")
}
