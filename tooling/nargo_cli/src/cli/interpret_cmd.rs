//! Use the SSA Interpreter to execute a SSA after a certain pass.

use std::collections::BTreeMap;
use std::path::PathBuf;

use fm::{FileId, FileManager};
use iter_extended::vecmap;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::ops::report_errors;
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noirc_abi::Abi;
use noirc_driver::{CompilationResult, CompileOptions, gen_abi};

use clap::Args;
use noirc_errors::CustomDiagnostic;
use noirc_evaluator::ssa::interpreter::InterpreterOptions;
use noirc_evaluator::ssa::interpreter::value::{NumericValue, Value};
use noirc_evaluator::ssa::ir::types::{NumericType, Type};
use noirc_evaluator::ssa::ssa_gen::{Ssa, generate_ssa};
use noirc_evaluator::ssa::{SsaEvaluatorOptions, SsaLogging, primary_passes};
use noirc_frontend::debug::DebugInstrumenter;
use noirc_frontend::hir::ParsedFiles;
use noirc_frontend::monomorphization::ast::Program;
use noirc_frontend::monomorphization::{monomorphize, monomorphize_debug};

use crate::errors::CliError;

use super::compile_cmd::parse_workspace;
use super::{LockType, PackageOptions, WorkspaceCommand};

/// Compile the program and interpret the SSA after each pass,
/// printing the results to the console.
#[derive(Debug, Clone, Args)]
pub(super) struct InterpretCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    pub(super) compile_options: CompileOptions,

    /// The name of the TOML file which contains the ABI encoded inputs.
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the SSA passes we want to interpret the results at.
    ///
    /// When nothing is specified, the SSA is interpreted after all passes.
    #[clap(long)]
    ssa_pass: Vec<String>,

    /// If true, the interpreter will trace its execution.
    #[clap(long)]
    trace: bool,

    /// Optional limit for the interpreter.
    #[clap(long)]
    step_limit: Option<usize>,
}

impl WorkspaceCommand for InterpretCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Does not write any artifacts.
        LockType::Shared
    }
}

pub(crate) fn run(args: InterpretCommand, workspace: Workspace) -> Result<(), CliError> {
    let (file_manager, parsed_files) = parse_workspace(&workspace, None);
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

    let opts = args.compile_options.as_ssa_options(PathBuf::new());
    let ssa_passes = primary_passes(&opts);
    let mut is_ok = true;

    for package in binary_packages {
        let ssa_options =
            &args.compile_options.as_ssa_options(workspace.package_build_path(package));

        // Compile into monomorphized AST
        let program_result = compile_into_program(
            &file_manager,
            &parsed_files,
            &workspace,
            package,
            &args.compile_options,
        );

        // Report warnings and get the AST, or exit if the compilation failed.
        let (program, abi) = report_errors(
            program_result,
            &file_manager,
            args.compile_options.deny_warnings,
            args.compile_options.silence_warnings,
        )?;

        // Parse the inputs and convert them to what the SSA interpreter expects.
        let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");
        let (prover_input, return_value) =
            noir_artifact_cli::fs::inputs::read_inputs_from_file(&prover_file, &abi)?;

        // We need to give a fresh copy of arrays each time, because the shared structures are modified.
        let ssa_args = noir_ast_fuzzer::input_values_to_ssa(&abi, &prover_input);

        let ssa_return =
            if let (Some(return_type), Some(return_value)) = (&abi.return_type, return_value) {
                Some(noir_ast_fuzzer::input_value_to_ssa(&return_type.abi_type, &return_value))
            } else {
                None
            };

        // Generate the initial SSA.
        let mut ssa = generate_ssa(program)
            .map_err(|e| CliError::Generic(format!("failed to generate SSA: {e}")))?;

        // If the main function returns `return_data`, the values are returned in a flattened array.
        // So, we change the expected return value by flattening it as well.
        // Ideally we'd have the interpreter return the data in the correct shape. However, doing
        // that would be replicating some logic which is unrelated to SSA. For the purpose of SSA
        // correctness, it's enough if we make sure the flattened values match.
        let ssa_return = ssa_return.map(|ssa_return| {
            let main_function = &ssa.functions[&ssa.main_id];
            if main_function.view().has_data_bus_return_data() {
                let values = flatten_databus_values(ssa_return);
                vec![Value::array(values, vec![Type::Numeric(NumericType::NativeField)])]
            } else {
                ssa_return
            }
        });

        let interpreter_options = InterpreterOptions {
            trace: args.trace,
            step_limit: args.step_limit,
            ..Default::default()
        };
        let file_manager =
            if args.compile_options.with_ssa_locations { Some(&file_manager) } else { None };

        is_ok &= print_and_interpret_ssa(
            ssa_options,
            &args.ssa_pass,
            &mut ssa,
            "Initial SSA",
            &ssa_args,
            &ssa_return,
            interpreter_options,
            file_manager,
        )?;

        // Run SSA passes in the pipeline and interpret the ones we are interested in.
        for (i, ssa_pass) in ssa_passes.iter().enumerate() {
            let msg = format!("{} (step {})", ssa_pass.msg(), i + 1);

            if msg_matches(&args.compile_options.skip_ssa_pass, &msg) {
                continue;
            }

            ssa = ssa_pass
                .run(ssa)
                .map_err(|e| CliError::Generic(format!("failed to run SSA pass {msg}: {e}")))?;

            is_ok &= print_and_interpret_ssa(
                ssa_options,
                &args.ssa_pass,
                &mut ssa,
                &msg,
                &ssa_args,
                &ssa_return,
                interpreter_options,
                file_manager,
            )?;
        }
    }
    if is_ok {
        Ok(())
    } else {
        Err(CliError::Generic("The interpreter encountered an error on one or more passes.".into()))
    }
}

