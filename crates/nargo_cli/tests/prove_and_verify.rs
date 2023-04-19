use tempdir::TempDir;

use std::collections::BTreeMap;
use std::fs;

const TEST_DIR: &str = "tests";
const TEST_DATA_DIR: &str = "test_data";
const CONFIG_FILE: &str = "config.toml";

mod tests {
    use std::path::Path;

    use super::*;

    fn load_conf(conf_path: &Path) -> BTreeMap<String, Vec<String>> {
        let config_str = std::fs::read_to_string(conf_path).unwrap();

        let mut conf_data = match toml::from_str(&config_str) {
            Ok(t) => t,
            Err(_) => BTreeMap::from([
                ("exclude".to_string(), Vec::new()),
                ("fail".to_string(), Vec::new()),
            ]),
        };
        if conf_data.get("exclude").is_none() {
            conf_data.insert("exclude".to_string(), Vec::new());
        }
        if conf_data.get("fail").is_none() {
            conf_data.insert("fail".to_string(), Vec::new());
        }
        conf_data
    }

    /// Copy files from source to destination recursively.
    pub fn copy_recursively(
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        fs::create_dir_all(&destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let filetype = entry.file_type()?;
            if filetype.is_dir() {
                copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    #[test]
    fn noir_integration() {
        let current_dir = std::env::current_dir().unwrap();

        let test_data_dir = current_dir.join(TEST_DIR).join(TEST_DATA_DIR);

        // Load config.toml file from test_data directory
        let config_file_path = test_data_dir.join(CONFIG_FILE);
        let config_data: BTreeMap<String, Vec<String>> = load_conf(&config_file_path);

        // Copy all the test cases into a temp dir so we don't leave artifacts around.
        let tmp_dir = TempDir::new("p_and_v_tests").unwrap();
        copy_recursively(test_data_dir, &tmp_dir)
            .expect("failed to copy test cases to temp directory");

        let test_case_dirs =
            fs::read_dir(&tmp_dir).unwrap().flatten().filter(|c| c.path().is_dir());

        for test_dir in test_case_dirs {
            let test_name =
                test_dir.file_name().into_string().expect("Directory can't be converted to string");
            let test_program_dir = &test_dir.path();

            if config_data["exclude"].contains(&test_name) {
                println!("Skipping test {test_name}");
                continue;
            }

            println!("Running test {test_name}");

            let verified = std::panic::catch_unwind(|| {
                nargo_cli::cli::prove_and_verify("pp", test_program_dir, false)
            });

            let r = match verified {
                Ok(result) => result,
                Err(_) => {
                    panic!(
                        "\n\n\nPanic occurred while running test {test_name} (ignore the following panic)"
                    );
                }
            };

            if config_data["fail"].contains(&test_name) {
                assert!(!r, "{:?} should not succeed", test_name);
            } else {
                assert!(r, "verification fail for {:?}", test_name);
            }
        }

        // Ensure that temp dir remains alive until all tests have run.
        drop(tmp_dir);
    }
}
