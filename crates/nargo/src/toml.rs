use serde_derive::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::write_stderr;
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub package: Package,
    pub dependencies: BTreeMap<String, Dependency>,
}

impl Config {
    // Local paths are usually relative and are discouraged when sharing libraries
    // It is better to separate these into different packages.
    pub fn has_local_path(&self) -> bool {
        let mut has_local_path = false;
        for dep in self.dependencies.values() {
            if let Dependency::Path { .. } = dep {
                has_local_path = true;
                break;
            }
        }
        has_local_path
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Package {
    // Note: a package name is not needed unless there is a registry
    pub authors: Vec<String>,
    // If not compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    pub compiler_version: Option<String>,
    pub backend: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
pub enum Dependency {
    Github { git: String, tag: String },
    Path { path: String },
}

/// Parses a Nargo.toml file from it's path
/// The path to the toml file must be present.
/// Calling this function without this guarantee is an ICE.
pub fn parse<P: AsRef<Path>>(path_to_toml: P) -> Config {
    let toml_as_string =
        std::fs::read_to_string(&path_to_toml).expect("ice: path given for toml file is invalid");

    match parse_toml_str(&toml_as_string) {
        Ok(cfg) => cfg,
        Err(msg) => {
            let path = path_to_toml.as_ref();
            write_stderr(&format!("{}\n Location: {}", msg, path.display()))
        }
    }
}

fn parse_toml_str(toml_as_string: &str) -> Result<Config, String> {
    match toml::from_str::<Config>(&toml_as_string) {
        Ok(cfg) => Ok(cfg),
        Err(err) => {
            let mut message = "input.toml file is badly formed, could not parse\n\n".to_owned();
            // XXX: This error is not always that helpful, but it gives the line number
            // and the entry, in those cases
            // which is useful for telling the user where the error occurred
            // XXX: maybe there is a way to extract ErrorInner from toml
            message.push_str(&err.to_string());
            Err(message)
        }
    }
}

#[test]
fn parse_standard_toml() {
    let src = r#"

        [package]
        authors = ["kev", "foo"]
        compiler_version = "0.1"

        [dependencies]
        rand = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        cool = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        hello = {path = "./noir_driver"}
    "#;

    assert!(parse_toml_str(src).is_ok());
}
