use super::{create_dir, write_to_file, PKG_FILE, SRC_DIR};
use clap::ArgMatches;
use std::path::{Path, PathBuf};

pub(crate) fn run(args: ArgMatches) {
    let cmd = args.subcommand_matches("new").unwrap();

    let package_name = cmd.value_of("package_name").unwrap();

    let mut package_dir = match cmd.value_of("path") {
        Some(path) => std::path::PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };
    package_dir.push(Path::new(package_name));
    if package_dir.exists() {
        println!(
            "error: destination {} already exists",
            package_dir.display()
        );
        std::process::exit(1)
    }

    let src_dir = package_dir.join(Path::new(SRC_DIR));
    create_src_dir(&src_dir);

    const EXAMPLE: &'static str = "
        fn main(x : Witness, y : Witness) {
            constrain x != y;
        }
    ";

    const INPUT: &'static str = r#"
        x = "5"
        y = "10"
    "#;

    const SETTINGS: &'static str = r#"
        [package]
        authors = [""]
        compiler_version = "0.1"
    
        [dependencies]
    "#;

    write_to_file(INPUT.as_bytes(), &src_dir.join(Path::new("input.toml")));
    write_to_file(SETTINGS.as_bytes(), &package_dir.join(Path::new(PKG_FILE)));
    let path = write_to_file(EXAMPLE.as_bytes(), &src_dir.join(Path::new("main.nr")));
    println!("Project successfully created! Binary located at {}", path);
}

fn create_src_dir<P: AsRef<Path>>(p: P) -> PathBuf {
    create_dir(p).expect("could not create `src` directory")
}
