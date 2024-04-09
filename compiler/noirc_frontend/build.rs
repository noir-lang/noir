fn main() {
    lalrpop::Configuration::new()
        .emit_rerun_directives(true)
        .use_cargo_dir_conventions()
        .process()
        .unwrap();
}
