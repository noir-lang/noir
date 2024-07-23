#[macro_use]
extern crate afl;
extern crate nargo_cli;
// extern crate noirc_driver;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(debug_compile_stdin) = std::str::from_utf8(data) {

            // let compile_options = noirc_driver::CompileOptions::default();
            let args = nargo_cli::cli::compile_cmd::CompileCommand {
                package: None,
                workspace: false,
                compile_options: Default::default(),
                watch: false,
            };

            let mut nargo_config = nargo_cli::cli::NargoConfig::default();
            // #[arg(long, hide = true, global = true, default_value = "./")]

            // ??
            nargo_config.program_dir = "/".into();
            nargo_config.debug_compile_stdin = true;

            let _ = nargo_cli::cli::compile_cmd::run_debug_compile_stdin(
                args,
                nargo_config,
                Some(debug_compile_stdin.to_string()),
            );
        }
    });
}

