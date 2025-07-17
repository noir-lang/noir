use crate::errors::CliError;

use super::NargoConfig;
use clap::Args;
use nargo::constants::{PKG_FILE, SRC_DIR};
use nargo::package::{CrateName, PackageType};
use noir_artifact_cli::fs::artifact::write_to_file;
use std::path::PathBuf;

#[allow(rustdoc::broken_intra_doc_links)]
/// Create a Noir project in the current directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct InitCommand {
    /// Name of the package [default: current directory name]
    #[clap(long)]
    name: Option<CrateName>,

    /// Use a library template
    #[arg(long, conflicts_with = "bin", conflicts_with = "contract")]
    pub(crate) lib: bool,

    /// Use a binary template [default]
    #[arg(long, conflicts_with = "lib", conflicts_with = "contract")]
    pub(crate) bin: bool,

    /// Use a contract template
    #[arg(long, conflicts_with = "lib", conflicts_with = "bin")]
    pub(crate) contract: bool,
}

const BIN_EXAMPLE: &str = include_str!("./noir_template_files/binary.nr");
const CONTRACT_EXAMPLE: &str = include_str!("./noir_template_files/contract.nr");
const LIB_EXAMPLE: &str = include_str!("./noir_template_files/library.nr");

pub(crate) fn run(args: InitCommand, config: NargoConfig) -> Result<(), CliError> {
    let package_name = match args.name {
        Some(name) => name,
        None => {
            let name = config.program_dir.file_name().unwrap().to_str().unwrap();
            name.parse().map_err(CliError::InvalidPackageName)?
        }
    };

    let package_type = if args.lib {
        PackageType::Library
    } else if args.contract {
        PackageType::Contract
    } else {
        PackageType::Binary
    };
    initialize_project(config.program_dir, package_name, package_type)
}

/// Initializes a new Noir project in `package_dir`.
pub(crate) fn initialize_project(
    package_dir: PathBuf,
    package_name: CrateName,
    package_type: PackageType,
) -> Result<(), CliError> {
    let src_dir = package_dir.join(SRC_DIR);
    let toml_path = package_dir.join(PKG_FILE);

    // We don't want to accidentally overwrite an existing Nargo.toml file
    if toml_path.exists() {
        return Err(CliError::NargoInitCannotBeRunOnExistingPackages);
    }

    let toml_contents = format!(
        r#"[package]
name = "{package_name}"
type = "{package_type}"
authors = [""]

[dependencies]"#
    );

    write_to_file(toml_contents.as_bytes(), &toml_path).unwrap();

    let (example, entry_name) = match package_type {
        PackageType::Binary => (BIN_EXAMPLE, "main.nr"),
        PackageType::Contract => (CONTRACT_EXAMPLE, "main.nr"),
        PackageType::Library => (LIB_EXAMPLE, "lib.nr"),
    };
    let entry_path = src_dir.join(entry_name);

    // It's fine to run `nargo init` if a source directory exists:
    // it might be that the source is already there and the user just wants to create the Nargo.toml file.
    if !entry_path.exists() {
        write_to_file(example.as_bytes(), &entry_path).unwrap();
    }

    println!("Project successfully created! It is located at {}", package_dir.display());

    Ok(())
}
