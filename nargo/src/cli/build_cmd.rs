use clap::ArgMatches;
use std::path::Path;

use crate::resolver::Resolver;

pub(crate) fn run(_args: ArgMatches) {
    let package_dir = std::env::current_dir().unwrap();
    build_from_path(package_dir);
    println!("Constraint system successfully built!")
}
// This is exposed so that we can run the examples and verify that they pass
pub fn build_from_path<P: AsRef<Path>>(p: P) {
    let (mut driver, _) = Resolver::resolve_root_config(p.as_ref());
    driver.build();
}
