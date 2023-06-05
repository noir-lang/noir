use rustc_version::{version, Version};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.66.0").unwrap(),
        "The minimal supported rustc version is 1.66.0."
    );
}

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() {
    check_rustc_version();

    // Only use build_data if the environment variable isn't set
    // The environment variable is always set when working via Nix
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("prove_and_verify.rs");
    let mut test_file = File::create(destination).unwrap();

    generate_tests(&mut test_file, false);
    generate_tests(&mut test_file, true);
}

fn load_conf(conf_path: &Path) -> BTreeMap<String, Vec<String>> {
    let config_str = std::fs::read_to_string(conf_path).unwrap();

    let mut conf_data = match toml::from_str(&config_str) {
        Ok(t) => t,
        Err(_) => {
            BTreeMap::from([("exclude".to_string(), Vec::new()), ("fail".to_string(), Vec::new())])
        }
    };
    if conf_data.get("exclude").is_none() {
        conf_data.insert("exclude".to_string(), Vec::new());
    }
    if conf_data.get("fail").is_none() {
        conf_data.insert("fail".to_string(), Vec::new());
    }
    conf_data
}

fn generate_tests(test_file: &mut File, experimental_ssa: bool) {
    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap().join("crates").join("nargo_cli"),
    };
    // Choose the test directory depending on whether we are in the SSA refactor module or not
    let test_sub_dir = if experimental_ssa { "test_data_ssa_refactor" } else { "test_data" };
    let test_data_dir = manifest_dir.join("tests").join(test_sub_dir);
    let config_path = test_data_dir.join("config.toml");

    // Load config.toml file from `test_data` directory
    let config_data: BTreeMap<String, Vec<String>> = load_conf(&config_path);

    let test_case_dirs =
        fs::read_dir(&test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        let exclude_macro =
            if config_data["exclude"].contains(&test_name) { "#[ignore]" } else { "" };

        let should_fail = config_data["fail"].contains(&test_name);

        write!(
            test_file,
            r#"
{exclude_macro}
#[test]
fn prove_and_verify_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let verified = std::panic::catch_unwind(|| {{
        nargo_cli::cli::prove_and_verify(&test_program_dir, {experimental_ssa})
    }});

    let r = match verified {{
        Ok(result) => result,
        Err(_) => {{
            panic!(
                "\n\n\nPanic occurred while running test {test_name} (ignore the following panic)"
            );
        }}
    }};

    if {should_fail} {{
        assert!(!r, "{test_name} should not succeed");
    }} else {{
        assert!(r, "verification fail for {test_name}");
    }}
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}
