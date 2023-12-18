use crate::types::type_serialization::TypeSerializationInterface;
use dep::protocol_types::{
    address::{
        AztecAddress,
        EthAddress
    },
};

global AZTEC_ADDRESS_SERIALIZED_LEN: Field = 1;

fn deserialize(fields: [Field; AZTEC_ADDRESS_SERIALIZED_LEN]) -> AztecAddress {
    AztecAddress::from_field(fields[0])
}

fn serialize(value: AztecAddress) -> [Field; AZTEC_ADDRESS_SERIALIZED_LEN] {
    [value.to_field()]
}

global AddressSerializationMethods = TypeSerializationInterface {
    deserialize,
    serialize,
};