use acvm::{acir::native_types::WitnessStack, FieldElement};
use noirc_abi::InputMap;

use crate::corpus::TestCaseId;

type CounterExample = InputMap;

#[derive(Debug)]
/// Returned to the fuzz op in case the fuzzer found a failure case
pub struct ProgramFailureResult {
    /// Failure message
    pub failure_reason: String,
    /// Failing testcase
    pub counterexample: CounterExample,
}
#[derive(Debug)]
/// The outcome of a fuzz test
pub enum FuzzTestResult {
    /// If the program has been executed properly and no failures have been found
    Success,
    /// If we've discovered a failing testcase
    ProgramFailure(ProgramFailureResult),

    /// Failed to load corpus or insert a file, etc (mb no write privileges for target folder)
    CorpusFailure(String),

    /// Will contain the foreign call failure string if there was a foreign call failure
    ForeignCallFailure(String),

    /// Will contain the reason for minimization failure (except in cases when a program failure was detected)
    MinimizationFailure(String),

    /// Successfully minimized corpus
    MinimizationSuccess,
}

/// Returned by a single fuzz in the case of a successful run
#[derive(Clone, Debug)]
pub struct SuccessfulCaseOutcome {
    /// Unique identifier of the testcase
    pub case_id: TestCaseId,

    /// Testcase contents
    pub case: InputMap,

    /// Resulting witness (only available if the acir program has been run)
    pub witness: Option<WitnessStack<FieldElement>>,

    /// Coverage from brillig execution (only available if  brillig program has been run)
    pub brillig_coverage: Option<Vec<u32>>,

    /// How much time executing the acir program took (0 if it hasn't been run)
    pub acir_time: u128,

    /// How much time executing the brillig program took (0 if it hasn't been run)
    pub brillig_time: u128,
}

/// Returned by a single fuzz when there is a discrepancy between brillig and acir execution
#[derive(Clone, Debug)]
pub struct DiscrepancyOutcome {
    /// Unique identifier of the testcase
    pub case_id: TestCaseId,

    /// Minimal reproduction test case for failing test
    pub counterexample: CounterExample,

    // True if the failure came from ACIR, false if from brillig
    pub acir_failed: bool,

    /// The status of the call
    pub exit_reason: String,
}

/// Returned by a single fuzz when a counterexample has been discovered
#[derive(Clone, Debug)]
pub struct CounterExampleOutcome {
    /// Unique identifier of the testcase
    pub case_id: TestCaseId,
    /// Minimal reproduction test case for failing test
    pub counterexample: CounterExample,
    /// The status of the call
    pub exit_reason: String,
}

/// Foreign Call issues
#[derive(Clone, Debug)]
pub struct ForeignCallErrorInFuzzing {
    pub exit_reason: String,
}

/// Outcome of a single execution (Brillig or Acir and Brillig)
#[derive(Clone, Debug)]
pub enum HarnessExecutionOutcome {
    Case(SuccessfulCaseOutcome),
    Discrepancy(DiscrepancyOutcome),
    CounterExample(CounterExampleOutcome),
    ForeignCallFailure(ForeignCallErrorInFuzzing),
}
