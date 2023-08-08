use crate::state_vars::type_serialisation::TypeSerialisationInterface;

global U32_SERIALISED_LEN: Field = 1;

fn deserialiseU32(fields: [Field; U32_SERIALISED_LEN]) -> u32 {
    fields[0] as u32
}

fn serialiseU32(value: u32) -> [Field; U32_SERIALISED_LEN] {
    [value as Field]
}

global U32SerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseU32,
    serialise: serialiseU32,
};