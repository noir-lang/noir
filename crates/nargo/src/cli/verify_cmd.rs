use clap::ArgMatches;
use std::{collections::BTreeMap, path::Path, path::PathBuf};

use acvm::{acir::native_types::Witness, ProofSystemCompiler};
use noirc_abi::{
    errors::AbiError,
    input_parser::{Format, InputValue},
};
use noirc_driver::CompiledProgram;

use super::{compile_cmd::compile_circuit, read_inputs_from_file};
use crate::constants::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE};
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("verify").unwrap();

    let proof_name = args.value_of("proof").unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new(PROOFS_DIR));

    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let allow_warnings = args.is_present("allow-warnings");
    let result = verify(proof_name, allow_warnings)?;
    println!("Proof verified : {result}\n");
    Ok(())
}

fn verify(proof_name: &str, allow_warnings: bool) -> Result<bool, CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new(); //or cur_dir?
    proof_path.push(PROOFS_DIR);
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);
    verify_with_path(&curr_dir, &proof_path, false, allow_warnings)
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<bool, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let mut public_inputs = BTreeMap::new();

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let abi = compiled_program.abi.clone().unwrap();
    let abi_len = abi.field_count();
    let public_abi = abi.clone().public_abi();
    let num_pub_params = public_abi.num_parameters();

    // Verifier needs to know which public inputs were also
    // public outputs, so as to avoid them
    //
    /*
    We could disallow programs like:

    fn main(x : pub Field) -> pub Field { // Allow this as its pretty non-trivial
        x
    }

    This program would need to be transformed into
    fn main(x : pub Field) {}

    Note:

    fn main(x : Field) -> pub Field {
        x
    }

    may be useful if the user wants to return x along with
    a group of intermediate variables.

    */

    // We need to deduplicate in the case that a public input is also a public output.
    // This is needed because the .toml file will have two entries, one for the public input
    // and another for the return value. The backend's verifier should only get one of
    // these.
    // Example:
    //
    // fn main(x : pub Field) -> pub Field {
    //    x
    // }
    // Will produce a .toml file with two public fields `x` and `return`
    // When we pass it to the backend's verifier, they should only receive one value
    // because there is only one witness index that was made public.
    // Reading the toml file, will give us two as that procedure
    // does not know that `x` and `return` both share the same witness index.

    // We have the same witness indices and we need to deduplicate
    // them when reading the input
    // TODO: So most of this code is not easy to reason about,
    // TODO: because there is no clear way to answer the following questions:
    // TODO: Given a variable in the toml file, what is its witness index?
    // TODO:  -  Is that variable public or private
    // TODO Given a witness index which is known to be in the abi
    // TODO what is its associated ABI parameter?
    //
    if num_pub_params != 0 {
        let curr_dir = program_dir;
        public_inputs =
            read_inputs_from_file(curr_dir, VERIFIER_INPUT_FILE, Format::Toml, public_abi)?;
    }

    let valid_proof = verify_proof(compiled_program, public_inputs, load_proof(proof_path)?)?;

    Ok(valid_proof)
}

fn verify_proof(
    compiled_program: CompiledProgram,
    public_inputs: BTreeMap<String, InputValue>,
    proof: Vec<u8>,
) -> Result<bool, CliError> {
    let abi = compiled_program.abi.clone().unwrap();
    let abi_len = abi.field_count();
    let public_abi = abi.clone().public_abi();

    let (encoded_inputs, encoded_outputs) =
        public_abi.encode(&public_inputs, false).map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{error} in the {VERIFIER_INPUT_FILE}.toml file."))
            }
            _ => CliError::from(error),
        })?;

    let public_input_indices: Vec<&Witness> = compiled_program
        .circuit
        .public_inputs
        .0
        .iter()
        .filter(|index| index.0 <= abi_len)
        .collect();
    let public_output_indices: Vec<&Witness> =
        compiled_program.circuit.public_outputs.0.iter().collect();

    // This can contain duplicates if the strings in the ABI contain duplicates
    let encoded_inputs_outputs =
        encoded_inputs.into_iter().chain(encoded_outputs.into_iter().flatten());

    let indices_inputs_outputs = public_input_indices.into_iter().chain(public_output_indices);

    let sorted_deduplicated_indices_to_outputs: BTreeMap<_, _> =
        indices_inputs_outputs.zip(encoded_inputs_outputs).collect();

    let public_inputs: Vec<_> =
        sorted_deduplicated_indices_to_outputs.into_iter().map(|(_, value)| value).collect();

    let backend = crate::backends::ConcreteBackend;
    let valid_proof = backend.verify_from_cs(&proof, public_inputs, compiled_program.circuit);

    Ok(valid_proof)
}

fn load_proof<P: AsRef<Path>>(proof_path: P) -> Result<Vec<u8>, CliError> {
    let proof_hex: Vec<_> = std::fs::read(&proof_path)
        .map_err(|_| CliError::PathNotValid(proof_path.as_ref().to_path_buf()))?;
    let proof = hex::decode(proof_hex).map_err(CliError::ProofNotValid)?;

    Ok(proof)
}
