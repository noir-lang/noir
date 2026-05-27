#[cfg(unix)]
use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
#[cfg(unix)]
use iai::black_box;
#[cfg(unix)]
use paste::paste;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
include!("./utils.rs");

#[cfg(unix)]
macro_rules! iai_command {
    ($command_name:tt, $command_string:expr_2021) => {
        paste! {
            fn [<iai_selected_tests_ $command_name>]() {
                let test_program_dirs = get_selected_tests();
                for test_program_dir in test_program_dirs {
                    #[allow(deprecated)]
                    let mut cmd = Command::cargo_bin("nargo").unwrap();
                    cmd.arg("--program-dir").arg(&test_program_dir);
                    cmd.arg($command_string);

                    black_box(cmd.assert());
                }
            }
        }
    };
}
#[cfg(unix)]
iai_command!(execution, "execute");
#[cfg(unix)]
iai::main!(iai_selected_tests_execution);

#[cfg(not(unix))]
fn main() {}
