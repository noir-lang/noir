use std::{collections::BTreeMap, fs::File, io::Write, path::Path};

use acvm::acir::native_types::Witness;
use clap::ArgMatches;
use clap::{App, Arg};
use pwg::Solver;

use crate::resolver::Resolver;

pub fn start_cli() {
    let matches = App::new("nargo")
        .about("Noir's package manager")
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
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("new") => new_package(matches),
        Some("build") => {
            let _ = build_package();
            println!("Constraint system successfully built!")
        }
        // Some("contract") => create_smart_contract(),
        Some("prove") => prove(matches),
        Some("verify") => verify(matches),
        None => println!("No subcommand was used"),
        _ => unreachable!(), // Assuming you've listed all direct children above, this is unreachable
    }
}

fn prove(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("prove")
        .unwrap()
        .value_of("proof_name")
        .unwrap();

    prove_(proof_name);
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
    proof_path.set_extension("proof");

    let result = verify_(proof_name);
    println!("Proof verified : {}\n", result);
}

fn new_package(args: ArgMatches) {
    let package_name = args
        .subcommand_matches("new")
        .unwrap()
        .value_of("package_name")
        .unwrap();

    let mut package_dir = std::env::current_dir().unwrap();
    package_dir.push(Path::new(package_name));

    const SRC_DIR: &str = "src";
    const PROOFS_DIR: &str = "proofs";
    const CONTRACT_DIR: &str = "contract";

    create_directory(&package_dir.join(Path::new(SRC_DIR)));
    create_directory(&package_dir.join(Path::new(PROOFS_DIR)));
    create_directory(&package_dir.join(Path::new(CONTRACT_DIR)));

    const EXAMPLE: &'static str = "
        fn main(x : Witness, y : Witness) {
            constrain x != y;
        }
    ";

    const INPUT: &'static str = r#"
        x = "5"
        y = "10"
    "#;

    let src_dir = package_dir.join(Path::new(SRC_DIR));

    let _ = write_to_file(INPUT.as_bytes(), &src_dir.join(Path::new("input.toml")));
    let path = write_to_file(EXAMPLE.as_bytes(), &src_dir.join(Path::new("main.nr")));
    println!("Project successfully created! Binary located at {}", path);
}

fn build_package() {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, _) = Resolver::resolve_root_config(&curr_dir);
    driver.build();
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: usize = 1;

fn prove_(proof_name: &str) {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, backend) = Resolver::resolve_root_config(&curr_dir);
    let compiled_program = driver.into_compiled_program();

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

    let sorted_abi = compiled_program.abi.unwrap().sort_by_public_input();
    let param_names = sorted_abi.parameter_names();
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

                let old_value = solved_witness.insert(
                    Witness::new(param.to_owned(), index + WITNESS_OFFSET),
                    value.clone(),
                );
                assert!(old_value.is_none());

                index += 1
            }
        } else {
            let value = witness_map.get(param).expect(err_msg);

            let old_value = solved_witness.insert(
                Witness::new(param.to_owned(), index + WITNESS_OFFSET),
                value.clone(),
            );
            assert!(old_value.is_none());

            index += 1;
        }
    }

    Solver::solve(&mut solved_witness, compiled_program.circuit.clone());

    let proof = backend.prove_with_meta(
        compiled_program.circuit,
        solved_witness,
        compiled_program.num_witnesses,
        compiled_program.num_public_inputs,
    );

    let mut proof_path = std::path::PathBuf::new();
    proof_path.push("proofs");
    proof_path.push(proof_name);
    proof_path.set_extension("proof");

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path)
}

fn verify_(proof_name: &str) -> bool {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, backend) = Resolver::resolve_root_config(&curr_dir);
    let compiled_program = driver.into_compiled_program();

    let mut proof_path = curr_dir;
    proof_path.push(Path::new("proofs"));
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension("proof");

    let proof_hex: Vec<_> = std::fs::read(proof_path).unwrap();
    let proof = hex::decode(proof_hex).unwrap();

    backend.verify_from_cs(
        &proof,
        compiled_program.circuit,
        compiled_program.num_witnesses,
        compiled_program.num_public_inputs,
    )
}

fn create_directory(path: &std::path::Path) {
    if path.exists() {
        println!("This directory {} already exists", path.display());
        return;
    }
    std::fs::create_dir_all(path).unwrap();
}

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
