use alloy_primitives::U256;
use super::{ForeignCall, ForeignCallError, ForeignCallExecutor};

/// Stub mock foreign call executor
pub struct MockForeignCallExecutor {
    mocks: Vec<()>, // Stub field
}

impl Default for MockForeignCallExecutor {
    fn default() -> Self {
        Self {
            mocks: Vec::new(),
        }
    }
}

impl MockForeignCallExecutor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ForeignCallExecutor<U256> for MockForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[U256],
    ) -> Result<Vec<U256>, ForeignCallError> {
        // Check if this is a mock-related call
        if let Some(mock_call) = ForeignCall::lookup(foreign_call) {
            match mock_call {
                ForeignCall::CreateMock |
                ForeignCall::SetMockParams |
                ForeignCall::GetMockLastParams |
                ForeignCall::SetMockReturns |
                ForeignCall::SetMockTimes |
                ForeignCall::ClearMock |
                ForeignCall::GetTimesCalled => {
                    // Return empty result for mock calls
                    Ok(Vec::new())
                }
                _ => Err(ForeignCallError::NoHandler(foreign_call.to_string()))
            }
        } else {
            Err(ForeignCallError::NoHandler(foreign_call.to_string()))
        }
    }
}

/// Stub disabled mock foreign call executor
pub struct DisabledMockForeignCallExecutor;

impl<F> ForeignCallExecutor<F> for DisabledMockForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        // Check if this is a mock-related call
        if let Some(mock_call) = ForeignCall::lookup(foreign_call) {
            match mock_call {
                ForeignCall::CreateMock |
                ForeignCall::SetMockParams |
                ForeignCall::GetMockLastParams |
                ForeignCall::SetMockReturns |
                ForeignCall::SetMockTimes |
                ForeignCall::ClearMock |
                ForeignCall::GetTimesCalled => {
                    Err(ForeignCallError::Disabled(foreign_call.to_string()))
                }
                _ => Err(ForeignCallError::NoHandler(foreign_call.to_string()))
            }
        } else {
            Err(ForeignCallError::NoHandler(foreign_call.to_string()))
        }
    }
}