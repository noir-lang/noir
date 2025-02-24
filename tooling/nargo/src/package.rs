use std::{collections::BTreeMap, fmt::Display, path::PathBuf};

use acvm::acir::circuit::ExpressionWidth;
pub use noirc_driver::CrateName;

use crate::constants::PROVER_INPUT_FILE;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PackageType {
    Library,
    Binary,
    Contract,
}

impl Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library => write!(f, "lib"),
            Self::Binary => write!(f, "bin"),
            Self::Contract => write!(f, "contract"),
        }
    }
}

#[derive(Clone)]
pub enum Dependency {
    Local { package: Package },
    Remote { package: Package },
}

impl Dependency {
    pub fn is_binary(&self) -> bool {
        match self {
            Self::Local { package } | Self::Remote { package } => package.is_binary(),
        }
    }

    pub fn package_name(&self) -> &CrateName {
        match self {
            Self::Local { package } | Self::Remote { package } => &package.name,
        }
    }
}

#[derive(Clone)]
pub struct Package {
    pub version: Option<String>,
    // A semver string which specifies the compiler version required to compile this package
    pub compiler_required_version: Option<String>,
    pub root_dir: PathBuf,
    pub package_type: PackageType,
    pub entry_path: PathBuf,
    pub name: CrateName,
    pub dependencies: BTreeMap<CrateName, Dependency>,
    pub expression_width: Option<ExpressionWidth>,
}

impl Package {
    pub fn prover_input_path(&self) -> PathBuf {
        // TODO: This should be configurable, such as if we are looking for .json or .toml or custom paths
        // For now it is hard-coded to be toml.
        self.root_dir.join(format!("{PROVER_INPUT_FILE}.toml"))
    }

    pub fn is_binary(&self) -> bool {
        self.package_type == PackageType::Binary
    }

    pub fn is_contract(&self) -> bool {
        self.package_type == PackageType::Contract
    }

    pub fn is_library(&self) -> bool {
        self.package_type == PackageType::Library
    }
}
