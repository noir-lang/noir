use crate::types::type_serialisation::TypeSerialisationInterface;

global FIELD_SERIALISED_LEN: Field = 1;

fn deserialiseField(fields: [Field; FIELD_SERIALISED_LEN]) -> Field {
    fields[0]
}

fn serialiseField(value: Field) -> [Field; FIELD_SERIALISED_LEN] {
    [value]
}

global FieldSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseField,
    serialise: serialiseField,
};