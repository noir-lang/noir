//! Use the SSA Interpreter to execute a SSA after a certain pass.

use std::collections::BTreeMap;
use std::io::{self, Write};
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
            &parsed_files,
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
        let mut last_ssa_printed: Option<String> = None;
        let mut previous_snapshot: Option<PassSnapshot> = None;

        is_ok &= print_and_interpret_ssa(
            ssa_options,
            &args.ssa_pass,
            &mut ssa,
            "Initial SSA",
            &ssa_args,
            &ssa_return,
            interpreter_options,
            file_manager,
            &mut last_ssa_printed,
            &mut previous_snapshot,
        )?;

        // Run SSA passes in the pipeline and interpret the ones we are interested in.
        for (i, ssa_pass) in ssa_passes.iter().enumerate() {
            let last_step = i == ssa_passes.len() - 1;
            let last_step_msg = if last_step { " (last step)" } else { "" };
            let msg = format!("{} (step {}){last_step_msg}", ssa_pass.msg(), i + 1);

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
                &mut last_ssa_printed,
                &mut previous_snapshot,
            )?;
        }
    }
    if is_ok {
        Ok(())
    } else {
        Err(CliError::Generic(
            "The interpreter encountered an error or disagreement on one or more passes.".into(),
        ))
    }
}

struct TeeWriter<A, B> {
    left: A,
    right: B,
}

impl<A, B> TeeWriter<A, B> {
    fn new(left: A, right: B) -> Self {
        Self { left, right }
    }
}

impl<A: Write, B: Write> Write for TeeWriter<A, B> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.left.write_all(buf)?;
        self.right.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.left.flush()?;
        self.right.flush()
    }
}

/// The observation recorded for an interpreted SSA pass.
///
/// `result` is `Some(..)` when interpretation succeeded and `None` when it returned an error.
#[derive(Clone, Debug, PartialEq)]
struct PassSnapshot {
    pass_name: String,
    print_output: String,
    result: Option<Vec<Value>>,
}

