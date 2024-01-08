use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use noirc_printable_type::{decode_string_value, ForeignCallError, PrintableValueDisplay};

pub trait ForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo,
    ) -> Result<ForeignCallResult, ForeignCallError>;
}

/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub(crate) enum ForeignCall {
    Print,
    CreateMock,
    SetMockParams,
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
    params: Option<Vec<ForeignCallParam>>,
    /// The result to return when this mock is called
    result: ForeignCallResult,
    /// How many times should this mock be called before it is removed
    times_left: Option<u64>,
}

impl MockedCall {
    fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            params: None,
            result: ForeignCallResult { values: vec![] },
            times_left: None,
        }
    }
}

impl MockedCall {
    fn matches(&self, name: &str, params: &Vec<ForeignCallParam>) -> bool {
        self.name == name && (self.params.is_none() || self.params.as_ref() == Some(params))
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
}

impl DefaultForeignCallExecutor {
    pub fn new(show_output: bool) -> Self {
        DefaultForeignCallExecutor { show_output, ..DefaultForeignCallExecutor::default() }
    }
}

impl DefaultForeignCallExecutor {
    fn extract_mock_id(
        foreign_call_inputs: &[ForeignCallParam],
    ) -> Result<(usize, &[ForeignCallParam]), ForeignCallError> {
        let (id, params) =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;
        Ok((id.unwrap_value().to_usize(), params))
    }

    fn find_mock_by_id(&mut self, id: usize) -> Option<&mut MockedCall> {
        self.mocked_responses.iter_mut().find(|response| response.id == id)
    }

    fn parse_string(param: &ForeignCallParam) -> String {
        let fields: Vec<_> = param.values().into_iter().map(|value| value.to_field()).collect();
        decode_string_value(&fields)
    }

    fn execute_print(foreign_call_inputs: &[ForeignCallParam]) -> Result<(), ForeignCallError> {
        let skip_newline = foreign_call_inputs[0].unwrap_value().is_zero();
        let display_values: PrintableValueDisplay = foreign_call_inputs
            .split_first()
            .ok_or(ForeignCallError::MissingForeignCallInputs)?
            .1
            .try_into()?;
        print!("{display_values}{}", if skip_newline { "" } else { "\n" });
        Ok(())
    }
}

impl ForeignCallExecutor for DefaultForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo,
    ) -> Result<ForeignCallResult, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
            Some(ForeignCall::Print) => {
                if self.show_output {
                    Self::execute_print(&foreign_call.inputs)?;
                }
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::CreateMock) => {
                let mock_oracle_name = Self::parse_string(&foreign_call.inputs[0]);
                assert!(ForeignCall::lookup(&mock_oracle_name).is_none());
                let id = self.last_mock_id;
                self.mocked_responses.push(MockedCall::new(id, mock_oracle_name));
                self.last_mock_id += 1;

                Ok(ForeignCallResult { values: vec![Value::from(id).into()] })
            }
            Some(ForeignCall::SetMockParams) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.find_mock_by_id(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .params = Some(params.to_vec());

                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::SetMockReturns) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.find_mock_by_id(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .result = ForeignCallResult { values: params.to_vec() };

                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::SetMockTimes) => {
                let (id, params) = Self::extract_mock_id(&foreign_call.inputs)?;
                let times = params[0]
                    .unwrap_value()
                    .to_field()
                    .try_to_u64()
                    .expect("Invalid bit size of times");

                self.find_mock_by_id(id)
                    .unwrap_or_else(|| panic!("Unknown mock id {}", id))
                    .times_left = Some(times);

                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::ClearMock) => {
                let (id, _) = Self::extract_mock_id(&foreign_call.inputs)?;
                self.mocked_responses.retain(|response| response.id != id);
                Ok(ForeignCallResult { values: vec![] })
            }
            None => {
                let response_position = self
                    .mocked_responses
                    .iter()
                    .position(|response| response.matches(foreign_call_name, &foreign_call.inputs))
                    .unwrap_or_else(|| panic!("Unknown foreign call {}", foreign_call_name));

                let mock = self
                    .mocked_responses
                    .get_mut(response_position)
                    .expect("Invalid position of mocked response");
                let result = mock.result.values.clone();

                if let Some(times_left) = &mut mock.times_left {
                    *times_left -= 1;
                    if *times_left == 0 {
                        self.mocked_responses.remove(response_position);
                    }
                }

                Ok(ForeignCallResult { values: result })
            }
        }
    }
}
