#[macro_use]
extern crate afl;
extern crate nargo;
extern crate nargo_cli;
extern crate noirc_driver;

use std::path::PathBuf;

use nargo::workspace::Workspace;
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use noirc_driver::CompileOptions;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(debug_compile_stdin) = std::str::from_utf8(data) {
            let workspace = Workspace {
                root_dir: PathBuf::new(),
                target_dir: None,
                members: vec![],
                selected_package_index: None,

                // used by LSP
                is_assumed: false,
            };

            let mut compile_options = CompileOptions::default();
            compile_options.debug_compile_stdin = true;

            // TODO: test run with this enabled
            compile_options.pedantic_solving = true;

            let _ = compile_workspace_full(
                &workspace,
                &compile_options,
                Some(debug_compile_stdin.to_string()),
            );
        }
    });
}
