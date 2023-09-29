use crate::types::type_serialization::TypeSerializationInterface;
use crate::types::address::AztecAddress;

global AZTEC_ADDRESS_SERIALIZED_LEN: Field = 1;

fn deserialize(fields: [Field; AZTEC_ADDRESS_SERIALIZED_LEN]) -> AztecAddress {
    AztecAddress::new(fields[0])
}

fn serialize(value: AztecAddress) -> [Field; AZTEC_ADDRESS_SERIALIZED_LEN] {
    [value.address]
}

global AztecAddressSerializationMethods = TypeSerializationInterface {
    deserialize,
    serialize,
};