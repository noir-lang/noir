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

    /// If there was a constraint failure, this field will be populated. Note that the test can
    /// still be successful (i.e self.success == true) when it's expected to fail.
    pub reason: Option<String>,

    /// Minimal reproduction test case for failing fuzz tests
    pub counterexample: Option<CounterExample>,
}

/// Returned by a single fuzz in the case of a successful run
#[derive(Clone, Debug)]
pub struct CaseOutcome {
    /// Data of a single fuzz test case
    pub case_id: TestCaseId,
    pub case: InputMap,
    pub witness: Option<WitnessStack<FieldElement>>,
    pub brillig_coverage: Option<Vec<u32>>,
    pub acir_time: u128,
    pub brillig_time: u128,
}

/// Returned by a single fuzz when there is a discrepancy between brillig and acir execution
#[derive(Clone, Debug)]
pub struct DiscrepancyOutcome {
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
    /// Minimal reproduction test case for failing test
    pub counterexample: CounterExample,
    /// The status of the call
    pub exit_reason: String,
}

/// Outcome of a single fuzz
#[derive(Clone, Debug)]
pub enum FuzzOutcome {
    Case(CaseOutcome),
    Discrepancy(DiscrepancyOutcome),
    CounterExample(CounterExampleOutcome),
}
