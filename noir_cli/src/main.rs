use clap::{App, Arg};

fn main() {
    let matches = App::new("noir")
        .about("A Domain Specific Language for PLONK")
        .version("0.1")
        .author("Kevaundray Wedderburn <kevtheappdev@gmail.com>")
        .subcommand(App::new("build").about("Builds the constraint system"))
        .subcommand(App::new("contract").about("Creates the smart contract code for circuit"))
        .subcommand(
            App::new("new").about("Create a new binary project").arg(
                Arg::with_name("package_name")
                    .help("Name of the package")
                    .required(true),
            ),
        )
        .subcommand(
            App::new("verify")
                .about("Given a proof and a program, verify whether the proof is valid")
                .arg(
                    Arg::with_name("proof")
                        .help("The proof to verify")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("prove")
                .about("Create proof for this program")
                .arg(
                    Arg::with_name("proof_name")
                        .help("The name of the proof")
                        .required(true),
                )
                .arg(
                    Arg::with_name("witness values")
                        .required(true)
                        .min_values(1)
                        .help("The values of the witness"),
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("new") => new_package(matches),
        Some("build") => {
            let _ = build_main();
            println!("Constraint system successfully built!")
        }
        Some("contract") => create_smart_contract(),
        Some("prove") => prove(matches),
        Some("verify") => verify(matches),
        None => println!("No subcommand was used"),
        _ => unreachable!(), // Assuming you've listed all direct children above, this is unreachable
    }
}

use aztec_backend::barretenberg_rs::composer::{Assignments, ConstraintSystem, StandardComposer};

use clap::ArgMatches;
use noirc_frontend::lexer::Lexer;
use noirc_frontend::Parser;
use acir::native_types::Witness;
use acir::circuit::Circuit;
use noir_evaluator::Evaluator;
use noir_field::FieldElement;
use acir::partial_witness_generator::Solver;
use std::collections::BTreeMap;

struct CompiledMain {
    standard_format_cs: ConstraintSystem,
    circuit: Circuit,
    abi: Vec<String>,
}

/// Looks for main.noir in the current directory
/// Returns the constraint system
/// The compiled circuit (DSL variation)
/// And the parameters for main
fn build_main() -> CompiledMain {
    let mut main_file = std::env::current_dir().unwrap();
    main_file.push(std::path::PathBuf::from("bin"));
    main_file.push(std::path::PathBuf::from("main.noir"));
    assert!(
        main_file.exists(),
        "Cannot find main file at located {}",
        main_file.display()
    );

    let file_as_string = std::fs::read_to_string(main_file).unwrap();

    let mut parser = Parser::new(Lexer::new(&file_as_string));
    let program = parser.parse_program();
    dbg!(program.clone());
    let (checked_program, symbol_table) = noirc_analyser::check(program);

    let abi = checked_program.abi().unwrap();

    let evaluator = Evaluator::new(checked_program, symbol_table);

    let (circuit, num_witnesses, num_public_inputs) = evaluator.evaluate();

    let constraint_system =
        aztec_backend::serialise_circuit(&circuit, num_witnesses, num_public_inputs);

    hash_constraint_system(&constraint_system);

    CompiledMain {
        standard_format_cs: constraint_system,
        circuit,
        abi,
    }
}

fn create_smart_contract() {

    let compiled_main = build_main();

    let mut composer = StandardComposer::new(compiled_main.standard_format_cs.size());

    let smart_contract_string = composer.smart_contract(&compiled_main.standard_format_cs);


    let mut proof_path = std::path::PathBuf::new();
    proof_path.push("contract");
    proof_path.push("plonk_vk");
    proof_path.set_extension("sol");

    let path = write_to_file(&smart_contract_string.as_bytes(), &proof_path);
    println!("Contract successfully created and located at {}", path)


}

fn new_package(args: ArgMatches) {
    let package_name = args
        .subcommand_matches("new")
        .unwrap()
        .value_of("package_name")
        .unwrap();

    let mut package_dir = std::env::current_dir().unwrap();
    package_dir.push(Path::new(package_name));

    const BINARY_DIR: &str = "bin";
    const PROOFS_DIR: &str = "proofs";
    const CONTRACT_DIR: &str = "contract";

    create_directory(&package_dir.join(Path::new(BINARY_DIR)));
    create_directory(&package_dir.join(Path::new(PROOFS_DIR)));
    create_directory(&package_dir.join(Path::new(CONTRACT_DIR)));

    let example = "
    fn main(x : Witness, y : Witness) {
        constrain x != y;
    }
    ";

    package_dir.push(Path::new(BINARY_DIR).join(Path::new("main.noir")));
    let path = write_to_file(example.as_bytes(), &package_dir);
    println!("Project successfully created! Binary located at {}", path);
}
fn verify(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("verify")
        .unwrap()
        .value_of("proof")
        .unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new("proofs"));
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension("noir");

    let proof: Vec<_> = std::fs::read(proof_path).unwrap();

    let compiled_main = build_main();

    let mut composer = StandardComposer::new(compiled_main.standard_format_cs.size());

    let public_inputs = None;
    let verified = composer.verify(&compiled_main.standard_format_cs, &proof, public_inputs);

    println!("Proof verified : {}\n", verified);
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: usize = 1;

fn prove(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("prove")
        .unwrap()
        .value_of("proof_name")
        .unwrap();

    let witness_values: Vec<FieldElement> = args
        .subcommand_matches("prove")
        .unwrap()
        .values_of("witness values")
        .unwrap()
        .map(|value| {
                if value.starts_with("0x") {
                   let val =  FieldElement::from_hex(value).expect("Could not parse hex value");
                    dbg!(val.clone());
                   val
                } else {
                    let val : i128 = value
                    .parse()
                    .expect("Expected witness values to be integers");

                    FieldElement::from(val)
                }
        })
        .collect();

    let compiled_main = build_main();

    // Check that enough witness values were supplied
    if compiled_main.abi.len() != witness_values.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            compiled_main.abi.len(),
            witness_values.len()
        )
    }

    let mut solved_witness = BTreeMap::new();

    // Since the Public values are added first. Even if the first parameter is a Witness, it may not be
    // The first in the witness Vector. We do however, still want people to enter the values as the ABI states, so we must
    // match the correct values to the correct indices
    for (index, (param, value)) in compiled_main
        .abi
        .into_iter()
        .zip(witness_values.into_iter())
        .enumerate()
    {
        solved_witness.insert(
            Witness::new(param, index + WITNESS_OFFSET),
            FieldElement::from(value),
        );
    }

    // Derive solution
    Solver::solve(&mut solved_witness, compiled_main.circuit.clone());

    let mut composer = StandardComposer::new(compiled_main.standard_format_cs.size());

    // Add witnesses in the correct order
    // Note: The witnesses are sorted via their witness index, since we implement Ord on Witness and use a BTreeMap
    let mut sorted_witness = Assignments::new();
    for (_, value) in solved_witness.iter() {
        sorted_witness.push(*value);
    }

    let proof = composer.create_proof(&compiled_main.standard_format_cs, sorted_witness);

    let mut proof_path = std::path::PathBuf::new();
    proof_path.push("proofs");
    proof_path.push(proof_name);
    proof_path.set_extension("noir");

    dbg!(hex::encode(proof.clone()));

    let path = write_to_file(&proof, &proof_path);
    println!("Proof successfully created and located at {}", path)
}

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => display.to_string(),
    }
}

fn create_directory(path: &std::path::Path) {
    if path.exists() {
        println!("This directory {} already exists", path.display());
        return;
    }
    std::fs::create_dir_all(path).unwrap();
}

fn hash_constraint_system(cs: &ConstraintSystem) {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(cs.to_bytes());
    let result = hasher.finalize();
    println!("hash of constraint system : {:x?}", &result[..]);
}
