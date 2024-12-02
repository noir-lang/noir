//! Select representative tests to bench with criterion
use acvm::{acir::native_types::WitnessMap, FieldElement};
use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use criterion::{criterion_group, criterion_main, Criterion};

use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap,
};
use noirc_artifacts::program::ProgramArtifact;
use noirc_driver::CompiledProgram;
use pprof::criterion::{Output, PProfProfiler};
use std::hint::black_box;
use std::path::Path;
use std::{cell::RefCell, collections::BTreeMap};
use std::{process::Command, time::Duration};

include!("./utils.rs");

/// Compile the test program in a sub-process
/// The `force_brillig` option is used to benchmark the program as if it was executed by the AVM.
fn compile_program(test_program_dir: &Path, force_brillig: bool) {
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("compile");
    cmd.arg("--force");
    if force_brillig {
        cmd.arg("--force-brillig");
    }
    cmd.assert().success();
}

/// Read the bytecode(s) of the program(s) from the compilation artifacts
/// from all the binary packages. Pair them up with their respective input.
///
/// Based on `ExecuteCommand::run`.
fn read_compiled_programs_and_inputs(
    dir: &Path,
) -> Vec<(CompiledProgram, WitnessMap<FieldElement>)> {
    let toml_path = nargo_toml::get_package_manifest(dir).expect("failed to read manifest");
    let workspace = nargo_toml::resolve_workspace_from_toml(
        &toml_path,
        nargo_toml::PackageSelection::All,
        Some(noirc_driver::NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .expect("failed to resolve workspace");

    let mut programs = Vec::new();
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram = read_program_from_file(&program_artifact_path).into();

        let (inputs, _) = read_inputs_from_file(
            &package.root_dir,
            nargo::constants::PROVER_INPUT_FILE,
            Format::Toml,
            &program.abi,
        );

        let initial_witness =
            program.abi.encode(&inputs, None).expect("failed to encode input witness");

        programs.push((program, initial_witness));
    }
    programs
}

/// Read the bytecode and ABI from the compilation output
fn read_program_from_file(circuit_path: &Path) -> ProgramArtifact {
    let file_path = circuit_path.with_extension("json");
    let input_string = std::fs::read(file_path).expect("failed to read artifact file");
    serde_json::from_slice(&input_string).expect("failed to deserialize artifact")
}

/// Read the inputs from Prover.toml
fn read_inputs_from_file(
    path: &Path,
    file_name: &str,
    format: Format,
    abi: &Abi,
) -> (InputMap, Option<InputValue>) {
    if abi.is_empty() {
        return (BTreeMap::new(), None);
    }

    let file_path = path.join(file_name).with_extension(format.ext());
    if !file_path.exists() {
        if abi.parameters.is_empty() {
            return (BTreeMap::new(), None);
        } else {
            panic!("input file doesn't exist: {}", file_path.display());
        }
    }

    let input_string = std::fs::read_to_string(file_path).expect("failed to read input file");
    let mut input_map = format.parse(&input_string, abi).expect("failed to parse input");
    let return_value = input_map.remove(noirc_abi::MAIN_RETURN_NAME);

    (input_map, return_value)
}

/// Use the nargo CLI to compile a test program, then benchmark its execution
/// by executing the command directly from the benchmark, so that we can have
/// meaningful flamegraphs about the ACVM.
fn criterion_test_execution(c: &mut Criterion, test_program_dir: &Path, force_brillig: bool) {
    let benchmark_name = format!(
        "{}_execute{}",
        test_program_dir.file_name().unwrap().to_str().unwrap(),
        if force_brillig { "_brillig" } else { "" }
    );

    // The program and its inputs will be populated in the first setup.
    let artifacts = RefCell::new(None);

    let mut foreign_call_executor =
        nargo::foreign_calls::DefaultForeignCallExecutor::new(false, None, None, None);

    c.bench_function(&benchmark_name, |b| {
        b.iter_batched(
            || {
                // Setup will be called many times to set a batch (which we don't use),
                // but we can compile it only once, and then the executions will not have to do so.
                // It is done as a setup so that we only compile the test programs that we filter for.
                if artifacts.borrow().is_some() {
                    return;
                }
                compile_program(test_program_dir, force_brillig);
                // Parse the artifacts for use in the benchmark routine
                let programs = read_compiled_programs_and_inputs(test_program_dir);
                // Warn, but don't stop, if we haven't found any binary packages.
                if programs.is_empty() {
                    eprintln!("\nWARNING: There is nothing to benchmark in {benchmark_name}");
                }
                // Store them for execution
                artifacts.replace(Some(programs));
            },
            |_| {
                let artifacts = artifacts.borrow();
                let artifacts = artifacts.as_ref().expect("setup compiled them");

                for (program, initial_witness) in artifacts {
                    let _witness_stack = black_box(nargo::ops::execute_program(
                        black_box(&program.program),
                        black_box(initial_witness.clone()),
                        &bn254_blackbox_solver::Bn254BlackBoxSolver,
                        &mut foreign_call_executor,
                    ))
                    .expect("failed to execute program");
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Go through all the selected tests and executem with and without Brillig.
fn criterion_selected_tests_execution(c: &mut Criterion) {
    for test_program_dir in get_selected_tests() {
        for force_brillig in [false, true] {
            criterion_test_execution(c, &test_program_dir, force_brillig);
        }
    }
}

criterion_group! {
    name = execution_benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_selected_tests_execution
}
criterion_main!(execution_benches);
