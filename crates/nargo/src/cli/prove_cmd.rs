use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::PartialWitnessGenerator;
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::{input_parser::InputValue, Abi};
use std::path::Path;

use crate::{errors::CliError, resolver::Resolver};

use super::{create_dir, write_to_file, PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let proof_name = args
        .subcommand_matches("prove")
        .unwrap()
        .value_of("proof_name")
        .unwrap();
    let interactive = args
        .subcommand_matches("prove")
        .unwrap()
        .value_of("interactive");
        let mut is_interactive = false;
    if let Some(int) = interactive {
        if int == "i" {
            is_interactive = true;
        }
    }
    prove(proof_name, is_interactive)
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn prove(proof_name: &str, interactive: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new();
    proof_path.push(PROOFS_DIR);
    let result = prove_with_path(proof_name, curr_dir, proof_path, interactive);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn create_proof_dir(proof_dir: PathBuf) -> PathBuf {
    create_dir(proof_dir).expect("could not create the `contract` directory")
}

/// Ordering is important here, which is why we need the ABI to tell us what order to add the elements in
/// We then need the witness map to get the elements field values.
fn process_abi_with_input(
    abi: Abi,
    witness_map: BTreeMap<String, InputValue>,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    let mut solved_witness = BTreeMap::new();

    let mut index = 0;

    for (param_name, param_type) in abi.parameters.into_iter() {
        let value = witness_map
            .get(&param_name)
            .unwrap_or_else(|| {
                panic!(
                    "ABI expects the parameter `{}`, but this was not found",
                    param_name
                )
            })
            .clone();

        if !value.matches_abi(param_type) {
            return Err(CliError::Generic(format!("The parameters in the main do not match the parameters in the {}.toml file. \n Please check `{}` parameter ", PROVER_INPUT_FILE,param_name)));
        }

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
    Ok(solved_witness)
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: &str,
    program_dir: P,
    proof_dir: P,
    interactive: bool,
) -> Result<PathBuf, CliError> {
    let driver = Resolver::resolve_root_config(program_dir.as_ref())?;
    let backend = crate::backends::ConcreteBackend;
    let compiled_program = driver.into_compiled_program(backend.np_language(), interactive);

    // Parse the initial witness values
    let witness_map = noirc_abi::input_parser::Format::Toml.parse(program_dir, PROVER_INPUT_FILE);

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
    let mut solved_witness = process_abi_with_input(abi, witness_map)?;

    let solver_res = backend.solve(&mut solved_witness, compiled_program.circuit.gates.clone());

    if let Err(opcode) = solver_res {
        return Err(CliError::Generic(format!(
            "backend does not currently support the {} opcode. ACVM does not currently fall back to arithmetic gates.",
            opcode
        )));
    }

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_proof_dir(proof_dir.as_ref().to_path_buf());
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path);
    println!("{:?}", std::fs::canonicalize(&proof_path));

    Ok(proof_path)
}
