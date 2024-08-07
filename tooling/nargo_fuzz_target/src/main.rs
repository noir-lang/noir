#[macro_use]
extern crate afl;
extern crate nargo_cli;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(debug_compile_stdin) = std::str::from_utf8(data) {

            let args = nargo_cli::cli::compile_cmd::CompileCommand {
                package: None,
                workspace: false,
                compile_options: Default::default(),
                watch: false,
            };

            let mut nargo_config = nargo_cli::cli::NargoConfig::default();
            nargo_config.program_dir = "/".into();

            let _ = nargo_cli::cli::compile_cmd::run_debug_compile_stdin(
                args,
                nargo_config,
                Some(debug_compile_stdin.to_string()),
            );
        }
    });
}
