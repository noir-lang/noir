use noirc_frontend::hir::crate_graph::CrateType;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
// nargo is the package manager for Noir
// This name was used because it sounds like `cargo` and
// Noir Package Manager abbreviated is npm, which is already taken.

pub fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

pub mod cli;
pub(crate) mod git;
pub mod resolver;
pub mod toml;
/// Searches for the Nargo.toml file
///
/// XXX: In the end, this should find the root of the project and check
/// for the Nargo.toml file there
/// However, it should only do this after checking the current path
/// This allows the use of workspace settings in the future.
pub fn find_package_config(current_path: &Path) -> PathBuf {
    match fm::find_file(current_path, "Nargo", "toml") {
        Some(p) => p,
        None => write_stderr(&format!(
            "cannot find a Nargo.toml in {}",
            current_path.display()
        )),
    }
}

pub fn lib_or_bin(current_path: &Path) -> (PathBuf, CrateType) {
    // A library has a lib.nr and a binary has a main.nr
    // You cannot have both.
    let src_path = match fm::find_dir(current_path, "src") {
        Some(path) => path,
        None => write_stderr(&format!(
            "cannot find src file in path {}",
            current_path.display()
        )),
    };
    dbg!(&src_path);
    let lib_nr_path = fm::find_file(&src_path, "lib", "nr");
    let bin_nr_path = fm::find_file(&src_path, "main", "nr");
    match (lib_nr_path, bin_nr_path) {
        (Some(_), Some(_)) => {
            write_stderr("package cannot contain both a `lib.nr` and a `main.nr`")
        }
        (None, Some(path)) => (path, CrateType::Binary),
        (Some(path), None) => (path, CrateType::Library),
        (None, None) => {
            write_stderr("package must contain either a `lib.nr`(Library) or a `main.nr`(Binary).")
        }
    }
}

fn write_stderr(message: &str) -> ! {
    let mut stderr = StandardStream::stderr(ColorChoice::Always);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .expect("cannot set color for stderr in StandardStream");
    writeln!(&mut stderr, "{}", message).expect("cannot write to stderr");

    std::process::exit(0)
}
