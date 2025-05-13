#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use acvm::FieldElement;
    use acvm::acir::circuit::Program;
    // Some of these imports are consumed by the injected tests
    use assert_cmd::prelude::*;
    use base64::Engine;
    use insta::assert_snapshot;
    use insta::internals::Content;
    use insta::internals::Redaction;
    use noirc_artifacts::contract::ContractArtifact;
    use noirc_artifacts::program::ProgramArtifact;
    use predicates::prelude::*;
    use serde::Deserialize;

    use std::collections::BTreeMap;
    use std::fs;
    use std::io::BufWriter;
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
        // Allow more bytecode in exchange to catch illegal states.
        nargo.arg("--enable-brillig-debug-assertions");

        // Enable pedantic solving
        let skip_pedantic_solving = [
            // TODO(https://github.com/noir-lang/noir/issues/8098): all of these are failing with:
            // ```
            // Failed to solve program:
            // \'Failed to solve blackbox function: embedded_curve_add, reason: Infinite input: embedded_curve_add(infinity, infinity)\'
            // ```
            "execution_success/multi_scalar_mul",
            "execution_success/regression_5045",
            "execution_success/regression_7744",
        ];
        if !skip_pedantic_solving
            .into_iter()
            .any(|test_to_skip| test_program_dir.ends_with(test_to_skip))
        {
            nargo.arg("--pedantic-solving");
        }

        // Enable enums and ownership as unstable features
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
        // Assuming `test_program_dir` is "/projects/noir/test_programs/compile_failure/some_program"...

        // `test_program_base_dir` is "/projects/noir/test_programs"
        let test_program_base_dir = test_program_dir.parent().unwrap().parent().unwrap();

        // `root_dir` is "/projects/noir"
        let root_dir = test_program_base_dir.parent().unwrap();

        // Here we turn the paths into strings and ensure they end with a `/`
        let mut test_program_base_dir = test_program_base_dir.to_string_lossy().to_string();
        if !test_program_base_dir.ends_with('/') {
            test_program_base_dir.push('/');
        }

        let mut test_program_dir = test_program_dir.to_string_lossy().to_string();
        if !test_program_dir.ends_with('/') {
            test_program_dir.push('/');
        }

        // `test_program_dir_without_root is "test_programs/compile_failure".
        // This one is needed because tests might run from the root of the project and paths
        // will end up starting with "test_programs/compile_failure/...".
        let test_program_dir_without_root =
            test_program_dir.strip_prefix(&root_dir.to_string_lossy().to_string()).unwrap();
        let test_program_dir_without_root = test_program_dir_without_root
            .strip_prefix('/')
            .unwrap_or(test_program_dir_without_root);

        // We replace all of these:
        // - test_program_dir ("/projects/noir/test_programs/compile_failure/foo")
        // - test_program_base_dir ("/projects/noir/test_programs")
        // - test_program_dir_without_root ("test_programs/compile_failure")
        string
            .lines()
            .map(|line| {
                line.replace(&test_program_dir, "")
                    .replace(&test_program_base_dir, "test_programs/")
                    .replace(test_program_dir_without_root, "")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn execution_success(
        mut nargo: Command,
        test_program_dir: PathBuf,
        check_stdout: bool,
        force_brillig: ForceBrillig,
        inliner: Inliner,
    ) {
        let target_dir = tempfile::tempdir().unwrap().into_path();

        nargo.arg(format!("--target-dir={}", target_dir.to_string_lossy()));

        nargo.assert().success();

        if check_stdout {
            let output = nargo.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stdout = remove_noise_lines(stdout);

            let stdout_path = test_program_dir.join("stdout.txt");
            let expected_stdout = if stdout_path.exists() {
                String::from_utf8(fs::read(stdout_path.clone()).unwrap()).unwrap()
            } else {
                String::new()
            };

            // Remove any trailing newlines added by some editors
            let stdout = stdout.trim();
            let expected_stdout = expected_stdout.trim();

            if stdout != expected_stdout {
                if std::env::var("OVERWRITE_TEST_OUTPUT").is_ok() {
                    fs::write(stdout_path, stdout.to_string() + "\n").unwrap();
                } else {
                    println!(
                        "stdout does not match expected output. Expected:\n{expected_stdout}\n\nActual:\n{stdout}"
                    );
                    if expected_stdout.is_empty() && !stdout_path.exists() {
                        println!(
                            "Hint: set the OVERWRITE_TEST_OUTPUT env var to establish a stdout.txt"
                        )
                    }
                    assert_eq!(stdout, expected_stdout);
                }
            }
        }

        check_program_artifact(
            "execution_success",
            &test_program_dir,
            &target_dir,
            force_brillig,
            inliner,
        );
    }

    fn execution_failure(mut nargo: Command) {
        nargo
            .assert()
            .failure()
            .stderr(predicate::str::contains("The application panicked (crashed).").not());
    }

    fn execution_panic(mut nargo: Command) {
        nargo
            .assert()
            .failure()
            .stderr(predicate::str::contains("The application panicked (crashed)."));
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

    fn compile_success_empty(
        mut nargo: Command,
        test_program_dir: PathBuf,
        no_warnings: bool,
        force_brillig: ForceBrillig,
        inliner: Inliner,
    ) {
        let target_dir = test_program_dir
            .join(format!("target_force_brillig_{}_inliner_{}", force_brillig.0, inliner.0));

        nargo.arg(format!("--target-dir={}", target_dir.to_string_lossy()));

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

        check_program_artifact(
            "compile_success_empty",
            &test_program_dir,
            &target_dir,
            force_brillig,
            inliner,
        );
    }

    fn compile_success_contract(
        mut nargo: Command,
        test_program_dir: PathBuf,
        force_brillig: ForceBrillig,
        inliner: Inliner,
    ) {
        let target_dir = test_program_dir
            .join(format!("target_force_brillig_{}_inliner_{}", force_brillig.0, inliner.0));
        nargo.arg(format!("--target-dir={}", target_dir.to_string_lossy()));

        nargo.assert().success().stderr(predicate::str::contains("warning:").not());

        check_contract_artifact(
            "compile_success_contract",
            &test_program_dir,
            &target_dir,
            force_brillig,
            inliner,
        );
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
            String::from_utf8(fs::read(stderr_path.clone()).unwrap()).unwrap()
        } else {
            String::new()
        };

        // Remove any trailing newlines added by some editors
        let stderr = stderr.trim();
        let expected_stderr = expected_stderr.trim();

        if stderr != expected_stderr {
            if std::env::var("OVERWRITE_TEST_OUTPUT").is_ok() {
                fs::write(stderr_path, stderr.to_string() + "\n").unwrap();
            } else {
                // If the expected stderr is empty this is likely a new test, so we produce the expected output for next time
                if expected_stderr.is_empty() {
                    fs::write(stderr_path, stderr.to_string() + "\n").unwrap();
                }

                println!(
                    "stderr does not match expected output. Expected:\n{expected_stderr}\n\nActual:\n{stderr}"
                );
                assert_eq!(stderr, expected_stderr);
            }
        }
    }

    fn nargo_expand_execute(test_program_dir: PathBuf) {
        // First run `nargo execute` on the original code to get the output
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(test_program_dir.clone());
        nargo.arg("execute").arg("--force").arg("--disable-comptime-printing");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        // Enable pedantic solving
        nargo.arg("--pedantic-solving");

        nargo.assert().success();

        let original_output = nargo.output().unwrap();
        let original_output: String = String::from_utf8(original_output.stdout).unwrap();

        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(test_program_dir.clone());
        nargo.arg("expand").arg("--force").arg("--disable-comptime-printing");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        // Enable pedantic solving
        nargo.arg("--pedantic-solving");

        nargo.assert().success();

        let expanded_code = nargo.output().unwrap();
        let expanded_code: String = String::from_utf8(expanded_code.stdout).unwrap();

        let test_name = test_program_dir.file_name().unwrap().to_string_lossy().to_string();
        let snapshot_name = "expanded";
        insta::with_settings!(
        {
            snapshot_path => format!("./snapshots/expand/execution_success/{test_name}")
        },
        {
            insta::assert_snapshot!(snapshot_name, expanded_code)
        });

        // Create a new directory where we'll put the expanded code
        let temp_dir = tempfile::tempdir().unwrap().into_path();

        // Copy everything from the original directory to the new directory
        // (because some depdendencies might be there and might be needed for the expanded code to work)
        copy_dir_all(test_program_dir.clone(), temp_dir.clone()).unwrap();

        // Create a main file for the expanded code
        fs::write(temp_dir.join("src").join("main.nr"), expanded_code).unwrap();

        // First check if `nargo fmt` works on the expanded code. If not, it means the code is not valid.
        run_nargo_fmt(temp_dir.clone());

        // Now we can run `nargo execute` on the expanded code
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(temp_dir);
        nargo.arg("execute").arg("--force").arg("--disable-comptime-printing");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        // Enable pedantic solving
        nargo.arg("--pedantic-solving");

        nargo.assert().success();

        let expanded_output = nargo.output().unwrap();
        let expanded_output: String = String::from_utf8(expanded_output.stdout).unwrap();

        let original_output = remove_noise_lines(original_output);
        let expanded_output = remove_noise_lines(expanded_output);

        assert_eq!(original_output, expanded_output);
    }

    fn nargo_expand_compile(test_program_dir: PathBuf, prefix: &'static str) {
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(test_program_dir.clone());
        nargo.arg("expand").arg("--force").arg("--disable-comptime-printing");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        // Enable pedantic solving
        nargo.arg("--pedantic-solving");

        nargo.assert().success();

        let expanded_code = nargo.output().unwrap();
        let expanded_code: String = String::from_utf8(expanded_code.stdout).unwrap();

        let test_name = test_program_dir.file_name().unwrap().to_string_lossy().to_string();
        let snapshot_name = "expanded";
        insta::with_settings!(
        {
            snapshot_path => format!("./snapshots/expand/{prefix}/{test_name}")
        },
        {
            insta::assert_snapshot!(snapshot_name, expanded_code)
        });

        // Create a new directory where we'll put the expanded code
        let temp_dir = tempfile::tempdir().unwrap().into_path();

        // Copy everything from the original directory to the new directory
        // (because some depdendencies might be there and might be needed for the expanded code to work)
        copy_dir_all(test_program_dir.clone(), temp_dir.clone()).unwrap();

        // Create a main file for the expanded code
        fs::write(temp_dir.join("src").join("main.nr"), expanded_code).unwrap();

        // First check if `nargo fmt` works on the expanded code. If not, it means the code is not valid.
        run_nargo_fmt(temp_dir.clone());

        // Now we can run `nargo compile` on the expanded code
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(temp_dir);
        nargo.arg("compile").arg("--force");

        // Enable enums as an unstable feature
        nargo.arg("-Zenums");

        // Enable pedantic solving
        nargo.arg("--pedantic-solving");

        nargo.assert().success();
    }

    fn run_nargo_fmt(target_dir: PathBuf) {
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo.arg("--program-dir").arg(target_dir);
        nargo.arg("fmt");
        nargo.assert().success();
    }

    fn check_program_artifact(
        prefix: &'static str,
        test_program_dir: &Path,
        target_dir: &PathBuf,
        force_brillig: ForceBrillig,
        inliner: Inliner,
    ) {
        let artifact_filename =
            find_program_artifact_in_dir(target_dir).expect("Expected an artifact to exist");

        let artifact_file = fs::File::open(&artifact_filename).unwrap();
        let artifact: ProgramArtifact = serde_json::from_reader(artifact_file).unwrap();

        let _ = fs::remove_dir_all(target_dir);

        let test_name = test_program_dir.file_name().unwrap().to_string_lossy().to_string();
        if test_name == "workspace" {
            // workspace outputs multiple artifacts so we get a non-deterministic result.
            return;
        }

        let snapshot_name = format!("force_brillig_{}_inliner_{}", force_brillig.0, inliner.0);
        insta::with_settings!(
            {
                snapshot_path => format!("./snapshots/{prefix}/{test_name}")
            },
            {
            insta::assert_json_snapshot!(snapshot_name, artifact, {
                ".noir_version" => "[noir_version]",
                ".hash" => "[hash]",
                ".bytecode" => insta::dynamic_redaction(|value, _path| {
                    // assert that the value looks like a uuid here
                    let bytecode_b64 = value.as_str().unwrap();
                    let bytecode = base64::engine::general_purpose::STANDARD
                        .decode(bytecode_b64)
                        .unwrap();
                    let program = Program::<FieldElement>::deserialize_program(&bytecode).unwrap();
                    Content::Seq(program.to_string().split("\n").filter(|line: &&str| !line.is_empty()).map(Content::from).collect::<Vec<Content>>())
                }),
                ".file_map.**.path" => file_map_path_redaction(),
            })
        })
    }

    fn check_contract_artifact(
        prefix: &'static str,
        test_program_dir: &Path,
        target_dir: &PathBuf,
        force_brillig: ForceBrillig,
        inliner: Inliner,
    ) {
        let artifact_filename =
            find_program_artifact_in_dir(target_dir).expect("Expected an artifact to exist");

        let artifact_file = fs::File::open(&artifact_filename).unwrap();
        let artifact: ContractArtifact = serde_json::from_reader(artifact_file).unwrap();

        fs::remove_dir_all(target_dir).expect("Could not remove target dir");

        let test_name = test_program_dir.file_name().unwrap().to_string_lossy().to_string();

        let snapshot_name = format!("force_brillig_{}_inliner_{}", force_brillig.0, inliner.0);
        insta::with_settings!(
            {
                snapshot_path => format!("./snapshots/{prefix}/{test_name}")
            },
            {
            insta::assert_json_snapshot!(snapshot_name, artifact, {
                ".noir_version" => "[noir_version]",
                ".functions[].hash" => "[hash]",
                ".file_map.**.path" => file_map_path_redaction(),
            })
        })
    }

    fn file_map_path_redaction() -> Redaction {
        insta::dynamic_redaction(|value, _path| {
            // Some paths are absolute: clear those out.
            let value = value.as_str().expect("Expected a string value in a path entry");
            if value.starts_with("/") { String::new() } else { value.to_string() }
        })
    }

    fn find_program_artifact_in_dir(dir: &PathBuf) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }

        for entry in fs::read_dir(dir).unwrap() {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            };

            return Some(path);
        }

        None
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
        {
            fs::create_dir_all(&dst)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                if ty.is_dir() {
                    copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
                } else {
                    fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
                }
            }
            Ok(())
        }
    }

    // include tests generated by `build.rs`
    include!(concat!(env!("OUT_DIR"), "/execute.rs"));
}
