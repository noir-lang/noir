#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    // Some of these imports are consumed by the injected tests
    use assert_cmd::prelude::*;
    use predicates::prelude::*;

    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::*;

    // Utilities to keep the test matrix labels more intuitive.
    #[derive(Debug, Clone, Copy)]
    struct ForceBrillig(pub bool);
    #[derive(Debug, Clone, Copy)]
    struct Inliner(pub i64);

    fn setup_nargo(
        test_program_dir: &Path,
        test_command: &str,
        force_brillig: ForceBrillig,
        inliner_aggressiveness: Inliner,
    ) -> Command {
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(test_program_dir);
        nargo.arg(test_command).arg("--force");
        nargo.arg("--inliner-aggressiveness").arg(inliner_aggressiveness.0.to_string());
        // Check whether the test case is non-deterministic
        nargo.arg("--check-non-determinism");
        // Allow more bytecode in exchange to catch illegal states.
        nargo.arg("--enable-brillig-debug-assertions");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        if force_brillig.0 {
            {
                nargo.arg("--force-brillig");

                // Set the maximum increase so that part of the optimization is exercised (it might fail).
                nargo.arg("--max-bytecode-increase-percent");
                nargo.arg("50");
            }
        }

        nargo
    }

    fn remove_noise_lines(string: String) -> String {
        string
            .lines()
            .filter(|line| {
                !line.contains("Witness saved to")
                    && !line.contains("Circuit witness successfully solved")
                    && !line.contains("Waiting for lock")
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn delete_test_program_dir_occurrences(string: String, test_program_dir: &Path) -> String {
        let test_program_base_dir = test_program_dir.parent().unwrap().parent().unwrap();
        let mut test_program_base_dir = test_program_base_dir.to_string_lossy().to_string();
        if !test_program_base_dir.ends_with('/') {
            test_program_base_dir.push('/');
        }

        let mut test_program_dir = test_program_dir.to_string_lossy().to_string();
        if !test_program_dir.ends_with('/') {
            test_program_dir.push('/');
        }

        // We replace both test_program_dir (test_programs/compile_failure/foo) and its ancestor
        // (test_programs) because in some cases some programs refer to other programs in `test_programs`.
        string
            .lines()
            .map(|line| {
                line.replace(&test_program_dir, "").replace(&test_program_base_dir, "../../")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn execution_success(mut nargo: Command, test_program_dir: PathBuf, check_stdout: bool) {
        nargo.assert().success();

        if check_stdout {
            let output = nargo.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stdout = remove_noise_lines(stdout);

            let stdout_path = test_program_dir.join("stdout.txt");
            let expected_stdout = if stdout_path.exists() {
                String::from_utf8(fs::read(stdout_path).unwrap()).unwrap()
            } else {
                String::new()
            };

            // Remove any trailing newlines added by some editors
            let stdout = stdout.trim();
            let expected_stdout = expected_stdout.trim();

            if stdout != expected_stdout {
                println!(
                    "stdout does not match expected output. Expected:\n{expected_stdout}\n\nActual:\n{stdout}"
                );
                assert_eq!(stdout, expected_stdout);
            }
        }
    }

    fn execution_failure(mut nargo: Command) {
        nargo
            .assert()
            .failure()
            .stderr(predicate::str::contains("The application panicked (crashed).").not());
    }

    fn noir_test_success(mut nargo: Command) {
        nargo.assert().success();
    }

    fn noir_test_failure(mut nargo: Command) {
        nargo
            .assert()
            .failure()
            .stderr(predicate::str::contains("The application panicked (crashed).").not());
    }

    fn compile_success_empty(mut nargo: Command, no_warnings: bool) {
        nargo.arg("--json");
        let output = nargo.output().expect("Failed to execute command");

        if !output.status.success() {
            {
                panic!(
                    "`nargo info` failed with: {}",
                    String::from_utf8(output.stderr).unwrap_or_default()
                );
            }
        }

        if no_warnings {
            nargo.assert().success().stderr(predicate::str::contains("warning:").not());
        }

        // `compile_success_empty` tests should be able to compile down to an empty circuit.
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            {
                panic!(
                    "JSON was not well-formatted {:?}\n\n{:?}",
                    e,
                    std::str::from_utf8(&output.stdout)
                )
            }
        });
        let num_opcodes = &json["programs"][0]["functions"][0]["opcodes"];
        assert_eq!(
            num_opcodes.as_u64().expect("number of opcodes should fit in a u64"),
            0,
            "expected the number of opcodes to be 0"
        );
    }

    fn compile_success_contract(mut nargo: Command) {
        nargo.assert().success().stderr(predicate::str::contains("warning:").not());
    }

    fn compile_success_no_bug(mut nargo: Command) {
        nargo.assert().success().stderr(predicate::str::contains("bug:").not());
    }

    fn compile_success_with_bug(mut nargo: Command) {
        nargo.assert().success().stderr(predicate::str::contains("bug:"));
    }

    fn compile_failure(mut nargo: Command, test_program_dir: PathBuf) {
        nargo
            .assert()
            .failure()
            .stderr(predicate::str::contains("The application panicked (crashed).").not());

        let output = nargo.output().unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        let stderr = remove_noise_lines(stderr);
        let stderr = delete_test_program_dir_occurrences(stderr, &test_program_dir);

        let stderr_path = test_program_dir.join("stderr.txt");

        let expected_stderr = if stderr_path.exists() {
            String::from_utf8(fs::read(stderr_path).unwrap()).unwrap()
        } else {
            String::new()
        };

        // Remove any trailing newlines added by some editors
        let stderr = stderr.trim();
        let expected_stderr = expected_stderr.trim();

        if stderr != expected_stderr {
            println!(
                "stderr does not match expected output. Expected:\n{expected_stderr}\n\nActual:\n{stderr}"
            );
            assert_eq!(stderr, expected_stderr);
        }
    }

    // include tests generated by `build.rs`
    include!(concat!(env!("OUT_DIR"), "/execute.rs"));
}
