use crate::{
    abis::{
    private_call_stack_item::PrivateCallStackItem, call_context::CallContext,
    function_data::FunctionData, caller_context::CallerContext,
    side_effect::{Ordered, RangeOrdered, Scoped}
},
    address::AztecAddress, constants::{PRIVATE_CALL_REQUEST_LENGTH, SCOPED_PRIVATE_CALL_REQUEST_LENGTH},
    traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};

struct PrivateCallRequest {
    target: AztecAddress,
    call_context: CallContext,
    function_data: FunctionData,
    args_hash: Field,
    returns_hash: Field,
    caller_context: CallerContext,
    start_side_effect_counter: u32,
    end_side_effect_counter: u32,
}

impl Ordered for PrivateCallRequest {
    fn counter(self) -> u32 {
        self.start_side_effect_counter
    }
}

impl RangeOrdered for PrivateCallRequest {
    fn counter_start(self) -> u32 {
        self.start_side_effect_counter
    }
    fn counter_end(self) -> u32 {
        self.end_side_effect_counter
    }
}

impl Eq for PrivateCallRequest {
    fn eq(self, other: PrivateCallRequest) -> bool {
        (self.target == other.target) 
            & (self.call_context == other.call_context) 
            & (self.function_data == other.function_data) 
            & (self.args_hash == other.args_hash) 
            & (self.returns_hash == other.returns_hash)
            & (self.caller_context == other.caller_context)
            & (self.start_side_effect_counter == other.start_side_effect_counter)
            & (self.end_side_effect_counter == other.end_side_effect_counter)
    }
}

impl Empty for PrivateCallRequest {
    fn empty() -> Self {
        PrivateCallRequest {
            target: AztecAddress::empty(),
            call_context: CallContext::empty(),
            function_data: FunctionData::empty(),
            args_hash: 0,
            returns_hash: 0,
            caller_context: CallerContext::empty(),
            start_side_effect_counter: 0,
            end_side_effect_counter: 0,
        }
    }
}

impl Serialize<PRIVATE_CALL_REQUEST_LENGTH> for PrivateCallRequest {
    fn serialize(self) -> [Field; PRIVATE_CALL_REQUEST_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CALL_REQUEST_LENGTH> = BoundedVec::new();

        fields.push(self.target.to_field());
        fields.extend_from_array(self.call_context.serialize());
        fields.extend_from_array(self.function_data.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);
        fields.extend_from_array(self.caller_context.serialize());
        fields.push(self.start_side_effect_counter as Field);
        fields.push(self.end_side_effect_counter as Field);

        assert_eq(fields.len(), PRIVATE_CALL_REQUEST_LENGTH);

        fields.storage
    }
}

impl Deserialize<PRIVATE_CALL_REQUEST_LENGTH> for PrivateCallRequest {
    fn deserialize(fields: [Field; PRIVATE_CALL_REQUEST_LENGTH]) -> PrivateCallRequest {
        let mut reader = Reader::new(fields);
        let item = PrivateCallRequest {
            target: reader.read_struct(AztecAddress::deserialize),
            call_context: reader.read_struct(CallContext::deserialize),
            function_data: reader.read_struct(FunctionData::deserialize),
            args_hash: reader.read(),
            returns_hash: reader.read(),
            caller_context: reader.read_struct(CallerContext::deserialize),
            start_side_effect_counter: reader.read_u32(),
            end_side_effect_counter: reader.read_u32(),
        };
        reader.finish();
        item
    }
}

impl PrivateCallRequest {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedPrivateCallRequest {
        ScopedPrivateCallRequest { call_request: self, contract_address }
    }

    pub fn matches_stack_item(self, stack_item: PrivateCallStackItem) -> bool {
        (self.target == stack_item.contract_address)
            & (self.call_context == stack_item.public_inputs.call_context)
            & (self.function_data == stack_item.function_data)
            & (self.args_hash == stack_item.public_inputs.args_hash)
            & (self.returns_hash == stack_item.public_inputs.returns_hash)
            & (self.start_side_effect_counter
                                == stack_item.public_inputs.start_side_effect_counter)
            & (self.end_side_effect_counter
                                    == stack_item.public_inputs.end_side_effect_counter)
    }
}

struct ScopedPrivateCallRequest {
    call_request: PrivateCallRequest,
    contract_address: AztecAddress,
}

impl Scoped<PrivateCallRequest> for ScopedPrivateCallRequest {
    fn inner(self) -> PrivateCallRequest {
        self.call_request
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Ordered for ScopedPrivateCallRequest {
    fn counter(self) -> u32 {
        self.call_request.counter_start()
    }
}

impl RangeOrdered for ScopedPrivateCallRequest {
    fn counter_start(self) -> u32 {
        self.call_request.counter_start()
    }
    fn counter_end(self) -> u32 {
        self.call_request.counter_end()
    }
}

impl Eq for ScopedPrivateCallRequest {
    fn eq(self, other: ScopedPrivateCallRequest) -> bool {
        (self.call_request == other.call_request)
            & (self.contract_address == other.contract_address)
    }
}

impl Empty for ScopedPrivateCallRequest {
    fn empty() -> Self {
        ScopedPrivateCallRequest {
            call_request: PrivateCallRequest::empty(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<SCOPED_PRIVATE_CALL_REQUEST_LENGTH> for ScopedPrivateCallRequest {
    fn serialize(self) -> [Field; SCOPED_PRIVATE_CALL_REQUEST_LENGTH] {
        let mut fields: BoundedVec<Field, SCOPED_PRIVATE_CALL_REQUEST_LENGTH> = BoundedVec::new();

        fields.extend_from_array(self.call_request.serialize());
        fields.extend_from_array(self.contract_address.serialize());

        assert_eq(fields.len(), SCOPED_PRIVATE_CALL_REQUEST_LENGTH);

        fields.storage
    }
}

impl Deserialize<SCOPED_PRIVATE_CALL_REQUEST_LENGTH> for ScopedPrivateCallRequest {
    fn deserialize(fields: [Field; SCOPED_PRIVATE_CALL_REQUEST_LENGTH]) -> ScopedPrivateCallRequest {
        let mut reader = Reader::new(fields);
        let item = ScopedPrivateCallRequest {
            call_request: reader.read_struct(PrivateCallRequest::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty() {
    let item = ScopedPrivateCallRequest::empty();
    let serialized = item.serialize();
    let deserialized = ScopedPrivateCallRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
