use crate::{
    constants::{PKG_FILE, SRC_DIR},
    errors::CliError,
};

use super::fs::{create_named_dir, write_to_file};
use super::{NargoConfig, CARGO_PKG_VERSION};
use acvm::Backend;
use clap::Args;
use const_format::formatcp;
use std::path::PathBuf;

/// Create a Noir project in the current directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct InitCommand;

const SETTINGS: &str = formatcp!(
    r#"[package]
authors = [""]
compiler_version = "{CARGO_PKG_VERSION}"

[dependencies]"#,
);

const EXAMPLE: &str = r#"fn main(x : Field, y : pub Field) {
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
"#;

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the "new" template in the future
    _backend: &B,
    _args: InitCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    initialize_project(config.program_dir);
    Ok(())
}

/// Initializes a new Noir project in `package_dir`.
pub(crate) fn initialize_project(package_dir: PathBuf) {
    let src_dir = package_dir.join(SRC_DIR);
    create_named_dir(&src_dir, "src");

    write_to_file(SETTINGS.as_bytes(), &package_dir.join(PKG_FILE));
    write_to_file(EXAMPLE.as_bytes(), &src_dir.join("main.nr"));
    println!("Project successfully created! Binary located at {}", package_dir.display());
}
