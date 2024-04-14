use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("parse.rs");
    let mut test_file = File::create(destination).unwrap();

    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
        Err(_) => std::env::current_dir().unwrap(),
    };
    let test_dir = root_dir.join("tooling/parser-fuzz-target/out");

    // Rebuild if the tests have changed
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed={}", test_dir.as_os_str().to_str().unwrap());

    generate_parse_failure_tests(&mut test_file, &test_dir);
}

fn generate_parse_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let path_read_error =
        format!("couldn't find fuzzing result directory at: {}", test_data_dir.display());
    for fuzzer_config_entry in test_data_dir.read_dir().expect(&path_read_error) {
        let crash_dir = fuzzer_config_entry.unwrap().path().join("crashes");
        if crash_dir.is_dir() {
            for crash_entry in crash_dir.read_dir().unwrap() {
                let crash_path = crash_entry.unwrap().path();
                if crash_path.is_file() {
                    let mut test_name = crash_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .expect("Crash filename can't be converted to str")
                        .to_string();
                    if test_name == "README.txt" {
                        continue;
                    }
                    test_name = test_name.replace("+", "_");
                    test_name = test_name.replace(",", "_");
                    test_name = test_name.replace(".", "_");
                    test_name = test_name.replace(":", "_");

                    write!(
                        test_file,
                        r#"
#[test]
fn parse_failure_{test_name}() {{
    let program_str = std::fs::read_to_string("{crash_path}").unwrap();
    let _ = parse_program(&program_str);
}}
                        "#,
                        crash_path = crash_path.display(),
                    )
                    .expect("Could not write templated test file.");
                }
            }
        }
    }
}
