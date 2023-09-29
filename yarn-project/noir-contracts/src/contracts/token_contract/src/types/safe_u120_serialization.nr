use dep::aztec::types::type_serialization::TypeSerializationInterface;
use dep::safe_math::SafeU120;

global SAFE_U120_SERIALIZED_LEN: Field = 1;

// This is safe when reading from storage IF only correct safeu120 was written to storage
fn deserializeU120(fields: [Field; SAFE_U120_SERIALIZED_LEN]) -> SafeU120 {
    SafeU120{value: fields[0] as u120}
}

fn serializeU120(value: SafeU120) -> [Field; SAFE_U120_SERIALIZED_LEN] {
    [value.value as Field]
}

global SafeU120SerializationMethods = TypeSerializationInterface {
    deserialize: deserializeU120,
    serialize: serializeU120,
};