impl PassSnapshot {
    fn result_string(&self) -> String {
        match &self.result {
            Some(values) => {
                let values = vecmap(values, ToString::to_string).join(", ");
                format!("Ok({values})")
            }
            None => "Err".to_string(),
        }
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

/// Prints the SSA (if it needs to be printed) and returns whether the SSA needs
/// to be interpreted afterwards (it doesn't need to if the SSA passes are being
/// printed and the user asked to skip SSA passes that don't produce changes).
fn print_ssa(
    options: &SsaEvaluatorOptions,
    ssa: &mut Ssa,
    msg: &str,
    files: Option<&FileManager>,
    last_ssa_printed: &mut Option<String>,
) -> bool {
    let print_ssa_pass = options.ssa_logging.matches(msg);

    // Always normalize if we are going to print at least one of the passes
    if !matches!(options.ssa_logging, SsaLogging::None) {
        ssa.normalize_ids();
    }

    if print_ssa_pass {
        let printed_ssa = format!("{}", ssa.print_with(files));
        let skip_print = options.ssa_logging_hide_unchanged
            && last_ssa_printed
                .as_ref()
                .is_some_and(|last_ssa_printed| last_ssa_printed == &printed_ssa);

        if !skip_print {
            println!("After {msg}:\n{printed_ssa}");
        }

        if options.ssa_logging_hide_unchanged {
            *last_ssa_printed = Some(printed_ssa);
        }

        !skip_print
    } else {
        true
    }
}

/// Interpret the SSA if it's part of the selected passes.
///
/// The return value is:
/// * `Ok(Some(_))` if the pass was interpreted.
/// * `Ok(None)` if the pass was skipped because it was not selected.
/// * `Err(_)` if the observed result did not match `return_value`.
fn interpret_ssa(
    passes_to_interpret: &[String],
    ssa: &Ssa,
    msg: &str,
    args: &[Value],
    return_value: &Option<Vec<Value>>,
    options: InterpreterOptions,
) -> Result<Option<PassSnapshot>, CliError> {
    if passes_to_interpret.is_empty() || msg_matches(passes_to_interpret, msg) {
        // We need to give a fresh copy of arrays each time, because the shared structures are modified.
        let args = Value::snapshot_args(args);
        let mut print_output = Vec::new();
        let result = {
            let output = TeeWriter::new(io::stdout(), &mut print_output);
            ssa.interpret_with_options(args, options, output)
        };
        match &result {
            Ok(value) => {
                let value_string = vecmap(value, ToString::to_string).join(", ");
                println!("--- Interpreter result after {msg}:\nOk({value_string})\n---");
            }
            Err(err) => {
                println!("--- Interpreter result after {msg}:\nErr({err})\n---");
            }
        }
        if let Some(return_value) = return_value {
            match &result {
                Ok(actual_result) if actual_result == return_value => {}
                Ok(actual_result) => {
                    let actual_result_string =
                        vecmap(actual_result, ToString::to_string).join(", ");
                    let return_value_string = vecmap(return_value, ToString::to_string).join(", ");
                    let error = format!(
                        "Error: interpreter produced an unexpected result.\nExpected result: {return_value_string}\nActual result:   {actual_result_string}"
                    );
                    return Err(CliError::Generic(error));
                }
                Err(err) => {
                    let return_value_string = vecmap(return_value, ToString::to_string).join(", ");
                    let error = format!(
                        "Error: interpreter produced an unexpected error.\nExpected result: {return_value_string}\nActual result:   Err({err})"
                    );
                    return Err(CliError::Generic(error));
                }
            }
        }
        let result = result.ok();
        Ok(Some(PassSnapshot {
            pass_name: msg.to_string(),
            print_output: String::from_utf8(print_output).expect("from_utf8 of interpreter print"),
            result,
        }))
    } else {
        Ok(None)
    }
}

/// Compares the current interpreted snapshot with the previous one.
///
/// Returns a discrepancy message when the observation changed and always advances the baseline
/// to the current pass.
fn compare_and_update_snapshot(
    previous_snapshot: &mut Option<PassSnapshot>,
    current_snapshot: PassSnapshot,
    compare_results: bool,
) -> Option<String> {
    let discrepancy = previous_snapshot.as_ref().and_then(|previous_snapshot| {
        let output_changed = previous_snapshot.print_output != current_snapshot.print_output;
        let result_changed = compare_results && previous_snapshot.result != current_snapshot.result;

        if !output_changed && !result_changed {
            return None;
        }

        let mut message = format!(
            "Error: interpreter observation changed between SSA passes.\nPrevious checked pass: {}\nCurrent checked pass: {}",
            previous_snapshot.pass_name, current_snapshot.pass_name
        );
        if output_changed {
            message.push_str(&format!(
                "\nPrinted output changed.\nPrevious output:\n{}\nCurrent output:\n{}",
                previous_snapshot.print_output, current_snapshot.print_output
            ));
        }
        if result_changed {
            message.push_str(&format!(
                "\nResult changed.\nPrevious result: {}\nCurrent result: {}",
                previous_snapshot.result_string(),
                current_snapshot.result_string(),
            ));
        }
        Some(message)
    });

    *previous_snapshot = Some(current_snapshot);

    discrepancy
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
    last_ssa_printed: &mut Option<String>,
    previous_snapshot: &mut Option<PassSnapshot>,
) -> Result<bool, CliError> {
    let must_interpret = print_ssa(options, ssa, msg, fm, last_ssa_printed);
    if must_interpret {
        if let Some(current_snapshot) =
            interpret_ssa(passes_to_interpret, ssa, msg, args, return_value, interpreter_options)?
        {
            let interpreter_failed = current_snapshot.result.is_none();
            let discrepancy = compare_and_update_snapshot(
                previous_snapshot,
                current_snapshot,
                return_value.is_none(),
            );
            let disagreed = discrepancy.is_some();
            if let Some(message) = &discrepancy {
                println!("{message}");
            }
            Ok(!interpreter_failed && !disagreed)
        } else {
            Ok(true)
        }
    } else {
        Ok(true)
    }
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
        Value::ArrayOrVector(array_value) => {
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{PassSnapshot, TeeWriter, compare_and_update_snapshot, interpret_ssa};
    use crate::errors::CliError;
    use noirc_evaluator::ssa::interpreter::InterpreterOptions;
    use noirc_evaluator::ssa::interpreter::value::Value;
    use noirc_evaluator::ssa::ssa_gen::Ssa;

    fn ok_result(value: u32) -> Option<Vec<Value>> {
        Some(vec![Value::u32(value)])
    }

    fn snapshot(pass_name: &str, result: Option<Vec<Value>>, print_output: &str) -> PassSnapshot {
        PassSnapshot {
            pass_name: pass_name.to_string(),
            print_output: print_output.to_string(),
            result,
        }
    }

    #[test]
    fn tee_writer_writes_to_both_outputs() {
        let mut left = Vec::new();
        let mut right = Vec::new();

        {
            let mut writer = TeeWriter::new(&mut left, &mut right);
            writer.write_all(b"hello").unwrap();
            writer.write_all(b" world").unwrap();
            writer.flush().unwrap();
        }

        assert_eq!(left, b"hello world");
        assert_eq!(right, b"hello world");
    }

    #[test]
    fn matching_observation_establishes_and_updates_the_baseline() {
        let mut previous_snapshot = None;
        let first_snapshot = snapshot("Initial SSA", ok_result(1), "same");
        let second_snapshot = snapshot("Mem2Reg Simple (step 6)", ok_result(1), "same");

        assert_eq!(compare_and_update_snapshot(&mut previous_snapshot, first_snapshot, true), None);

        assert_eq!(
            compare_and_update_snapshot(&mut previous_snapshot, second_snapshot.clone(), true),
            None
        );

        let previous_snapshot = previous_snapshot.expect("expected updated baseline");
        assert_eq!(previous_snapshot, second_snapshot);
    }

    #[test]
    fn different_printed_output_reports_a_discrepancy_and_advances_the_snapshot() {
        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", ok_result(1), "0x06"));
        let current_snapshot = snapshot("Mem2Reg Simple (step 6)", ok_result(1), "0x05");

        let message =
            compare_and_update_snapshot(&mut previous_snapshot, current_snapshot.clone(), true)
                .expect("expected mismatch message");

        assert!(message.contains("Printed output changed."));
        assert!(message.contains("Removing Unreachable Functions (step 5)"));
        assert!(message.contains("Mem2Reg Simple (step 6)"));
        assert!(message.contains("0x06"));
        assert!(message.contains("0x05"));

        let previous_snapshot = previous_snapshot.expect("expected updated baseline");
        assert_eq!(previous_snapshot, current_snapshot);
    }

    #[test]
    fn different_printed_output_on_errors_is_still_reported() {
        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", None, "0x06"));
        let current_snapshot = snapshot("Mem2Reg Simple (step 6)", None, "0x05");

        let message =
            compare_and_update_snapshot(&mut previous_snapshot, current_snapshot.clone(), true)
                .expect("expected mismatch message");

        assert!(message.contains("Printed output changed."));
        assert!(!message.contains("Result changed."));

        let previous_snapshot = previous_snapshot.expect("expected updated baseline");
        assert_eq!(previous_snapshot, current_snapshot);
    }

    #[test]
    fn result_mismatch_is_reported_when_expected_return_is_absent() {
        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", ok_result(1), "same"));
        let changed_result = snapshot("Mem2Reg Simple (step 6)", ok_result(2), "same");

        let message = compare_and_update_snapshot(&mut previous_snapshot, changed_result, true)
            .expect("expected mismatch message");

        assert!(message.contains("Result changed."));
        assert!(message.contains("Ok(u32 1)"));
        assert!(message.contains("Ok(u32 2)"));

        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", ok_result(1), "same"));
        let err_result = snapshot("Mem2Reg Simple (step 6)", None, "same");

        let message = compare_and_update_snapshot(&mut previous_snapshot, err_result, true)
            .expect("expected mismatch message");

        assert!(message.contains("Result changed."));
        assert!(message.contains("Ok(u32 1)"));
        assert!(message.contains("Current result: Err"));

        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", None, "same"));
        assert_eq!(
            compare_and_update_snapshot(
                &mut previous_snapshot,
                snapshot("Mem2Reg Simple (step 6)", None, "same"),
                true,
            ),
            None
        );
    }

    #[test]
    fn return_value_comparison_can_be_disabled_for_pass_snapshots() {
        let mut previous_snapshot =
            Some(snapshot("Removing Unreachable Functions (step 5)", ok_result(1), "same"));
        let current_snapshot = snapshot("Mem2Reg Simple (step 6)", ok_result(2), "same");

        assert_eq!(
            compare_and_update_snapshot(&mut previous_snapshot, current_snapshot, false),
            None
        );
    }

    #[test]
    fn expected_return_mismatch_returns_a_cli_error() {
        let ssa = Ssa::from_str(
            r#"
                acir(inline) fn main f0 {
                  b0():
                    return u32 2
                }
            "#,
        )
        .unwrap();

        let error = interpret_ssa(
            &[],
            &ssa,
            "Initial SSA",
            &[],
            &Some(vec![Value::u32(1)]),
            InterpreterOptions::default(),
        )
        .expect_err("expected return mismatch");

        let CliError::Generic(message) = error else {
            panic!("expected generic cli error");
        };

        assert!(message.contains("Expected result: u32 1"));
        assert!(message.contains("Actual result:   u32 2"));
    }

    #[test]
    fn unexpected_interpreter_error_with_expected_return_is_a_cli_error() {
        let ssa = Ssa::from_str(
            r#"
                acir(inline) predicate_pure fn main f0 {
                b0():
                  call f1(u1 0)
                  return
                }
                brillig(inline) predicate_pure fn func_2 f1 {
                  b0(v0: u1):
                    jmp b1()
                  b1():
                    jmpif v0 then: b2(), else: b3()
                  b2():
                    return
                  b3():
                    jmp b1()
                }
            "#,
        )
        .unwrap();

        let error = interpret_ssa(
            &[],
            &ssa,
            "Initial SSA",
            &[],
            &Some(vec![]),
            InterpreterOptions { step_limit: Some(100), ..Default::default() },
        )
        .expect_err("expected interpreter error");

        let CliError::Generic(message) = error else {
            panic!("expected generic cli error");
        };

        assert!(message.contains("Error: interpreter produced an unexpected error."));
        assert!(message.contains("Actual result:   Err("));
    }
}
