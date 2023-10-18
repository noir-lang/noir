use crate::backends::Backend;
use crate::errors::CliError;

use super::fs::{create_named_dir, write_to_file};
use super::{NargoConfig, CARGO_PKG_VERSION};
use clap::Args;
use nargo::constants::{PKG_FILE, SRC_DIR};
use nargo::package::PackageType;
use noirc_frontend::graph::CrateName;
use std::path::PathBuf;

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

pub(crate) fn run(
    // Backend is currently unused, but we might want to use it to inform the "new" template in the future
    _backend: &Backend,
    args: InitCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let package_name = match args.name {
        Some(name) => name,
        None => {
            let name = config.program_dir.file_name().unwrap().to_str().unwrap();
            name.parse().map_err(|_| CliError::InvalidPackageName(name.into()))?
        }
    };

    let package_type = if args.lib {
        PackageType::Library
    } else if args.contract {
        PackageType::Contract
    } else {
        PackageType::Binary
    };
    initialize_project(config.program_dir, package_name, package_type);
    Ok(())
}

/// Initializes a new Noir project in `package_dir`.
pub(crate) fn initialize_project(
    package_dir: PathBuf,
    package_name: CrateName,
    package_type: PackageType,
) {
    let src_dir = package_dir.join(SRC_DIR);
    create_named_dir(&src_dir, "src");

    let toml_contents = format!(
        r#"[package]
name = "{package_name}"
type = "{package_type}"
authors = [""]
compiler_version = "{CARGO_PKG_VERSION}"

[dependencies]"#
    );

    write_to_file(toml_contents.as_bytes(), &package_dir.join(PKG_FILE));
    // This uses the `match` syntax instead of `if` so we get a compile error when we add new package types (which likely need new template files)
    match package_type {
        PackageType::Binary => write_to_file(BIN_EXAMPLE.as_bytes(), &src_dir.join("main.nr")),
        PackageType::Contract => {
            write_to_file(CONTRACT_EXAMPLE.as_bytes(), &src_dir.join("main.nr"))
        }
        PackageType::Library => write_to_file(LIB_EXAMPLE.as_bytes(), &src_dir.join("lib.nr")),
    };
    println!("Project successfully created! It is located at {}", package_dir.display());
}
