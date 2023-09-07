struct GrumpkinScalar {
    low: Field,
    high: Field,
}

impl GrumpkinScalar {
    fn new(low: Field, high: Field) -> Self {
        // TODO: max value check
        GrumpkinScalar { low, high }
    }
}

global GRUMPKIN_SCALAR_SERIALISED_LEN: Field = 2;

fn deserialise_grumpkin_scalar(fields: [Field; GRUMPKIN_SCALAR_SERIALISED_LEN]) -> GrumpkinScalar {
    GrumpkinScalar { low: fields[0], high: fields[1] }
}

fn serialise_grumpkin_scalar(scalar: GrumpkinScalar) -> [Field; GRUMPKIN_SCALAR_SERIALISED_LEN] {
    [scalar.low, scalar.high]
}