/// Compile the source code into the monomorphized AST, which is one step before SSA passes.
///
/// This isn't exposed through the `nargo` library operations at the moment, so this is a
/// bit of copy pasting from the functions that normally produce an artifact.
fn compile_into_program(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    options: &CompileOptions,
) -> CompilationResult<(Program, Abi)> {
    let (mut context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
    context.debug_instrumenter = DebugInstrumenter::default();
    context.package_build_path = workspace.package_build_path(package);
    noirc_driver::link_to_debug_crate(&mut context, crate_id);
    let (_, warnings) = noirc_driver::check_crate(&mut context, crate_id, options)?;

    let main_id = context.get_main_function(&crate_id).ok_or_else(|| {
        let err = CustomDiagnostic::from_message(
            "cannot compile crate into a program as it does not contain a `main` function",
            FileId::default(),
        );
        vec![err]
    })?;

    let force_unconstrained = options.force_brillig || options.minimal_ssa;

    let monomorphize_result = if options.instrument_debug {
        monomorphize_debug(
            main_id,
            &mut context.def_interner,
            &context.debug_instrumenter,
            force_unconstrained,
        )
    } else {
        monomorphize(main_id, &mut context.def_interner, force_unconstrained)
    };

    let program = monomorphize_result.map_err(|error| vec![CustomDiagnostic::from(error)])?;

    if options.show_monomorphized {
        println!("{program}");
    }

    let error_types = BTreeMap::default();
    let abi = gen_abi(&context, &main_id, program.return_visibility(), error_types);

    Ok(((program, abi), warnings))
}

fn msg_matches(patterns: &[String], msg: &str) -> bool {
    let msg = msg.to_lowercase();
    patterns.iter().any(|p| msg.contains(&p.to_lowercase()))
}

fn print_ssa(options: &SsaEvaluatorOptions, ssa: &mut Ssa, msg: &str, fm: Option<&FileManager>) {
    let print = match options.ssa_logging {
        SsaLogging::All => true,
        SsaLogging::None => false,
        SsaLogging::Contains(ref ps) => msg_matches(ps, msg),
    };
    if print {
        ssa.normalize_ids();
        println!("After {msg}:\n{}", ssa.print_with(fm));
    }
}

/// Interpret the SSA if it's part of the selected passes.
///
/// The return value is:
/// * `Ok(true)` if the interpretation was successful, or it was skipped.
/// * `Ok(false)` if the interpreter returned an error, but we didn't have any expectation.
/// * `Err(_)` if the returned result did not match the expectation.
fn interpret_ssa(
    passes_to_interpret: &[String],
    ssa: &Ssa,
    msg: &str,
    args: &[Value],
    return_value: &Option<Vec<Value>>,
    options: InterpreterOptions,
) -> Result<bool, CliError> {
    if passes_to_interpret.is_empty() || msg_matches(passes_to_interpret, msg) {
        // We need to give a fresh copy of arrays each time, because the shared structures are modified.
        let args = Value::snapshot_args(args);
        let result = ssa.interpret_with_options(args, options, std::io::stdout());
        match &result {
            Ok(value) => {
                let value_as_string = vecmap(value, ToString::to_string).join(", ");
                println!("--- Interpreter result after {msg}:\nOk({value_as_string})\n---");
            }
            Err(err) => {
                println!("--- Interpreter result after {msg}:\nErr({err})\n---");
            }
        }
        let is_ok = result.is_ok();
        if let Some(return_value) = return_value {
            let result = result.expect("Expected a non-error result");
            if &result != return_value {
                let result_as_string = vecmap(&result, ToString::to_string).join(", ");
                let return_value_as_string = vecmap(return_value, ToString::to_string).join(", ");
                let error = format!(
                    "Error: interpreter produced an unexpected result.\nExpected result: {return_value_as_string}\nActual result:   {result_as_string}"
                );
                return Err(CliError::Generic(error));
            }
        }
        Ok(is_ok)
    } else {
        Ok(true)
    }
}

#[allow(clippy::too_many_arguments)]
fn print_and_interpret_ssa(
    options: &SsaEvaluatorOptions,
    passes_to_interpret: &[String],
    ssa: &mut Ssa,
    msg: &str,
    args: &[Value],
    return_value: &Option<Vec<Value>>,
    interpreter_options: InterpreterOptions,
    fm: Option<&FileManager>,
) -> Result<bool, CliError> {
    print_ssa(options, ssa, msg, fm);
    interpret_ssa(passes_to_interpret, ssa, msg, args, return_value, interpreter_options)
}

fn flatten_databus_values(values: Vec<Value>) -> Vec<Value> {
    let mut flattened_values = Vec::new();
    for value in values {
        flatten_databus_value(value, &mut flattened_values);
    }
    flattened_values
}

fn flatten_databus_value(value: Value, flattened_values: &mut Vec<Value>) {
    match value {
        Value::ArrayOrSlice(array_value) => {
            for value in array_value.elements.borrow().iter() {
                flatten_databus_value(value.clone(), flattened_values);
            }
        }
        Value::Numeric(value) => {
            flattened_values.push(Value::Numeric(NumericValue::Field(value.convert_to_field())));
        }
        Value::Reference(..)
        | Value::Function(..)
        | Value::Intrinsic(..)
        | Value::ForeignFunction(..) => flattened_values.push(value),
    }
}
