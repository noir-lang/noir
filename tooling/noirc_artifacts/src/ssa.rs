use noirc_errors::call_stack::CallStack;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum SsaReport {
    Warning(InternalWarning),
    Bug(InternalBug),
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalWarning {
    #[error("Return variable contains a constant value")]
    ReturnConstant { call_stack: CallStack },
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalBug {
    #[error("Input to Brillig function is in a separate subgraph to output")]
    IndependentSubgraph { call_stack: CallStack },
    #[error("Brillig function call isn't properly covered by a manual constraint")]
    UncheckedBrilligCall { call_stack: CallStack },
    #[error("Assertion is always false")]
    AssertFailed { call_stack: CallStack, message: Option<String> },
}
