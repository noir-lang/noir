use crate::types::type_serialisation::TypeSerialisationInterface;

struct GrumpkinScalar {
    high: Field,
    low: Field,
}

impl GrumpkinScalar {
    fn new(high: Field, low: Field) -> Self {
        // TODO: max value check
        GrumpkinScalar { high, low }
    }
}

global GRUMPKIN_SCALAR_SERIALISED_LEN: Field = 2;

fn deserialise_grumpkin_scalar(fields: [Field; GRUMPKIN_SCALAR_SERIALISED_LEN]) -> GrumpkinScalar {
    GrumpkinScalar {
        high: fields[0],
        low: fields[1],
    }
}

fn serialise_grumpkin_scalar(scalar: GrumpkinScalar) -> [Field; GRUMPKIN_SCALAR_SERIALISED_LEN] {
    [scalar.high, scalar.low]
}

global GrumpkinScalarSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialise_grumpkin_scalar,
    serialise: serialise_grumpkin_scalar,
};