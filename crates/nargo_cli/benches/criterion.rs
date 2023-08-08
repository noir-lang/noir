//! Select representative tests to bench with criterion

use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use criterion::{criterion_group, criterion_main, Criterion};
use paste::paste;
use pprof::criterion::{Output, PProfProfiler};
use std::process::Command;
include!("./utils.rs");

macro_rules! criterion_command {
    ($command_name:tt, $command_string:expr) => {
        paste! {
            fn [<criterion_selected_tests_ $command_name>](c: &mut Criterion) {
                let test_program_dirs = get_selected_tests();
                for test_program_dir in test_program_dirs {
                    let mut cmd = Command::cargo_bin("nargo").unwrap();
                    cmd.arg("--program-dir").arg(&test_program_dir);
                    cmd.arg($command_string);

                    c.bench_function(&format!("{}_{}", test_program_dir.file_name().unwrap().to_str().unwrap(), $command_string), |b| {
                        b.iter(|| cmd.assert())
                    });
                }
            }
        }
    };
}
criterion_command!(execution, "execute");

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets =  criterion_selected_tests_execution
}
criterion_main!(benches);
