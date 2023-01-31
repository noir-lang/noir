use std::collections::BTreeMap;

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_driver::{CompiledProgram, Driver};

use crate::{errors::CliError, resolver::Resolver};

use super::{add_std_lib, prove_cmd::parse_and_solve_witness, verify_cmd::verify_proof};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("test").unwrap();
    let test_name = args.value_of("test_name").unwrap_or("");
    let allow_warnings = args.is_present("allow-warnings");
    run_tests(test_name, allow_warnings)
}

fn run_tests(test_name: &str, allow_warnings: bool) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let package_dir = std::env::current_dir().unwrap();
    let mut driver = Resolver::resolve_root_config(&package_dir, backend.np_language())?;
    add_std_lib(&mut driver);

    if driver.check_crate(allow_warnings).is_err() {
        std::process::exit(1);
    }

    let test_functions = driver.get_all_test_functions_in_crate_matching(test_name);
    println!("Running {} test functions...", test_functions.len());

    for test_function in test_functions {
        let test_name = driver.function_name(test_function);
        print!("Testing {test_name}... ");

        let (program, proof) = prove(test_name, &driver, allow_warnings)?;
        verify(program, proof)?;

        println!("ok");
    }

    Ok(())
}

type Proof = Vec<u8>;

fn prove(
    test_name: &str,
    driver: &Driver,
    allow_warnings: bool,
) -> Result<(CompiledProgram, Proof), CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut program_dir = std::env::current_dir().unwrap();
    program_dir.push("src");

    let program =
        driver.compile_no_check(backend.np_language(), false, allow_warnings).map_err(|_| {
            // TODO: Add test name
            CliError::Generic(format!("Test '{test_name}' failed to compile"))
        })?;

    let solved_witness = parse_and_solve_witness(program_dir, &program)?;

    let proof = backend.prove_with_meta(program.circuit.clone(), solved_witness);
    Ok((program, proof))
}

fn verify(compiled_program: CompiledProgram, proof: Proof) -> Result<bool, CliError> {
    verify_proof(compiled_program, BTreeMap::new(), proof)
}
