pub use self::check::check_program;
pub use self::compile::{
    check_crate_and_report_errors, collect_errors, compile_contract, compile_program,
    compile_program_with_debug_instrumenter, compile_workspace, report_errors,
};
pub use self::optimize::{optimize_contract, optimize_program};
pub use self::transform::{transform_contract, transform_program};

pub use self::execute::{execute_program, execute_program_with_profiling};
pub use self::fuzz::{
    FuzzExecutionConfig, FuzzFolderConfig, FuzzingRunStatus, run_fuzzing_harness,
};
pub use self::test::{
    TestStatus, check_expected_failure_message, run_test, test_status_program_compile_fail,
    test_status_program_compile_pass,
};

mod check;
mod compile;
pub mod debug;
mod execute;
mod fuzz;
mod optimize;
mod test;
mod transform;
