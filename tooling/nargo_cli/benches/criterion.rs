//! Select representative tests to bench with criterion
use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use criterion::{criterion_group, criterion_main, Criterion};

use nargo_cli::cli;
use noirc_driver::CompileOptions;
use pprof::criterion::{Output, PProfProfiler};
use std::{process::Command, time::Duration};
include!("./utils.rs");

/// Use the nargo CLI to compile a test program, then benchmark its execution
/// by executing the command directly from the benchmark, so that we can have
/// meaningful flamegraphs about the ACVM.
fn criterion_selected_tests_execution(c: &mut Criterion) {
    for test_program_dir in get_selected_tests() {
        let mut compiled = false;
        let benchmark_name =
            format!("{}_execute", test_program_dir.file_name().unwrap().to_str().unwrap());

        c.bench_function(&benchmark_name, |b| {
            b.iter_batched(
                || {
                    // Setup will be called many times to set a batch (which we don't use),
                    // but we can compile it only once, and then the executions will not have to do so.
                    // It is done as a setup so that we only compile the test programs that we filter for.
                    if !compiled {
                        compiled = true;
                        let mut cmd = Command::cargo_bin("nargo").unwrap();
                        cmd.arg("--program-dir").arg(&test_program_dir);
                        cmd.arg("compile");
                        cmd.arg("--force");
                        cmd.assert().success();
                    }
                },
                |_| {
                    let cmd = cli::NargoCli {
                        command: cli::NargoCommand::Execute(cli::execute_cmd::ExecuteCommand {
                            witness_name: None,
                            prover_name: nargo::constants::PROVER_INPUT_FILE.to_string(),
                            package: None,
                            workspace: true,
                            compile_options: CompileOptions {
                                silence_warnings: true,
                                ..Default::default()
                            },
                            oracle_resolver: None,
                        }),
                        config: cli::NargoConfig { program_dir: test_program_dir.clone() },
                    };
                    cli::run_cmd(cmd).expect("failed to execute command");
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

criterion_group! {
    name = execution_benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_selected_tests_execution
}
criterion_main!(execution_benches);
