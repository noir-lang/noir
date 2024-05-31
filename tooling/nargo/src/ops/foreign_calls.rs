use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
    AcirField, FieldElement,
};
use jsonrpc::{arg as build_json_rpc_arg, minreq_http::Builder, Client};
use noirc_printable_type::{decode_string_value, ForeignCallError, PrintableValueDisplay};

pub trait ForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<FieldElement>,
    ) -> Result<ForeignCallResult<FieldElement>, ForeignCallError>;
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
struct MockedCall {
    /// The id of the mock, used to update or remove it
    id: usize,
    /// The oracle it's mocking
    name: String,
    /// Optionally match the parameters
    params: Option<Vec<ForeignCallParam<FieldElement>>>,
    /// The parameters with which the mock was last called
    last_called_params: Option<Vec<ForeignCallParam<FieldElement>>>,
    /// The result to return when this mock is called
    result: ForeignCallResult<FieldElement>,
    /// How many times should this mock be called before it is removed
    times_left: Option<u64>,
}

impl MockedCall {
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

impl MockedCall {
    fn matches(&self, name: &str, params: &[ForeignCallParam<FieldElement>]) -> bool {
        self.name == name && (self.params.is_none() || self.params.as_deref() == Some(params))
    }
}

#[derive(Debug, Default)]
pub struct DefaultForeignCallExecutor {
    /// Mocks have unique ids used to identify them in Noir, allowing to update or remove them.
    last_mock_id: usize,
    /// The registered mocks
    mocked_responses: Vec<MockedCall>,
    /// Whether to print [`ForeignCall::Print`] output.
    show_output: bool,
    /// JSON RPC client to resolve foreign calls
    external_resolver: Option<Client>,
}

impl DefaultForeignCallExecutor {
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
            ..DefaultForeignCallExecutor::default()
        }
    }
}

impl DefaultForeignCallExecutor {
    fn extract_mock_id(
        foreign_call_inputs: &[ForeignCallParam<FieldElement>],
    ) -> Result<(usize, &[ForeignCallParam<FieldElement>]), ForeignCallError> {
        let (id, params) =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;
        let id =
            usize::try_from(id.unwrap_field().try_to_u64().expect("value does not fit into u64"))
                .expect("value does not fit into usize");
        Ok((id, params))
    }

    fn find_mock_by_id(&self, id: usize) -> Option<&MockedCall> {
        self.mocked_responses.iter().find(|response| response.id == id)
    }

    fn find_mock_by_id_mut(&mut self, id: usize) -> Option<&mut MockedCall> {
        self.mocked_responses.iter_mut().find(|response| response.id == id)
    }

    fn parse_string(param: &ForeignCallParam<FieldElement>) -> String {
        let fields: Vec<_> = param.fields().to_vec();
        decode_string_value(&fields)
    }

    fn execute_print(
        foreign_call_inputs: &[ForeignCallParam<FieldElement>],
    ) -> Result<(), ForeignCallError> {
        let skip_newline = foreign_call_inputs[0].unwrap_field().is_zero();

        let foreign_call_inputs =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?.1;
        let display_string = Self::format_printable_value(foreign_call_inputs, skip_newline)?;

        print!("{display_string}");

        Ok(())
    }

    fn format_printable_value(
        foreign_call_inputs: &[ForeignCallParam<FieldElement>],
        skip_newline: bool,
    ) -> Result<String, ForeignCallError> {
        let display_values: PrintableValueDisplay = foreign_call_inputs.try_into()?;

        let result = format!("{display_values}{}", if skip_newline { "" } else { "\n" });

        Ok(result)
    }
}

impl ForeignCallExecutor for DefaultForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<FieldElement>,
    ) -> Result<ForeignCallResult<FieldElement>, ForeignCallError> {
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

                Ok(FieldElement::from(id).into())
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

                    let encoded_params: Vec<_> =
                        foreign_call.inputs.iter().map(build_json_rpc_arg).collect();

                    let req = external_resolver.build_request(foreign_call_name, &encoded_params);

                    let response = external_resolver.send_request(req)?;

                    let parsed_response: ForeignCallResult<FieldElement> = response.result()?;

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

    #[allow(unreachable_pub)]
    #[rpc]
    pub trait OracleResolver {
        #[rpc(name = "echo")]
        fn echo(
            &self,
            param: ForeignCallParam<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>>;

        #[rpc(name = "sum")]
        fn sum(
            &self,
            array: ForeignCallParam<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>>;
    }

    struct OracleResolverImpl;

    impl OracleResolver for OracleResolverImpl {
        fn echo(
            &self,
            param: ForeignCallParam<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>> {
            Ok(vec![param].into())
        }

        fn sum(
            &self,
            array: ForeignCallParam<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>> {
            let mut res: FieldElement = 0_usize.into();

            for value in array.fields() {
                res += value;
            }

            Ok(res.into())
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

        let mut executor = DefaultForeignCallExecutor::new(false, Some(&url));

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
}
