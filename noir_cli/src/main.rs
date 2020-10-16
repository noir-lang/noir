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
use noirc_frontend::analyser;
use noirc_frontend::ast::Type;
use acir::native_types::Witness;
use acir::circuit::Circuit;
use noir_evaluator::Evaluator;
use noir_field::FieldElement;
use acir::partial_witness_generator::Solver;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Abi {
    parameters: Vec<(String,Type)>,
}

impl Abi {
    // In barretenberg, we need to add public inputs first
    // currently there does not seem to be a way to add a witness and then a public input
    // So we have this special function to sort for barretenberg. 
    // It will need to be abstracted away or hidden behind the aztec_backend
    fn sort_by_public_input(mut self) -> Self {
        let comparator = |a: &(String,Type),b: &(String,Type)| {
            let typ_a = &a.1;
            let typ_b = &b.1;

            if typ_a == &Type::Public && typ_b == &Type::Public {
                std::cmp::Ordering::Equal
            } else if typ_a == &Type::Public {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
            
        };

        self.parameters.sort_by(comparator);
        self

    }

    fn parameter_names(&self) -> Vec<&String> {
        self.parameters.iter().map(|x|&x.0).collect()
    }

    fn len(&self) -> usize {
        self.parameters.len()
    }
}

struct CompiledMain {
    standard_format_cs: ConstraintSystem,
    circuit: Circuit,
    abi: Abi,
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
    let (checked_program, symbol_table) = analyser::check(program);

    let abi = checked_program.abi().unwrap();

    let evaluator = Evaluator::new(checked_program, symbol_table);

    let (circuit, num_witnesses, num_public_inputs) = evaluator.evaluate();

    let constraint_system =
        aztec_backend::serialise_circuit(&circuit, num_witnesses, num_public_inputs);

    hash_constraint_system(&constraint_system);

    CompiledMain {
        standard_format_cs: constraint_system,
        circuit,
        abi : Abi{parameters: abi},
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

    let input = 
    r#"
        x = "5"
        y = "10"
    "#;

    let bin_dir = package_dir.join(Path::new(BINARY_DIR));

    let _ = write_to_file(input.as_bytes(), &bin_dir.join(Path::new("input.toml")));
    let path = write_to_file(example.as_bytes(), &&bin_dir.join(Path::new("main.noir")));
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

    // Parse the input.toml file
    let witness_map = parse_input();

    // Compile main
    let compiled_main = build_main();

    // Check that enough witness values were supplied
    if compiled_main.abi.len() != witness_map.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            compiled_main.abi.len(),
            witness_map.len()
        )
    }

    let mut solved_witness = BTreeMap::new();

    let sorted_abi = compiled_main.abi.sort_by_public_input();
    let param_names = sorted_abi.parameter_names();
    for (index, param) in param_names.into_iter().enumerate() {
        
        let value = witness_map.get(param).expect(&format!("ABI expects the parameter `{}`, but this was not found in input.toml", param));
        
        solved_witness.insert(
            Witness::new(param.clone(), index + WITNESS_OFFSET),
            value.clone(),
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

fn parse_input() -> BTreeMap<String, FieldElement> {

    // Get the path to the input file
    let mut input_file = std::env::current_dir().unwrap();
    input_file.push(std::path::PathBuf::from("bin"));
    input_file.push(std::path::PathBuf::from("input.toml"));
    assert!(
        input_file.exists(),
        "Cannot find input file at located {}",
        input_file.display()
    );

    // Closure to parse a string to a field element
    let parse = |value : &str| -> FieldElement {
        if value.starts_with("0x") {
            FieldElement::from_hex(value).expect("Could not parse hex value")                   
         } else {
             let val : i128 = value
             .parse()
             .expect("Expected witness values to be integers");

             FieldElement::from(val)
         }
    };

    // Get input.toml file as a string
    let input_as_string = std::fs::read_to_string(input_file).unwrap();

    // Parse input.toml into a BTreeMap, converting the argument to field elements 
    let data : BTreeMap<String, String> = toml::from_str(&input_as_string).expect("input.toml file is badly formed, could not parse");
    data.into_iter().map(|(parameter, argument)| (parameter, parse(&argument))).collect()
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
