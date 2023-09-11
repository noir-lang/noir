use crate::types::type_serialisation::TypeSerialisationInterface;

global BOOL_SERIALISED_LEN: Field = 1;

fn deserialiseBool(fields: [Field; BOOL_SERIALISED_LEN]) -> bool {
    fields[0] as bool
}

fn serialiseBool(value: bool) -> [Field; BOOL_SERIALISED_LEN] {
    [value as Field]
}

global BoolSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseBool,
    serialise: serialiseBool,
};