use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
    AcirField,
};
use jsonrpc::{arg as build_json_rpc_arg, minreq_http::Builder, Client};
use noirc_printable_type::{decode_string_value, ForeignCallError, PrintableValueDisplay};
use rand::Rng;
use serde::{Deserialize, Serialize};

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

/// This struct represents an oracle mock. It can be used for testing programs that use oracles.
#[derive(Debug, PartialEq, Eq, Clone)]
struct MockedCall<F> {
    /// The id of the mock, used to update or remove it
    id: usize,
    /// The oracle it's mocking
    name: String,
    /// Optionally match the parameters
    params: Option<Vec<ForeignCallParam<F>>>,
    /// The parameters with which the mock was last called
    last_called_params: Option<Vec<ForeignCallParam<F>>>,
    /// The result to return when this mock is called
    result: ForeignCallResult<F>,
    /// How many times should this mock be called before it is removed
    times_left: Option<u64>,
}

impl<F> MockedCall<F> {
    fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            params: None,
            last_called_params: None,
            result: ForeignCallResult { values: vec![] },
            times_left: None,
        }
    }
}

impl<F: PartialEq> MockedCall<F> {
    fn matches(&self, name: &str, params: &[ForeignCallParam<F>]) -> bool {
        self.name == name && (self.params.is_none() || self.params.as_deref() == Some(params))
    }
}

#[derive(Debug, Default)]
pub struct DefaultForeignCallExecutor<F> {
    /// A randomly generated id for this `DefaultForeignCallExecutor`.
    ///
    /// This is used so that a single `external_resolver` can distinguish between requests from multiple
    /// instantiations of `DefaultForeignCallExecutor`.
    id: u64,

    /// Mocks have unique ids used to identify them in Noir, allowing to update or remove them.
    last_mock_id: usize,
    /// The registered mocks
    mocked_responses: Vec<MockedCall<F>>,
    /// Whether to print [`ForeignCall::Print`] output.
    show_output: bool,
    /// JSON RPC client to resolve foreign calls
    external_resolver: Option<Client>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResolveForeignCallRequest<F> {
    /// A session ID which allows the external RPC server to link this foreign call request to other foreign calls
    /// for the same program execution.
    ///
    /// This is intended to allow a single RPC server to maintain state related to multiple program executions being
    /// performed in parallel.
    session_id: u64,

    #[serde(flatten)]
    /// The foreign call which the external RPC server is to provide a response for.
    function_call: ForeignCallWaitInfo<F>,
}

impl<F> DefaultForeignCallExecutor<F> {
    pub fn new(show_output: bool, resolver_url: Option<&str>) -> Self {
        let oracle_resolver = resolver_url.map(|resolver_url| {
            let mut transport_builder =
                Builder::new().url(resolver_url).expect("Invalid oracle resolver URL");

            if let Some(Ok(timeout)) =
                std::env::var("NARGO_FOREIGN_CALL_TIMEOUT").ok().map(|timeout| timeout.parse())
            {
                let timeout_duration = std::time::Duration::from_millis(timeout);
                transport_builder = transport_builder.timeout(timeout_duration);
            };
            Client::with_transport(transport_builder.build())
        });
        DefaultForeignCallExecutor {
            show_output,
            external_resolver: oracle_resolver,
            id: rand::thread_rng().gen(),
            mocked_responses: Vec::new(),
            last_mock_id: 0,
        }
    }
}

impl<F: AcirField> DefaultForeignCallExecutor<F> {
    fn extract_mock_id(
        foreign_call_inputs: &[ForeignCallParam<F>],
    ) -> Result<(usize, &[ForeignCallParam<F>]), ForeignCallError> {
        let (id, params) =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;
        let id =
            usize::try_from(id.unwrap_field().try_to_u64().expect("value does not fit into u64"))
                .expect("value does not fit into usize");
        Ok((id, params))
    }

    fn find_mock_by_id(&self, id: usize) -> Option<&MockedCall<F>> {
        self.mocked_responses.iter().find(|response| response.id == id)
    }

    fn find_mock_by_id_mut(&mut self, id: usize) -> Option<&mut MockedCall<F>> {
        self.mocked_responses.iter_mut().find(|response| response.id == id)
    }

