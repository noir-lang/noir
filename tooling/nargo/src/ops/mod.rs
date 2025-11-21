pub use self::check::check_program;
pub use self::compile::{
    check_crate_and_report_errors, collect_errors, compile_contract, compile_program,
    compile_program_with_debug_instrumenter, report_errors,
};
pub use self::optimize::{optimize_contract, optimize_program};
pub use self::transform::{transform_contract, transform_program};

pub use self::execute::{execute_program, execute_program_with_profiling};
pub use self::fuzz::{
    FuzzExecutionConfig, FuzzFolderConfig, FuzzingRunStatus, run_fuzzing_harness,
};
pub use self::test::{
    FuzzConfig, TestStatus, check_expected_failure_message, fuzz_test, run_or_fuzz_test, run_test,
    test_status_program_compile_fail, test_status_program_compile_pass,
};
pub use acvm::brillig_vm::Version as BrilligVmVersion;

mod check;
mod compile;
pub mod debug;
mod execute;
mod fuzz;
mod optimize;
mod test;
mod transform;
