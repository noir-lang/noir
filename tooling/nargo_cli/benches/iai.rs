//! Select representative tests to bench with iai
use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use iai::black_box;
use std::process::Command;
include!("./utils.rs");

macro_rules! iai_command {
    ($fn_name:ident, $command_string:expr_2021) => {
        fn $fn_name() {
            let test_program_dirs = get_selected_tests();
            for test_program_dir in test_program_dirs {
                #[allow(deprecated)]
                let mut cmd = Command::cargo_bin("nargo").unwrap();
                cmd.arg("--program-dir").arg(&test_program_dir);
                cmd.arg($command_string);

                black_box(cmd.assert());
            }
        }
    };
}
iai_command!(iai_selected_tests_execution, "execute");
iai::main!(iai_selected_tests_execution);