    fn parse_string(param: &ForeignCallParam<F>) -> String {
        let fields: Vec<_> = param.fields().to_vec();
        decode_string_value(&fields)
    }

    fn execute_print(foreign_call_inputs: &[ForeignCallParam<F>]) -> Result<(), ForeignCallError> {
        let skip_newline = foreign_call_inputs[0].unwrap_field().is_zero();

        let foreign_call_inputs =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?.1;
        let display_string = Self::format_printable_value(foreign_call_inputs, skip_newline)?;

        print!("{display_string}");

        Ok(())
    }

    fn format_printable_value(
        foreign_call_inputs: &[ForeignCallParam<F>],
        skip_newline: bool,
    ) -> Result<String, ForeignCallError> {
        let display_values: PrintableValueDisplay<F> = foreign_call_inputs.try_into()?;

        let result = format!("{display_values}{}", if skip_newline { "" } else { "\n" });

        Ok(result)
    }
}

impl<F: AcirField + Serialize + for<'a> Deserialize<'a>> ForeignCallExecutor<F>
    for DefaultForeignCallExecutor<F>
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
            Some(ForeignCall::Print) => {
                if self.show_output {
                    Self::execute_print(&foreign_call.inputs)?;
                }
                Ok(ForeignCallResult::default())
            }
            Some(ForeignCall::CreateMock) => {
                let mock_oracle_name = Self::parse_string(&foreign_call.inputs[0]);
                assert!(ForeignCall::lookup(&mock_oracle_name).is_none());
                let id = self.last_mock_id;
                self.mocked_responses.push(MockedCall::new(id, mock_oracle_name));
                self.last_mock_id += 1;

                Ok(F::from(id).into())
            }
            Some(ForeignCall::SetMockParams) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.find_mock_by_id_mut(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .params = Some(params.to_vec());

                Ok(ForeignCallResult::default())
            }
            Some(ForeignCall::GetMockLastParams) => {
                let (id, _) = Self::extract_mock_id(&foreign_call.inputs)?;
                let mock =
                    self.find_mock_by_id(id).unwrap_or_else(|| panic!("Unknown mock id {}", id));

                let last_called_params = mock
                    .last_called_params
                    .clone()
                    .unwrap_or_else(|| panic!("Mock {} was never called", mock.name));

                Ok(last_called_params.into())
            }
            Some(ForeignCall::SetMockReturns) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.find_mock_by_id_mut(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .result = ForeignCallResult { values: params.to_vec() };

                Ok(ForeignCallResult::default())
            }
            Some(ForeignCall::SetMockTimes) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                let times =
                    params[0].unwrap_field().try_to_u64().expect("Invalid bit size of times");

