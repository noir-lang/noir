use crate::types::type_serialization::TypeSerializationInterface;

global U8_SERIALIZED_LEN: Field = 1;

fn deserializeU8(fields: [Field; U8_SERIALIZED_LEN]) -> u8 {
    fields[0] as u8
}

fn serializeU8(value: u8) -> [Field; U8_SERIALIZED_LEN] {
    [value as Field]
}

global U8SerializationMethods = TypeSerializationInterface {
    deserialize: deserializeU8,
    serialize: serializeU8,
};
