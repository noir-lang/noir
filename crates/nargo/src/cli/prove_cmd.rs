use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::ProofSystemCompiler;
use acvm::{GateResolution, PartialWitnessGenerator};
use clap::ArgMatches;
use noirc_abi::encoding::process_abi_with_input;
use noirc_abi::errors::AbiError;
use noirc_abi::AbiType;
use noirc_abi::{input_parser::InputValue, Abi};
use std::path::Path;

use crate::errors::CliError;

use super::{
    create_named_dir, write_to_file, PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    prove(proof_name, show_ssa, allow_warnings)
}

fn prove(proof_name: &str, show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new();
    proof_path.push(PROOFS_DIR);
    let result = prove_with_path(proof_name, curr_dir, proof_path, show_ssa, allow_warnings);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn compile_circuit_and_witness<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_unused_variables: bool,
) -> Result<(noirc_driver::CompiledProgram, BTreeMap<Witness, FieldElement>), CliError> {
    let compiled_program = super::compile_cmd::compile_circuit(
        program_dir.as_ref(),
        show_ssa,
        allow_unused_variables,
    )?;
    let solved_witness = parse_and_solve_witness(program_dir, &compiled_program)?;
    Ok((compiled_program, solved_witness))
}

pub fn parse_and_solve_witness<P: AsRef<Path>>(
    program_dir: P,
    compiled_program: &noirc_driver::CompiledProgram,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    // Parse the initial witness values from Prover.toml
    let witness_map =
        noirc_abi::input_parser::Format::Toml.parse(&program_dir, PROVER_INPUT_FILE)?;

    // Solve the remaining witnesses
    let (solved_witness, return_value) = solve_witness(compiled_program, &witness_map)?;

    // Write public inputs into Verifier.toml
    let abi = compiled_program.abi.as_ref().unwrap();
    export_public_inputs(return_value, &solved_witness, &witness_map, abi, &program_dir)?;

    Ok(solved_witness)
}

fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<(BTreeMap<Witness, FieldElement>, Option<Witness>), CliError> {
    let abi = compiled_program.abi.as_ref().unwrap();
    let (mut solved_witness, return_value) = process_abi_with_input(abi.clone(), witness_map)
        .map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{} in the {}.toml file.", error, PROVER_INPUT_FILE))
            }
            _ => CliError::from(error),
        })?;

    let backend = crate::backends::ConcreteBackend;
    let solver_res = backend.solve(&mut solved_witness, compiled_program.circuit.gates.clone());

    match solver_res {
        GateResolution::UnsupportedOpcode(opcode) => return Err(CliError::Generic(format!(
                "backend does not currently support the {} opcode. ACVM does not currently fall back to arithmetic gates.",
                opcode
        ))),
        GateResolution::UnsatisfiedConstrain => return Err(CliError::Generic(
                "could not satisfy all constraints".to_string()
        )),
        GateResolution::Resolved => (),
        _ => unreachable!(),
    }

    Ok((solved_witness, return_value))
}

fn export_public_inputs<P: AsRef<Path>>(
    w_ret: Option<Witness>,
    solved_witness: &BTreeMap<Witness, FieldElement>,
    witness_map: &BTreeMap<String, InputValue>,
    abi: &Abi,
    path: P,
) -> Result<(), noirc_abi::errors::InputParserError> {
    // generate a name->value map for the public inputs, using the ABI and witness_map:
    let mut public_inputs = BTreeMap::new();
    for i in &abi.parameters {
        if i.1.is_public() {
            let v = &witness_map[&i.0];

            let iv = if matches!(*v, InputValue::Undefined) {
                let w_ret = w_ret.unwrap();
                match &i.1 {
                    AbiType::Array { length, .. } => {
                        let return_values = noirc_frontend::util::vecmap(0..*length, |i| {
                            *solved_witness.get(&Witness::new(w_ret.0 + i as u32)).unwrap()
                        });
                        InputValue::Vec(return_values)
                    }
                    _ => InputValue::Field(*solved_witness.get(&w_ret).unwrap()),
                }
            } else {
                v.clone()
            };
            public_inputs.insert(i.0.clone(), iv);
        }
    }
    //serialise public inputs into verifier.toml
    noirc_abi::input_parser::Format::Toml.serialise(&path, VERIFIER_INPUT_FILE, &public_inputs)
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: &str,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let (compiled_program, solved_witness) =
        compile_circuit_and_witness(program_dir, show_ssa, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path);
    println!("{:?}", std::fs::canonicalize(&proof_path));

    Ok(proof_path)
}
