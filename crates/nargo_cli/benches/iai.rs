//! Select representative tests to bench with iai
use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use iai::black_box;
use paste::paste;
use std::process::Command;
include!("./utils.rs");

macro_rules! iai_command {
    ($command_name:tt, $command_string:expr) => {
        paste! {
            fn [<iai_selected_tests_ $command_name>]() {
                let test_program_dirs = get_selected_tests();
                for test_program_dir in test_program_dirs {
                    let mut cmd = Command::cargo_bin("nargo").unwrap();
                    cmd.arg("--program-dir").arg(&test_program_dir);
                    cmd.arg($command_string);

                    black_box(cmd.assert());
                }
            }
        }
    };
}
iai_command!(execution, "execute");
iai_command!(prove, "prove");

iai::main!(iai_selected_tests_execution, iai_selected_tests_prove);
