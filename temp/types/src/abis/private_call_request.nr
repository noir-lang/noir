use crate::{
    abis::{call_context::CallContext, side_effect::{Ordered, RangeOrdered}},
    constants::PRIVATE_CALL_REQUEST_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub struct PrivateCallRequest {
    pub call_context: CallContext,
    pub args_hash: Field,
    pub returns_hash: Field,
    pub start_side_effect_counter: u32,
    pub end_side_effect_counter: u32,
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
        (self.call_context == other.call_context)
            & (self.args_hash == other.args_hash)
            & (self.returns_hash == other.returns_hash)
            & (self.start_side_effect_counter == other.start_side_effect_counter)
            & (self.end_side_effect_counter == other.end_side_effect_counter)
    }
}

impl Empty for PrivateCallRequest {
    fn empty() -> Self {
        PrivateCallRequest {
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            start_side_effect_counter: 0,
            end_side_effect_counter: 0,
        }
    }
}

impl Serialize<PRIVATE_CALL_REQUEST_LENGTH> for PrivateCallRequest {
    fn serialize(self) -> [Field; PRIVATE_CALL_REQUEST_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CALL_REQUEST_LENGTH> = BoundedVec::new();

        fields.extend_from_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);
        fields.push(self.start_side_effect_counter as Field);
        fields.push(self.end_side_effect_counter as Field);

        assert_eq(fields.len(), PRIVATE_CALL_REQUEST_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_CALL_REQUEST_LENGTH> for PrivateCallRequest {
    fn deserialize(fields: [Field; PRIVATE_CALL_REQUEST_LENGTH]) -> PrivateCallRequest {
        let mut reader = Reader::new(fields);
        let item = PrivateCallRequest {
            call_context: reader.read_struct(CallContext::deserialize),
            args_hash: reader.read(),
            returns_hash: reader.read(),
            start_side_effect_counter: reader.read_u32(),
            end_side_effect_counter: reader.read_u32(),
        };
        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateCallRequest::empty();
    let serialized = item.serialize();
    let deserialized = PrivateCallRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
