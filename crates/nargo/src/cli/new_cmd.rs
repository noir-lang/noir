use crate::errors::CliError;

use super::{create_named_dir, write_to_file, PKG_FILE, SRC_DIR};
use clap::ArgMatches;
use std::path::Path;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let cmd = args.subcommand_matches("new").unwrap();

    let package_name = cmd.value_of("package_name").unwrap();

    let mut package_dir = match cmd.value_of("path") {
        Some(path) => std::path::PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };
    package_dir.push(Path::new(package_name));
    if package_dir.exists() {
        return Err(CliError::DestinationAlreadyExists(package_dir));
    }

    let src_dir = package_dir.join(Path::new(SRC_DIR));
    create_named_dir(&src_dir, "src");

    const EXAMPLE: &str =
        concat!("fn main(x : Field, y : pub Field) {\n", "    constrain x != y;\n", "}");

    const SETTINGS: &str = concat!(
        "[package]\n",
        "authors = [\"\"]\n",
        "compiler_version = \"0.1\"\n",
        "\n",
        "[dependencies]"
    );

    write_to_file(SETTINGS.as_bytes(), &package_dir.join(Path::new(PKG_FILE)));
    write_to_file(EXAMPLE.as_bytes(), &src_dir.join(Path::new("main.nr")));
    println!("Project successfully created! Binary located at {}", package_dir.display());
    Ok(())
}
