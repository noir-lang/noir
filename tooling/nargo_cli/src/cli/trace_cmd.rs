use std::str::FromStr;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use nargo::ops::debug::compile_options_for_debugging;
use nargo::package::Package;
use nargo::{constants::PROVER_INPUT_FILE, ops::debug::compile_bin_package_for_debugging};
use nargo_toml::{PackageSelection, get_package_manifest, resolve_workspace_from_toml};
use noir_tracer::tracer_glue::begin_trace;
use noir_tracer::tracer_glue::finish_trace;
use noirc_abi::InputMap;
use noirc_abi::input_parser::Format;
use noirc_artifacts::debug::DebugArtifact;
use noirc_artifacts::program::CompiledProgram;
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;

use super::fs::inputs::read_inputs_from_file;
use crate::errors::CliError;

use codetracer_trace_writer::{create_trace_writer, TraceEventsFileFormat};

use super::NargoConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum TraceFormat {
    /// Binary format version 1 (latest)
    Binary,

    /// Binary format version 0
    BinaryV0,

    /// JSON text format
    Json
}

impl FromStr for TraceFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binary" => Ok(Self::Binary),
            "binaryv0" => Ok(Self::BinaryV0),
            "json" => Ok(Self::Json),
            other => Err(format!("Unknown trace format '{other}'")),
        }
    }
}

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

    /// The trace file format to use ('binary' or 'json')
    #[arg(value_parser = clap::value_parser!(TraceFormat))]
    #[clap(long, default_value = "binary")]
    trace_format: TraceFormat,
}

pub(crate) fn run(args: TraceCommand, config: NargoConfig) -> Result<(), CliError> {
    let acir_mode = false;
    let skip_instrumentation = false;

    let trace_format = match args.trace_format {
        TraceFormat::Binary => TraceEventsFileFormat::Binary,
        TraceFormat::BinaryV0 => TraceEventsFileFormat::BinaryV0,
        TraceFormat::Json => TraceEventsFileFormat::Json,
    };

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

    let compile_options = compile_options_for_debugging(
        acir_mode,
        skip_instrumentation,
        args.compile_options.clone(),
    );

    let compiled_program =
        compile_bin_package_for_debugging(&workspace, package, &compile_options)?;

    trace_program_and_decode(
        compiled_program,
        package,
        &args.prover_name,
        &args.trace_dir,
        args.compile_options.pedantic_solving,
        trace_format,
    )
}

fn trace_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
    trace_dir: &str,
    pedantic_solving: bool,
    trace_format: TraceEventsFileFormat,
) -> Result<(), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;

    trace_program(&program, &package.name, &inputs_map, trace_dir, pedantic_solving, trace_format)
}

pub(crate) fn trace_program(
    compiled_program: &CompiledProgram,
    crate_name: &CrateName,
    inputs_map: &InputMap,
    trace_dir: &str,
    pedantic_solving: bool,
    trace_format: TraceEventsFileFormat,
) -> Result<(), CliError> {
    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let debug_artifact = DebugArtifact {
        debug_symbols: compiled_program.debug.clone(),
        file_map: compiled_program.file_map.clone(),
    };

    let crate_name_string: String = crate_name.into();
    let mut tracer = create_trace_writer(crate_name_string.as_str(), &[], trace_format);
    begin_trace(&mut *tracer, trace_dir, trace_format);
    if let Err(error) = noir_tracer::trace_circuit(
        &Bn254BlackBoxSolver(pedantic_solving),
        &compiled_program.program.functions,
        &debug_artifact,
        initial_witness,
        &compiled_program.program.unconstrained_functions,
        &compiled_program.abi.error_types,
        &mut *tracer,
    ) {
        return Err(CliError::from(error));
    };

    finish_trace(&mut *tracer, trace_dir);

    Ok(())
}
