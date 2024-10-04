use std::borrow::Borrow;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use nargo::constants::PROVER_INPUT_FILE;
use nargo::ops::{compile_program, report_errors};
use nargo::package::Package;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noir_tracer::tracer_glue::store_trace;
use noirc_abi::input_parser::Format;
use noirc_abi::InputMap;
use noirc_artifacts::debug::DebugArtifact;
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_evaluator::ssa::plonky2_gen::asm_writer::DebugTraceList;
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::type_check::generics::Generic;

use super::compile_cmd::get_target_width;
use super::debug_cmd::compile_bin_package_for_debugging;
use super::fs::inputs::read_inputs_from_file;
use crate::errors::CliError;

use runtime_tracing::Tracer;

use super::NargoConfig;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct TraceCommand {
    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Directory where to store trace.json
    #[clap(long, short)]
    trace_dir: String,

    /// Insert plonky2 information to the trace file
    #[arg(long)]
    trace_plonky2: bool,
}

fn generate_plonky2_debug_trace_list(
    args: TraceCommand,
    config: NargoConfig,
) -> Result<DebugTraceList, CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    //let default_selection =
    //    if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(std::path::Path::new(""));
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let compilation_result = compile_program(
            &workspace_file_manager,
            &parsed_files,
            &workspace,
            package,
            &args.compile_options,
            None,
            true,
            true,
        );

        let dtl = if let Ok((compiled_program, _)) = &compilation_result {
            if let Some(plonky2_circuit) = &compiled_program.plonky2_circuit {
                plonky2_circuit.debug_trace_list.clone()
            } else {
                None
            }
        } else {
            None
        };

        let _compiled_program = report_errors(
            compilation_result,
            &workspace_file_manager,
            args.compile_options.deny_warnings,
            args.compile_options.silence_warnings,
        )?;

        return Ok(dtl.unwrap());
    }

    Err(CliError::Generic(
        "No matching binary packages found in workspace. Only binary packages can be debugged."
            .to_string(),
    ))
}

pub(crate) fn run(args: TraceCommand, config: NargoConfig) -> Result<(), CliError> {
    let debug_trace_list = if args.trace_plonky2 {
        Some(generate_plonky2_debug_trace_list(args.clone(), config.clone())?)
    } else {
        None
    };

    let acir_mode = false;
    let skip_instrumentation = false;

    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let Some(package) = workspace.into_iter().find(|p| p.is_binary()) else {
        println!(
            "No matching binary packages found in workspace. Only binary packages can be debugged."
        );
        return Ok(());
    };

    let compiled_program = compile_bin_package_for_debugging(
        &workspace,
        package,
        acir_mode,
        skip_instrumentation,
        args.compile_options.clone(),
    )?;

    trace_program_and_decode(
        compiled_program,
        package,
        &args.prover_name,
        &args.trace_dir,
        debug_trace_list,
    )
}

fn trace_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
    trace_dir: &str,
    debug_trace_list: Option<DebugTraceList>,
) -> Result<(), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;

    trace_program(&program, &package.name, &inputs_map, trace_dir, debug_trace_list)
}

pub(crate) fn trace_program(
    compiled_program: &CompiledProgram,
    crate_name: &CrateName,
    inputs_map: &InputMap,
    trace_dir: &str,
    debug_trace_list: Option<DebugTraceList>,
) -> Result<(), CliError> {
    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let debug_artifact = DebugArtifact {
        debug_symbols: compiled_program.debug.clone(),
        file_map: compiled_program.file_map.clone(),
    };

    let crate_name_string: String = crate_name.into();
    let mut tracer = Tracer::new(crate_name_string.as_str(), &vec![]);

    match noir_tracer::trace_circuit(
        &Bn254BlackBoxSolver,
        &compiled_program.program.functions,
        &debug_artifact,
        initial_witness,
        &compiled_program.program.unconstrained_functions,
        &mut tracer,
    ) {
        Err(error) => return Err(CliError::from(error)),
        Ok(()) => (),
    };

    store_trace(tracer, trace_dir);

    Ok(())
}
