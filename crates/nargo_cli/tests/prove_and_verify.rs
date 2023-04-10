use std::collections::BTreeMap;
use std::fs;

mod tests {
    use std::path::PathBuf;

    use super::*;

    fn load_conf(conf_path: &str) -> BTreeMap<String, Vec<String>> {
        // Parse config.toml into a BTreeMap, do not fail if config file does not exist.
        let mut conf_data = match toml::from_str(conf_path) {
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

    #[test]
    fn noir_integration() {
        // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
        // is the root of the repository and append the crate path
        let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => {
                PathBuf::from(std::env::current_dir().unwrap()).join("crates").join("nargo_cli")
            }
        };
        let test_data_dir = manifest_dir.join("tests").join("test_data");
        let config_path = test_data_dir.join("config.toml");

        //load config.tml file from test_data directory
        let config_path = std::fs::read_to_string(config_path).unwrap();
        let config_data: BTreeMap<String, Vec<String>> = load_conf(&config_path);

        for c in fs::read_dir(test_data_dir).unwrap().flatten() {
            if let Ok(test_name) = c.file_name().into_string() {
                if c.path().is_dir() && !config_data["exclude"].contains(&test_name) {
                    println!("Running test {test_name:?}");
                    let verified = std::panic::catch_unwind(|| {
                        nargo_cli::cli::prove_and_verify("pp", &c.path(), false)
                    });

                    let r = match verified {
                        Ok(result) => result,
                        Err(_) => {
                            panic!("\n\n\nPanic occurred while running test {:?} (ignore the following panic)", c.file_name());
                        }
                    };

                    if config_data["fail"].contains(&test_name) {
                        assert!(!r, "{:?} should not succeed", c.file_name());
                    } else {
                        assert!(r, "verification fail for {:?}", c.file_name());
                    }
                } else {
                    println!("Ignoring test {test_name:?}");
                }
            }
        }
    }
}
