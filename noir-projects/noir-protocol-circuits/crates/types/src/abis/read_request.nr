use crate::{
    abis::side_effect::{OrderedValue, ContractScopedOrderedValue},
    traits::{Empty, Serialize, Deserialize}, address::AztecAddress
};
use dep::std::cmp::Eq;

global READ_REQUEST_SERIALIZED_LEN = 2;
global READ_REQUEST_CONTEXT_SERIALIZED_LEN = 3;

struct ReadRequest {
    value: Field,
    counter: u32,
}

impl OrderedValue<Field> for ReadRequest {
    fn value(self) -> Field {
        self.value
    }
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for ReadRequest {
    fn eq(self, read_request: ReadRequest) -> bool {
        (self.value == read_request.value)
        & (self.counter == read_request.counter)
    }
}

impl Empty for ReadRequest {
    fn empty() -> Self {
        ReadRequest {
            value: 0,
            counter: 0,
        }
    }
}

impl Serialize<READ_REQUEST_SERIALIZED_LEN> for ReadRequest {
    fn serialize(self) -> [Field; READ_REQUEST_SERIALIZED_LEN] {
        [self.value, self.counter as Field]
    }
}

impl Deserialize<READ_REQUEST_SERIALIZED_LEN> for ReadRequest {
    fn deserialize(values: [Field; READ_REQUEST_SERIALIZED_LEN]) -> Self {
        Self {
            value: values[0],
            counter: values[1] as u32,
        }
    }
}

impl ReadRequest {
    pub fn to_context(self, contract_address: AztecAddress) -> ReadRequestContext {
        ReadRequestContext { value: self.value, counter: self.counter, contract_address }
    }
}

struct ReadRequestContext {
    value: Field,
    counter: u32,
    contract_address: AztecAddress,
}

impl OrderedValue<Field> for ReadRequestContext {
    fn value(self) -> Field {
        self.value
    }
    fn counter(self) -> u32 {
        self.counter
    }
}

impl ContractScopedOrderedValue<Field> for ReadRequestContext {
    fn value(self) -> Field {
        self.value
    }
    fn counter(self) -> u32 {
        self.counter
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Eq for ReadRequestContext {
    fn eq(self, read_request: ReadRequestContext) -> bool {
        (self.value == read_request.value)
        & (self.counter == read_request.counter)
        & (self.contract_address.eq(read_request.contract_address))
    }
}

impl Empty for ReadRequestContext {
    fn empty() -> Self {
        ReadRequestContext {
            value: 0,
            counter: 0,
            contract_address: AztecAddress::empty(),
        }
    }
}

impl Serialize<READ_REQUEST_CONTEXT_SERIALIZED_LEN> for ReadRequestContext {
    fn serialize(self) -> [Field; READ_REQUEST_CONTEXT_SERIALIZED_LEN] {
        [self.value, self.counter as Field, self.contract_address.to_field()]
    }
}

impl Deserialize<READ_REQUEST_CONTEXT_SERIALIZED_LEN> for ReadRequestContext {
    fn deserialize(values: [Field; READ_REQUEST_CONTEXT_SERIALIZED_LEN]) -> Self {
        Self {
            value: values[0],
            counter: values[1] as u32,
            contract_address: AztecAddress::from_field(values[2]),
        }
    }
}

