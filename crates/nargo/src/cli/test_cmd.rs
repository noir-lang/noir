use std::io::Write;

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_driver::{CompiledProgram, Driver};
use noirc_frontend::node_interner::FuncId;

use crate::{errors::CliError, resolver::Resolver};

use super::add_std_lib;

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
    let mut failing = 0;

    for test_function in test_functions {
        let test_name = driver.function_name(test_function);
        print!("Testing {test_name}... ");
        std::io::stdout().flush().ok();

        match prove(test_name, test_function, &driver, allow_warnings) {
            Ok(_) => println!("ok"),
            Err(_) => {
                // An error likely was printed in the meantime, so this is the start of a newline.
                // Add an extra newline as well to help separate failing tests.
                println!("{test_name} failed\n");
                failing += 1;
            }
        }
    }

    if failing == 0 {
        println!("All tests passed");
    } else {
        let plural = if failing == 1 { "" } else { "s" };
        println!("{failing} test{plural} failed");
        std::process::exit(1);
    }

    Ok(())
}

fn prove(
    test_name: &str,
    main: FuncId,
    driver: &Driver,
    allow_warnings: bool,
) -> Result<CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let language = backend.np_language();

    driver
        .compile_no_check(language, false, allow_warnings, Some(main))
        .map_err(|_| CliError::Generic(format!("Test '{test_name}' failed to compile")))
}
