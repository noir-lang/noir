use std::{collections::BTreeMap, path::PathBuf};

use noirc_frontend::graph::{CrateName, CrateType};

use crate::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

#[derive(Clone)]
pub enum Dependency {
    Local { package: Package },
    Remote { package: Package },
}

#[derive(Clone)]
pub struct Package {
    pub root_dir: PathBuf,
    pub crate_type: CrateType,
    pub entry_path: PathBuf,
    pub name: CrateName,
    pub dependencies: BTreeMap<CrateName, Dependency>,
}

impl Package {
    pub fn prover_input_path(&self) -> PathBuf {
        // TODO: This should be configurable, such as if we are looking for .json or .toml or custom paths
        // For now it is hard-coded to be toml.
        self.root_dir.join(format!("{PROVER_INPUT_FILE}.toml"))
    }
    pub fn verifier_input_path(&self) -> PathBuf {
        // TODO: This should be configurable, such as if we are looking for .json or .toml or custom paths
        // For now it is hard-coded to be toml.
        self.root_dir.join(format!("{VERIFIER_INPUT_FILE}.toml"))
    }
}
