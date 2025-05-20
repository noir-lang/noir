use crate::{
    abis::side_effect::{Ordered, Scoped},
    address::AztecAddress,
    constants::{READ_REQUEST_LENGTH, SCOPED_READ_REQUEST_LEN},
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct ReadRequest {
    pub value: Field,
    pub counter: u32,
}

impl Ordered for ReadRequest {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for ReadRequest {
    fn eq(self, read_request: ReadRequest) -> bool {
        (self.value == read_request.value) & (self.counter == read_request.counter)
    }
}

impl Empty for ReadRequest {
    fn empty() -> Self {
        ReadRequest { value: 0, counter: 0 }
    }
}

impl Serialize<READ_REQUEST_LENGTH> for ReadRequest {
    fn serialize(self) -> [Field; READ_REQUEST_LENGTH] {
        [self.value, self.counter as Field]
    }
}

impl Deserialize<READ_REQUEST_LENGTH> for ReadRequest {
    fn deserialize(values: [Field; READ_REQUEST_LENGTH]) -> Self {
        Self { value: values[0], counter: values[1] as u32 }
    }
}

impl ReadRequest {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedReadRequest {
        ScopedReadRequest { read_request: self, contract_address }
    }
}

pub struct ScopedReadRequest {
    pub read_request: ReadRequest,
    pub contract_address: AztecAddress,
}

impl Scoped<ReadRequest> for ScopedReadRequest {
    fn inner(self) -> ReadRequest {
        self.read_request
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Eq for ScopedReadRequest {
    fn eq(self, other: ScopedReadRequest) -> bool {
        (self.read_request == other.read_request)
            & (self.contract_address.eq(other.contract_address))
    }
}

impl Empty for ScopedReadRequest {
    fn empty() -> Self {
        ScopedReadRequest {
            read_request: ReadRequest::empty(),
            contract_address: AztecAddress::empty(),
        }
    }
}

impl Serialize<SCOPED_READ_REQUEST_LEN> for ScopedReadRequest {
    fn serialize(self) -> [Field; SCOPED_READ_REQUEST_LEN] {
        array_concat(
            self.read_request.serialize(),
            [self.contract_address.to_field()],
        )
    }
}

impl Deserialize<SCOPED_READ_REQUEST_LEN> for ScopedReadRequest {
    fn deserialize(values: [Field; SCOPED_READ_REQUEST_LEN]) -> Self {
        let mut reader = Reader::new(values);
        let res = Self {
            read_request: reader.read_struct(ReadRequest::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

impl ScopedReadRequest {
    pub fn value(self) -> Field {
        self.read_request.value
    }
    pub fn counter(self) -> u32 {
        self.read_request.counter
    }
}

#[test]
fn serialization_of_empty_read() {
    let item = ReadRequest::empty();
    let serialized = item.serialize();
    let deserialized = ReadRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_empty_scoped() {
    let item = ScopedReadRequest::empty();
    let serialized = item.serialize();
    let deserialized = ScopedReadRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
