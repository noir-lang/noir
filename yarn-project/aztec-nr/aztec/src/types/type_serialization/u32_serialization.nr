use crate::types::type_serialization::TypeSerializationInterface;

global U32_SERIALIZED_LEN: Field = 1;

fn deserializeU32(fields: [Field; U32_SERIALIZED_LEN]) -> u32 {
    fields[0] as u32
}

fn serializeU32(value: u32) -> [Field; U32_SERIALIZED_LEN] {
    [value as Field]
}

global U32SerializationMethods = TypeSerializationInterface {
    deserialize: deserializeU32,
    serialize: serializeU32,
};