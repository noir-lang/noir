use std::path::PathBuf;

use alloy_primitives::U256;
use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram};
use noirc_errors::{CustomDiagnostic, debug_info::DebugInfo};
use noirc_frontend::{
    hir::{
        Context,
        def_map::TestFunction,
    },
};

use crate::{
    NargoError,
    foreign_calls::{ForeignCallError, ForeignCallExecutor},
};

#[derive(Debug, Clone)]
pub enum TestStatus {
    Pass,
    Fail {
        message: String,
        error_diagnostic: Option<CustomDiagnostic>,
    },
    Skipped,
    CompileError(CustomDiagnostic),
}

impl TestStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, TestStatus::Pass | TestStatus::Skipped)
    }
}

/// Stub test executor - testing requires ACVM backend
pub struct TestForeignCallExecutor<'a> {
    pub write_to_file: &'a Option<PathBuf>,
}

impl<'a> ForeignCallExecutor<U256> for TestForeignCallExecutor<'a> {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[U256],
    ) -> Result<Vec<U256>, ForeignCallError> {
        Err(ForeignCallError::Other(
            format!("Test execution is not available in Sensei (requires ZK backend): {}", foreign_call)
        ))
    }
}

/// Stub - run test requires ACVM backend
pub fn run_test<'a>(
    _context: &Context,
    _test_function: &TestFunction,
    _should_fail: bool,
    _foreign_call_executor: &'a mut dyn ForeignCallExecutor<U256>,
    _compile_options: &CompileOptions,
) -> Result<TestStatus, Vec<CustomDiagnostic>> {
    Err(vec![CustomDiagnostic::simple_error(
        "Test execution is not available in Sensei (requires ZK backend)".to_string(),
        "Test functionality has been removed with ACVM".to_string(),
        noirc_errors::Location::dummy(),
    )])
}

/// Stub - test status from compile pass
pub fn test_status_program_compile_pass(
    _test_function: &TestFunction,
    _abi: &Abi,
    _debug: &[DebugInfo],
    _result: &Result<Vec<U256>, NargoError>,
) -> TestStatus {
    TestStatus::Fail {
        message: "Test execution is not available in Sensei (requires ZK backend)".to_string(),
        error_diagnostic: None,
    }
}

/// Stub - test status from compile fail
pub fn test_status_program_compile_fail(
    err: Vec<CustomDiagnostic>,
    _test_function: &TestFunction,
) -> TestStatus {
    TestStatus::CompileError(err.into_iter().next().unwrap_or_else(|| {
        CustomDiagnostic::simple_error(
            "Unknown compilation error".to_string(),
            "Compilation failed".to_string(),
            noirc_errors::Location::dummy(),
        )
    }))
}