use std::fs::{read_to_string, File};
use std::io::Write;

fn main() {
    lalrpop::Configuration::new()
        .emit_rerun_directives(true)
        .use_cargo_dir_conventions()
        .process()
        .unwrap();

    // here, we get a lint error from "extern crate core" so patching that until lalrpop does
    // (adding cfg directives appears to be unsupported by lalrpop)
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let parser_path = std::path::Path::new(&out_dir).join("noir_parser.rs");
    let content_str = read_to_string(parser_path.clone()).unwrap();
    let mut parser_file = File::create(parser_path).unwrap();
    for line in content_str.lines() {
        if line.contains("extern crate core") {
            parser_file
                .write_all(
                    format!("{}\n", line.replace("extern crate core", "use core")).as_bytes(),
                )
                .unwrap();
        } else {
            parser_file.write_all(format!("{}\n", line).as_bytes()).unwrap();
        }
    }
}
