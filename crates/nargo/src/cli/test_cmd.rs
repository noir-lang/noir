use std::{collections::BTreeMap, io::Write};

use acvm::{PartialWitnessGenerator, ProofSystemCompiler};
use clap::ArgMatches;
use noirc_driver::Driver;
use noirc_frontend::node_interner::FuncId;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    for test_function in test_functions {
        let test_name = driver.function_name(test_function);
        write!(writer, "Testing {test_name}... ").expect("Failed to write to stdout");
        writer.flush().ok();

        match run_test(test_name, test_function, &driver, allow_warnings) {
            Ok(_) => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
                writeln!(writer, "ok").ok();
            }
            // Assume an error was already printed to stdout
            Err(_) => failing += 1,
        }
        writer.reset().ok();
    }

    if failing == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
        writeln!(writer, "All tests passed").ok();
    } else {
        let plural = if failing == 1 { "" } else { "s" };
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
        writeln!(writer, "{failing} test{plural} failed").ok();
        std::process::exit(1);
    }

    writer.reset().ok();
    Ok(())
}

fn run_test(
    test_name: &str,
    main: FuncId,
    driver: &Driver,
    allow_warnings: bool,
) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;
    let language = backend.np_language();

    let program = driver
        .compile_no_check(language, false, allow_warnings, Some(main))
        .map_err(|_| CliError::Generic(format!("Test '{test_name}' failed to compile")))?;

    let mut solved_witness = BTreeMap::new();

    // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
    // otherwise constraints involving these expressions will not error.
    if let Err(error) = backend.solve(&mut solved_witness, program.circuit.opcodes) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
        writeln!(writer, "failed").ok();
        writer.reset().ok();
        return Err(error.into());
    }
    Ok(())
}
