use crate::errors::CliError;

use super::fs::{create_named_dir, write_to_file};
use super::{NargoConfig, CARGO_PKG_VERSION};
use acvm::Backend;
use clap::Args;
use nargo::constants::{PKG_FILE, SRC_DIR};
use nargo::package::PackageType;
use std::path::PathBuf;

/// Create a Noir project in the current directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct InitCommand {
    /// Use a library template
    #[clap(long)]
    pub(crate) lib: bool,

    /// Use a binary template [default]
    #[clap(long)]
    pub(crate) bin: bool,
}

const BIN_EXAMPLE: &str = r#"fn main(x : Field, y : pub Field) {
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
"#;

const LIB_EXAMPLE: &str = r#"fn my_util(x : Field, y : Field) -> bool {
    x != y
}

#[test]
fn test_main() {
    assert(my_util(1, 2));

    // Uncomment to make test fail
    // assert(my_util(1, 1));
}
"#;

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the "new" template in the future
    _backend: &B,
    args: InitCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let package_type = if args.lib { PackageType::Library } else { PackageType::Binary };
    initialize_project(config.program_dir, package_type);
    Ok(())
}

/// Initializes a new Noir project in `package_dir`.
pub(crate) fn initialize_project(package_dir: PathBuf, package_type: PackageType) {
    // TODO: Should this reject if we have non-Unicode filepaths?
    let package_name = package_dir.file_name().expect("Expected a filename").to_string_lossy();
    let src_dir = package_dir.join(SRC_DIR);
    create_named_dir(&src_dir, "src");

    // TODO: Need to make type configurable
    let toml_contents = format!(
        r#"[package]
name = "{package_name}"
type = "{package_type}"
authors = [""]
compiler_version = "{CARGO_PKG_VERSION}"

[dependencies]"#
    );

    write_to_file(toml_contents.as_bytes(), &package_dir.join(PKG_FILE));
    // This is a match so we get a compile error when we add to the package types
    match package_type {
        PackageType::Binary => write_to_file(BIN_EXAMPLE.as_bytes(), &src_dir.join("main.nr")),
        PackageType::Library => write_to_file(LIB_EXAMPLE.as_bytes(), &src_dir.join("lib.nr")),
    };
    println!("Project successfully created! It is located at {}", package_dir.display());
}
