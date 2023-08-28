use std::io::Write;

use acvm::{acir::native_types::WitnessMap, Backend};
use clap::Args;
use nargo::{ops::execute_circuit, package::Package, prepare_package};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{compile_no_check, CompileOptions};
use noirc_frontend::{
    graph::CrateName,
    hir::{def_map::TestFunction, Context, FunctionNameMatch},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::{compile_cmd::optimize_circuit, NargoConfig};

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
pub(crate) struct TestCommand {
    /// If given, only tests with names containing this string will be run
    test_name: Option<String>,

    /// Display output of `println` statements
    #[arg(long)]
    show_output: bool,

    /// Only run tests that match exactly
    #[clap(long)]
    exact: bool,

    /// The name of the package to test
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Test all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: TestCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;

    let pattern = match &args.test_name {
        Some(name) => {
            if args.exact {
                FunctionNameMatch::Exact(name)
            } else {
                FunctionNameMatch::Contains(name)
            }
        }
        None => FunctionNameMatch::Anything,
    };

    for package in &workspace {
        run_tests(backend, package, pattern, args.show_output, &args.compile_options)?;
    }

    Ok(())
}

fn run_tests<B: Backend>(
    backend: &B,
    package: &Package,
    test_name: FunctionNameMatch,
    show_output: bool,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = prepare_package(package);
    check_crate_and_report_errors(&mut context, crate_id, compile_options.deny_warnings)?;

    let test_functions = context.get_all_test_functions_in_crate_matching(&crate_id, test_name);

    println!("[{}] Running {} test functions", package.name, test_functions.len());
    let mut failing = 0;

    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    for (test_name, test_function) in test_functions {
        write!(writer, "[{}] Testing {test_name}... ", package.name)
            .expect("Failed to write to stdout");
        writer.flush().expect("Failed to flush writer");

        match run_test(backend, &test_name, test_function, &context, show_output, compile_options) {
            Ok(_) => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .expect("Failed to set color");
                writeln!(writer, "ok").expect("Failed to write to stdout");
            }
            // Assume an error was already printed to stdout
            Err(_) => failing += 1,
        }
        writer.reset().expect("Failed to reset writer");
    }

    if failing == 0 {
        write!(writer, "[{}] ", package.name).expect("Failed to write to stdout");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).expect("Failed to set color");
        writeln!(writer, "All tests passed").expect("Failed to write to stdout");
    } else {
        let plural = if failing == 1 { "" } else { "s" };
        return Err(CliError::Generic(format!("[{}] {failing} test{plural} failed", package.name)));
    }

    writer.reset().expect("Failed to reset writer");
    Ok(())
}

fn run_test<B: Backend>(
    backend: &B,
    test_name: &str,
    test_function: TestFunction,
    context: &Context,
    show_output: bool,
    config: &CompileOptions,
) -> Result<(), CliError<B>> {
    let report_error = |err| {
        noirc_errors::reporter::report_all(&context.file_manager, &[err], config.deny_warnings);
        Err(CliError::Generic(format!("Test '{test_name}' failed to compile")))
    };
    let write_error = |err| {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
        writeln!(writer, "failed").ok();
        writer.reset().ok();
        Err(err)
    };

    let program = compile_no_check(context, config, test_function.get_id());
    match program {
        Ok(mut program) => {
            // Note: We don't need to use the optimized ACIR here
            program.circuit = optimize_circuit(backend, program.circuit).unwrap().0;

            // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
            // otherwise constraints involving these expressions will not error.
            let circuit_execution =
                execute_circuit(backend, program.circuit, WitnessMap::new(), show_output);

            if test_function.should_fail() {
                match circuit_execution {
                    Ok(_) => {
                        write_error(CliError::Generic(format!("Test '{test_name}' should fail")))
                    }
                    Err(_) => Ok(()),
                }
            } else {
                match circuit_execution {
                    Ok(_) => Ok(()),
                    Err(error) => write_error(error.into()),
                }
            }
        }
        // Test function failed to compile
        //
        // Note: This could be because the compiler was able to deduce
        // that a constraint was never satisfiable.
        // An example of this is the program `assert(false)`
        //  In that case, we check if the test function should fail, and if so, we return Ok.
        Err(err) => {
            // The test has failed compilation, but it should never fail. Report error.
            if !test_function.should_fail() {
                return report_error(err);
            }

            // The test has failed compilation, check if it is because the program is never satisfiable.
            // If it is never satisfiable, then this is the expected behavior.
            let program_is_never_satisfiable = err.diagnostic.message.contains("Failed constraint");
            if program_is_never_satisfiable {
                return Ok(());
            }

            // The test has failed compilation, but its a compilation error. Report error
            report_error(err)
        }
    }
}
