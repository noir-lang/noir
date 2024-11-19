pub use self::compile::{
    collect_errors, compile_contract, compile_program, compile_program_with_debug_instrumenter,
    compile_workspace, report_errors,
};
pub use self::execute::{execute_program, execute_program_with_profiling};
pub use self::foreign_calls::{DefaultForeignCallExecutor, ForeignCall, ForeignCallExecutor};
pub use self::optimize::{optimize_contract, optimize_program};
pub use self::transform::{transform_contract, transform_program};

pub use self::test::{run_test, TestStatus};

mod compile;
mod execute;
mod foreign_calls;
mod optimize;
mod test;
mod transform;
