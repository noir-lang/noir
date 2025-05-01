#[macro_use]
extern crate afl;
extern crate nargo;
extern crate nargo_cli;
extern crate noirc_driver;

use std::path::PathBuf;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(debug_compile_stdin) = std::str::from_utf8(data) {

            // let args = nargo_cli::cli::compile_cmd::CompileCommand {
            //     package: None,
            //     workspace: false,
            //     compile_options: Default::default(),
            //     watch: false,
            // };

            // let mut nargo_config = nargo_cli::cli::NargoConfig::default();
            // nargo_config.program_dir = "/".into();

            // let args = ();
            // let nargo_config = ();

            let workspace = nargo::workspace::Workspace {
                root_dir: PathBuf::new(),
                target_dir: None,
                // TODO: members
                members: vec![],
                // TODO: Some(0) if one member?
                selected_package_index: None,

                // used by LSP
                is_assumed: false,
            };

            let mut compile_options = noirc_driver::CompileOptions::default();
            compile_options.debug_compile_stdin = true;

            let _ = nargo_cli::cli::compile_cmd::compile_workspace_full(
                &workspace,
                &compile_options,
                Some(debug_compile_stdin.to_string()),
            );
        }
    });
}
