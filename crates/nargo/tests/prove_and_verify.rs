use std::collections::BTreeMap;
use std::fs;

const TEST_DIR: &str = "tests";
const TEST_DATA_DIR: &str = "test_data";
const CONFIG_FILE: &str = "config.toml";

mod tests {
    use super::*;

    fn load_conf(conf_path: &String) -> BTreeMap<String, Vec<String>> {
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
        let mut cdir = std::env::current_dir().unwrap();
        cdir.push(TEST_DIR);
        cdir.push(TEST_DATA_DIR);

        //load config.tml file from test_data directory
        cdir.push(CONFIG_FILE);
        let config_path = std::fs::read_to_string(cdir).unwrap();
        let conf_data: BTreeMap<String, Vec<String>> = load_conf(&config_path);
        let mut cdir = std::env::current_dir().unwrap();
        cdir.push(TEST_DIR);
        cdir.push(TEST_DATA_DIR);

        for c in fs::read_dir(cdir.as_path()).unwrap() {
            if let Ok(c) = c {
                let test_name = c.file_name().into_string();
                match test_name {
                    Ok(str) => {
                        if c.path().is_dir() && !conf_data["exclude"].contains(&str) {
                            let r = nargo::cli::prove_and_verify("pp", &c.path(), false);
                            if conf_data["fail"].contains(&str) {
                                assert!(!r, "{:?} should not succeed", c.file_name());
                            } else {
                                assert!(r, "verification fail for {:?}", c.file_name());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
