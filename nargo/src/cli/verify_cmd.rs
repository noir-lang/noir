use super::PROOFS_DIR;
use crate::resolver::Resolver;
use clap::ArgMatches;
use std::path::Path;

pub(crate) fn run(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("verify")
        .unwrap()
        .value_of("proof")
        .unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new(PROOFS_DIR));

    proof_path.push(Path::new(proof_name));
    proof_path.set_extension("proof");

    let result = verify(proof_name);
    println!("Proof verified : {}\n", result);
}

fn verify(proof_name: &str) -> bool {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, backend_ptr) = Resolver::resolve_root_config(&curr_dir);
    let compiled_program = driver.into_compiled_program(backend_ptr);

    let mut proof_path = curr_dir;
    proof_path.push(Path::new("proofs"));
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension("proof");

    // XXX: Instead of unwrap, return a PathNotValidError
    let proof_hex: Vec<_> = std::fs::read(proof_path).unwrap();
    // XXX: Instead of unwrap, return a ProofNotValidError
    let proof = hex::decode(proof_hex).unwrap();

    backend_ptr
        .backend()
        .verify_from_cs(&proof, compiled_program.circuit)
}
