mod context;
mod dap;
pub mod errors;
mod foreign_calls;
mod repl;
mod source_code_printer;

use std::io::{Read, Write};

use ::dap::errors::ServerError;
use ::dap::server::Server;
// TODO: extract these pub structs to its own module
pub use context::DebugExecutionResult;
pub use context::DebugProject;
pub use context::RunParams;

pub fn run_repl_session(project: DebugProject, run_params: RunParams) -> DebugExecutionResult {
    repl::run(project, run_params)
}

pub fn run_dap_loop<R: Read, W: Write>(
    server: &mut Server<R, W>,
    project: DebugProject,
    run_params: RunParams,
) -> Result<DebugExecutionResult, ServerError> {
    dap::run_session(server, project, run_params)
}
