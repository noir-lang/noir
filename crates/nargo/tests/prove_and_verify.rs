use std::collections::BTreeMap;
use std::fs;

const TEST_DIR: &str = "tests";
const TEST_DATA_DIR: &str = "test_data";
const CONFIG_FILE: &str = "config.toml";

mod tests {
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
        let mut current_dir = std::env::current_dir().unwrap();
        current_dir.push(TEST_DIR);
        current_dir.push(TEST_DATA_DIR);

        //load config.tml file from test_data directory
        current_dir.push(CONFIG_FILE);
        let config_path = std::fs::read_to_string(current_dir).unwrap();
        let config_data: BTreeMap<String, Vec<String>> = load_conf(&config_path);
        let mut current_dir = std::env::current_dir().unwrap();
        current_dir.push(TEST_DIR);
        current_dir.push(TEST_DATA_DIR);

        for c in fs::read_dir(current_dir.as_path()).unwrap().flatten() {
            if let Ok(test_name) = c.file_name().into_string() {
                println!("Running test {test_name:?}");
                if c.path().is_dir() && !conf_data["exclude"].contains(&test_name) {
                    let verified = std::panic::catch_unwind(|| {
                        nargo::cli::prove_and_verify("pp", &c.path(), false)
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
                }
            }
        }
    }
}
