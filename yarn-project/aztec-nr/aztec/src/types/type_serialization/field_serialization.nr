use crate::types::type_serialization::TypeSerializationInterface;

global FIELD_SERIALIZED_LEN: Field = 1;

fn deserializeField(fields: [Field; FIELD_SERIALIZED_LEN]) -> Field {
    fields[0]
}

fn serializeField(value: Field) -> [Field; FIELD_SERIALIZED_LEN] {
    [value]
}

global FieldSerializationMethods = TypeSerializationInterface {
    deserialize: deserializeField,
    serialize: serializeField,
};