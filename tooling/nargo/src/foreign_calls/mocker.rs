use std::marker::PhantomData;

use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
    AcirField,
};
use noirc_abi::decode_string_value;

use super::{ForeignCall, ForeignCallError, ForeignCallExecutor};

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
pub struct MockForeignCallExecutor<F> {
    /// Mocks have unique ids used to identify them in Noir, allowing to update or remove them.
    last_mock_id: usize,
    /// The registered mocks
    mocked_responses: Vec<MockedCall<F>>,
}

impl<F: AcirField> MockForeignCallExecutor<F> {
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
}

impl<F> ForeignCallExecutor<F> for MockForeignCallExecutor<F>
where
    F: AcirField,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
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
            _ => {
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
                } else {
                    Err(ForeignCallError::NoHandler(foreign_call_name.to_string()))
                }
            }
        }
    }
}

/// Handler that panics if any of the mock functions are called.
#[allow(dead_code)] // TODO: Make the mocker optional
pub(crate) struct DisabledMockForeignCallExecutor<F> {
    _field: PhantomData<F>,
}

impl<F> ForeignCallExecutor<F> for DisabledMockForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        if let Some(
            ForeignCall::CreateMock
            | ForeignCall::SetMockParams
            | ForeignCall::GetMockLastParams
            | ForeignCall::SetMockReturns
            | ForeignCall::SetMockTimes
            | ForeignCall::ClearMock,
        ) = ForeignCall::lookup(foreign_call_name)
        {
            panic!("unexpected mock call: {}", foreign_call.function)
        }
        Err(ForeignCallError::NoHandler(foreign_call.function.clone()))
    }
}
