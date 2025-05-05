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

            let compile_options = CompileOptions {
                debug_compile_stdin: true,
                pedantic_solving: true,
                ..Default::default()
            };

            let _ = compile_workspace_full(
                &workspace,
                &compile_options,
                Some(debug_compile_stdin.to_string()),
            );
        }
    });
}
