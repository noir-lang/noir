use crate::state_vars::public_state_value::PublicStateValueInterface;

global FIELD_VALUE_LEN: Field = 1;

fn deserialiseField(fields: [Field; FIELD_VALUE_LEN]) -> Field {
    fields[0]
}

fn serialiseField(value: Field) -> [Field; FIELD_VALUE_LEN] {
    [value]
}

global FieldValueInterface = PublicStateValueInterface {
    deserialise: deserialiseField,
    serialise: serialiseField,
};