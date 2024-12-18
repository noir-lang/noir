use std::path::PathBuf;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use mocker::MockForeignCallExecutor;
use noirc_printable_type::ForeignCallError;
use print::{PrintForeignCallExecutor, PrintOutput};
use rand::Rng;
use rpc::RPCForeignCallExecutor;
use serde::{Deserialize, Serialize};

pub mod layers;
pub(crate) mod mocker;
pub(crate) mod print;
pub(crate) mod rpc;

pub trait ForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError>;
}

/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub enum ForeignCall {
    Print,
    CreateMock,
    SetMockParams,
    GetMockLastParams,
    SetMockReturns,
    SetMockTimes,
    ClearMock,
}

impl std::fmt::Display for ForeignCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ForeignCall {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ForeignCall::Print => "print",
            ForeignCall::CreateMock => "create_mock",
            ForeignCall::SetMockParams => "set_mock_params",
            ForeignCall::GetMockLastParams => "get_mock_last_params",
            ForeignCall::SetMockReturns => "set_mock_returns",
            ForeignCall::SetMockTimes => "set_mock_times",
            ForeignCall::ClearMock => "clear_mock",
        }
    }

    pub(crate) fn lookup(op_name: &str) -> Option<ForeignCall> {
        match op_name {
            "print" => Some(ForeignCall::Print),
            "create_mock" => Some(ForeignCall::CreateMock),
            "set_mock_params" => Some(ForeignCall::SetMockParams),
            "get_mock_last_params" => Some(ForeignCall::GetMockLastParams),
            "set_mock_returns" => Some(ForeignCall::SetMockReturns),
            "set_mock_times" => Some(ForeignCall::SetMockTimes),
            "clear_mock" => Some(ForeignCall::ClearMock),
            _ => None,
        }
    }
}

pub struct DefaultForeignCallExecutor;

impl DefaultForeignCallExecutor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, F: AcirField + Serialize + for<'de> Deserialize<'de> + 'a>(
        output: PrintOutput<'a>,
        resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> impl ForeignCallExecutor<F> + 'a {
        // Adding them in the opposite order, so print is the outermost layer.
        layers::Layer::default()
            .add(resolver_url.map(|resolver_url| {
                let id = rand::thread_rng().gen();
                RPCForeignCallExecutor::new(resolver_url, id, root_path, package_name)
            }))
            .add(MockForeignCallExecutor::default())
            .add(PrintForeignCallExecutor::new(output))
    }
}