                self.find_mock_by_id_mut(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .times_left = Some(times);

                Ok(ForeignCallResult::default())
            }
            Some(ForeignCall::ClearMock) => {
                let (id, _) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.mocked_responses.retain(|response| response.id != id);
                Ok(ForeignCallResult::default())
            }
            None => {
                let mock_response_position = self
                    .mocked_responses
                    .iter()
                    .position(|response| response.matches(foreign_call_name, &foreign_call.inputs));

                if let Some(response_position) = mock_response_position {
                    // If the program has registered a mocked response to this oracle call then we prefer responding
                    // with that.

                    let mock = self
                        .mocked_responses
                        .get_mut(response_position)
                        .expect("Invalid position of mocked response");

                    mock.last_called_params = Some(foreign_call.inputs.clone());

                    let result = mock.result.values.clone();

                    if let Some(times_left) = &mut mock.times_left {
                        *times_left -= 1;
                        if *times_left == 0 {
                            self.mocked_responses.remove(response_position);
                        }
                    }

                    Ok(result.into())
                } else if let Some(external_resolver) = &self.external_resolver {
                    // If the user has registered an external resolver then we forward any remaining oracle calls there.

                    let encoded_params = vec![build_json_rpc_arg(ResolveForeignCallRequest {
                        session_id: self.id,
                        function_call: foreign_call.clone(),
                    })];

                    let req =
                        external_resolver.build_request("resolve_foreign_call", &encoded_params);

                    let response = external_resolver.send_request(req)?;

                    let parsed_response: ForeignCallResult<F> = response.result()?;

                    Ok(parsed_response)
                } else {
                    // If there's no registered mock oracle response and no registered resolver then we cannot
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
}

#[cfg(test)]
mod tests {
    use acvm::{
        acir::brillig::ForeignCallParam, brillig_vm::brillig::ForeignCallResult,
        pwg::ForeignCallWaitInfo, FieldElement,
    };
    use jsonrpc_core::Result as RpcResult;
    use jsonrpc_derive::rpc;
    use jsonrpc_http_server::{Server, ServerBuilder};

    use crate::ops::{DefaultForeignCallExecutor, ForeignCallExecutor};

    use super::ResolveForeignCallRequest;

    #[allow(unreachable_pub)]
    #[rpc]
    pub trait OracleResolver {
        #[rpc(name = "resolve_foreign_call")]
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>>;
    }

    struct OracleResolverImpl;

    impl OracleResolverImpl {
        fn echo(&self, param: ForeignCallParam<FieldElement>) -> ForeignCallResult<FieldElement> {
            vec![param].into()
        }

        fn sum(&self, array: ForeignCallParam<FieldElement>) -> ForeignCallResult<FieldElement> {
            let mut res: FieldElement = 0_usize.into();

            for value in array.fields() {
                res += value;
            }

            res.into()
        }
    }

    impl OracleResolver for OracleResolverImpl {
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>> {
            let response = match req.function_call.function.as_str() {
                "sum" => self.sum(req.function_call.inputs[0].clone()),
                "echo" => self.echo(req.function_call.inputs[0].clone()),
                "id" => FieldElement::from(req.session_id as u128).into(),

                _ => panic!("unexpected foreign call"),
            };
            Ok(response)
        }
    }

    fn build_oracle_server() -> (Server, String) {
        let mut io = jsonrpc_core::IoHandler::new();
        io.extend_with(OracleResolverImpl.to_delegate());

        // Choosing port 0 results in a random port being assigned.
        let server = ServerBuilder::new(io)
            .start_http(&"127.0.0.1:0".parse().expect("Invalid address"))
            .expect("Could not start server");

        let url = format!("http://{}", server.address());
        (server, url)
    }

    #[test]
    fn test_oracle_resolver_echo() {
        let (server, url) = build_oracle_server();

        let mut executor = DefaultForeignCallExecutor::<FieldElement>::new(false, Some(&url));

        let foreign_call = ForeignCallWaitInfo {
            function: "echo".to_string(),
            inputs: vec![ForeignCallParam::Single(1_u128.into())],
        };

        let result = executor.execute(&foreign_call);
        assert_eq!(result.unwrap(), ForeignCallResult { values: foreign_call.inputs });

        server.close();
    }

    #[test]
    fn test_oracle_resolver_sum() {
        let (server, url) = build_oracle_server();

        let mut executor = DefaultForeignCallExecutor::new(false, Some(&url));

        let foreign_call = ForeignCallWaitInfo {
            function: "sum".to_string(),
            inputs: vec![ForeignCallParam::Array(vec![1_usize.into(), 2_usize.into()])],
        };

        let result = executor.execute(&foreign_call);
        assert_eq!(result.unwrap(), FieldElement::from(3_usize).into());

        server.close();
    }

    #[test]
    fn foreign_call_executor_id_is_persistent() {
        let (server, url) = build_oracle_server();

        let mut executor = DefaultForeignCallExecutor::<FieldElement>::new(false, Some(&url));

        let foreign_call = ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor.execute(&foreign_call).unwrap();
        let result_2 = executor.execute(&foreign_call).unwrap();
        assert_eq!(result_1, result_2);

        server.close();
    }

    #[test]
    fn oracle_resolver_rpc_can_distinguish_executors() {
        let (server, url) = build_oracle_server();

        let mut executor_1 = DefaultForeignCallExecutor::<FieldElement>::new(false, Some(&url));
        let mut executor_2 = DefaultForeignCallExecutor::<FieldElement>::new(false, Some(&url));

        let foreign_call = ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor_1.execute(&foreign_call).unwrap();
        let result_2 = executor_2.execute(&foreign_call).unwrap();
        assert_ne!(result_1, result_2);

        server.close();
    }
}
