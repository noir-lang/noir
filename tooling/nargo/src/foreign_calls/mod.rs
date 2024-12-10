use std::path::PathBuf;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use mocker::MockForeignCallExecutor;
use noirc_printable_type::ForeignCallError;
use print::{PrintForeignCallExecutor, PrintOutput};
use rand::Rng;
use rpc::RPCForeignCallExecutor;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default)]
pub struct DefaultForeignCallExecutor<'a, F> {
    /// The executor for any [`ForeignCall::Print`] calls.
    printer: PrintForeignCallExecutor<'a>,
    mocker: MockForeignCallExecutor<F>,
    external: Option<RPCForeignCallExecutor>,
}

impl<'a, F: Default> DefaultForeignCallExecutor<'a, F> {
    pub fn new(
        output: PrintOutput<'a>,
        resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        let id = rand::thread_rng().gen();
        let printer = PrintForeignCallExecutor { output };
        let external_resolver = resolver_url.map(|resolver_url| {
            RPCForeignCallExecutor::new(resolver_url, id, root_path, package_name)
        });
        DefaultForeignCallExecutor {
            printer,
            mocker: MockForeignCallExecutor::default(),
            external: external_resolver,
        }
    }
}

impl<'a, F: AcirField + Serialize + for<'b> Deserialize<'b>> ForeignCallExecutor<F>
    for DefaultForeignCallExecutor<'a, F>
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
            Some(ForeignCall::Print) => self.printer.execute(foreign_call),
            Some(
                ForeignCall::CreateMock
                | ForeignCall::SetMockParams
                | ForeignCall::GetMockLastParams
                | ForeignCall::SetMockReturns
                | ForeignCall::SetMockTimes
                | ForeignCall::ClearMock,
            ) => self.mocker.execute(foreign_call),

            None => {
                // First check if there's any defined mock responses for this foreign call.
                match self.mocker.execute(foreign_call) {
                    Err(ForeignCallError::NoHandler(_)) => (),
                    response_or_error => return response_or_error,
                };

                if let Some(external_resolver) = &mut self.external {
                    // If the user has registered an external resolver then we forward any remaining oracle calls there.
                    match external_resolver.execute(foreign_call) {
                        Err(ForeignCallError::NoHandler(_)) => (),
                        response_or_error => return response_or_error,
                    };
                }

                // If all executors have no handler for the given foreign call then we cannot
                // return a correct response to the ACVM. The best we can do is to return an empty response,
                // this allows us to ignore any foreign calls which exist solely to pass information from inside
                // the circuit to the environment (e.g. custom logging) as the execution will still be able to progress.
                //
                // We optimistically return an empty response for all oracle calls as the ACVM will error
                // should a response have been required.
                Ok(ForeignCallResult::default())
            }
        }
    }
}
