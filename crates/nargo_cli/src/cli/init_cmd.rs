use crate::errors::CliError;

use super::fs::{create_named_dir, write_to_file};
use super::{NargoConfig, CARGO_PKG_VERSION};
use acvm::Backend;
use clap::Args;
use nargo::constants::{PKG_FILE, SRC_DIR};
use std::path::PathBuf;

/// Create a Noir project in the current directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct InitCommand {
    /// Name of the package (Defaults to current directory name)
    #[clap(long)]
    name: Option<String>,
}

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
    args: InitCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let package_name = args
        .name
        .unwrap_or_else(|| config.program_dir.file_name().unwrap().to_str().unwrap().to_owned());

    initialize_project(config.program_dir, &package_name);
    Ok(())
}

/// Initializes a new Noir project in `package_dir`.
pub(crate) fn initialize_project(package_dir: PathBuf, package_name: &str) {
    // TODO: Should this reject if we have non-Unicode filepaths?
    let src_dir = package_dir.join(SRC_DIR);
    create_named_dir(&src_dir, "src");

    let toml_contents = format!(
        r#"[package]
name = "{package_name}"
authors = [""]
compiler_version = "{CARGO_PKG_VERSION}"

[dependencies]"#
    );

    write_to_file(toml_contents.as_bytes(), &package_dir.join(PKG_FILE));
    write_to_file(EXAMPLE.as_bytes(), &src_dir.join("main.nr"));
    println!("Project successfully created! Binary located at {}", package_dir.display());
}
