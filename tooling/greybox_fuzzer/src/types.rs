use acvm::{acir::native_types::WitnessStack, FieldElement};
use noirc_abi::InputMap;

use crate::corpus::TestCaseId;

type CounterExample = InputMap;

/// The outcome of a fuzz test
#[derive(Debug)]
pub struct FuzzTestResult {
    /// Whether the test case was successful. This means that the program executed
    /// properly, or that there was a constraint failure and that the test was expected to fail
    /// (has the `should_fail` attribute)
    pub success: bool,

    /// Set if the PUT failed because of a foreign call
    pub foreign_call_failure: bool,

    /// If there was a constraint failure, this field will be populated. Note that the test can
    /// still be successful (i.e self.success == true) when it's expected to fail.
    /// It will also contain the foreign call failure string if there was a foreign call failure
    pub reason: Option<String>,

    /// Minimal reproduction test case for failing fuzz tests
    pub counterexample: Option<CounterExample>,
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

/// Outcome of a single fuzz
#[derive(Clone, Debug)]
pub enum FuzzOutcome {
    Case(SuccessfulCaseOutcome),
    Discrepancy(DiscrepancyOutcome),
    CounterExample(CounterExampleOutcome),
    ForeignCallFailure(ForeignCallErrorInFuzzing),
}
