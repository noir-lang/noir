use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() -> Result<(), String> {
    // Only use build_data if the environment variable isn't set.
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT()?;
        build_data::set_GIT_DIRTY()?;
        build_data::no_debug_rebuilds()?;
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("debug.rs");
    let mut test_file = File::create(destination).unwrap();

    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
        Err(_) => std::env::current_dir().unwrap(),
    };
    let test_dir = root_dir.join("test_programs");

    // Rebuild if the tests have changed
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed=ignored-tests.txt");
    // TODO(https://github.com/noir-lang/noir/issues/8351): Running the tests changes the timestamps on test_programs files (file lock?).
    // That has the knock-on effect of then needing to rebuild the tests after running the tests.
    println!("cargo:rerun-if-changed={}", test_dir.as_os_str().to_str().unwrap());

    generate_debugger_tests(&mut test_file, &test_dir);
    generate_test_runner_debugger_tests(&mut test_file, &test_dir);

    Ok(())
}

fn generate_debugger_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "execution_success";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs = std::fs::read_dir(test_data_dir)
        .unwrap()
        .flatten()
        .filter(|c| c.path().is_dir() && c.path().join("Nargo.toml").exists());
    let ignored_tests_contents = std::fs::read_to_string("ignored-tests.txt").unwrap();
    let ignored_tests = ignored_tests_contents.lines().collect::<HashSet<_>>();

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        let ignored = ignored_tests.contains(test_name.as_str());
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        write!(
            test_file,
            r#"
#[test]
{ignored}
fn debug_{test_name}() {{
    debugger_execution_success("{test_dir}");
}}
            "#,
            test_dir = test_dir.display(),
            ignored = if ignored { "#[ignore]" } else { "" },
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_test_runner_debugger_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "noir_test_success";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs = std::fs::read_dir(test_data_dir)
        .unwrap()
        .flatten()
        .filter(|c| c.path().is_dir() && c.path().join("Nargo.toml").exists());
    let ignored_tests_contents = std::fs::read_to_string("ignored-noir-tests.txt").unwrap();
    let ignored_tests = ignored_tests_contents.lines().collect::<HashSet<_>>();

    for test_dir in test_case_dirs {
        let test_file_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_file_name.contains('-') {
            panic!(
                "Invalid test directory: {test_file_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        let file_name = test_dir.join("src").join("main.nr");
        let buf_reader =
            BufReader::new(File::open(file_name.clone()).expect("Could not open file"));
        let lines = buf_reader.lines();
        let test_names: Vec<String> = lines
            .filter_map(|line_res| {
                line_res.ok().map(|line| if line.contains("fn test_") { Some(line) } else { None })
            })
            .flatten()
            .collect();
        for test_name_line in test_names {
            // TODO(https://github.com/noir-lang/noir/issues/8352): get test name by regex perhaps?
            let test_name = test_name_line
                .split("fn ")
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap()
                .split("<")
                .next()
                .unwrap()
                .split("(")
                .next()
                .unwrap();

            let ignored = ignored_tests.contains(test_name);

            write!(
                test_file,
                r#"
    #[test]
    {ignored}
    fn debug_test_{test_file_name}_{test_name}() {{
        debugger_test_success("{test_dir}", "{test_name}");
    }}
                "#,
                test_dir = test_dir.display(),
                ignored = if ignored { "#[ignore]" } else { "" },
            )
            .expect("Could not write templated test file.");
        }
    }
}
